import Darwin
import Foundation
import HeimdallDomain
import HeimdallServices

public actor HeimdallHelperController: HelperRuntime {
    enum LiveProvidersProbeResult: Equatable {
        case compatible
        case incompatible
        case unavailable
    }

    struct HelperBinaryFingerprint: Equatable {
        var path: String
        var modificationTimeInterval: TimeInterval?
        var size: UInt64?
    }

    private struct OwnedHelper {
        var process: Process
        var executable: URL
        var fingerprint: HelperBinaryFingerprint
        var port: Int
        var launchID: UUID
    }

    private var ownedHelper: OwnedHelper?

    public init() {}

    public func ensureServerRunning(port: Int) async -> Bool {
        if let ownedHelper = self.ownedHelper, !ownedHelper.process.isRunning {
            self.ownedHelper = nil
        }

        guard let executable = Self.resolveExecutable(),
              let fingerprint = Self.fingerprint(for: executable) else {
            return false
        }

        let serverIsHealthy = await self.hasHealthyServer(on: port)
        let compatibility = serverIsHealthy ? await Self.liveProvidersProbeResult(port: port) : .unavailable
        let trustedExistingServer = Self.hasTrustedListener(
            on: port,
            expectedExecutable: executable,
            expectedFingerprint: fingerprint
        )

        if serverIsHealthy, compatibility == .incompatible, trustedExistingServer {
            await Self.stopProcessListening(on: port)
        }

        if Self.canReuseExistingServer(isHealthy: serverIsHealthy, compatibility: compatibility) {
            if let ownedHelper = self.ownedHelper {
                let resolved = Self.resolveExecutable()
                let resolvedFingerprint = resolved.flatMap(Self.fingerprint(for:))
                if ownedHelper.port == port,
                   resolved?.path == ownedHelper.executable.path,
                   resolvedFingerprint == ownedHelper.fingerprint {
                    return true
                }

                await self.stopOwnedHelper()
                if Self.canReuseExistingServer(
                    isHealthy: await self.hasHealthyServer(on: port),
                    compatibility: await Self.liveProvidersProbeResult(port: port)
                ) {
                    return true
                }
            } else if trustedExistingServer || compatibility == .compatible {
                return true
            }
            return false
        }

        if serverIsHealthy, compatibility == .unavailable {
            return trustedExistingServer
                ? await self.waitForReadyServer(on: port, attempts: 30, intervalNanoseconds: 200_000_000)
                : false
        }

        if let ownedHelper = self.ownedHelper,
           ownedHelper.port != port || !ownedHelper.process.isRunning {
            await self.stopOwnedHelper()
        }

        if !serverIsHealthy, !Self.listeningPIDs(on: port).isEmpty {
            return false
        }

        let process = Process()
        process.executableURL = executable
        process.arguments = [
            "dashboard",
            "--host", "127.0.0.1",
            "--port", "\(port)",
            "--watch",
            "--no-open",
            "--background-poll",
        ]
        process.standardOutput = Pipe()
        process.standardError = Pipe()
        let launchID = UUID()
        process.terminationHandler = { [weak self] _ in
            Task {
                await self?.clearOwnedHelper(launchID: launchID)
            }
        }
        do {
            try process.run()
            self.ownedHelper = OwnedHelper(
                process: process,
                executable: executable,
                fingerprint: fingerprint,
                port: port,
                launchID: launchID
            )
            return await self.waitForReadyServer(on: port, attempts: 50, intervalNanoseconds: 200_000_000)
        } catch {
            return false
        }
    }

    public func stopOwnedHelper() async {
        guard let ownedHelper = self.ownedHelper else { return }
        if ownedHelper.process.isRunning {
            ownedHelper.process.terminate()
            for _ in 0..<10 {
                if !ownedHelper.process.isRunning {
                    break
                }
                try? await Task.sleep(nanoseconds: 100_000_000)
            }
            if ownedHelper.process.isRunning {
                ownedHelper.process.interrupt()
            }
        }
        self.ownedHelper = nil
    }

    private func clearOwnedHelper(launchID: UUID) {
        guard self.ownedHelper?.launchID == launchID else { return }
        self.ownedHelper = nil
    }

    private func hasHealthyServer(on port: Int) async -> Bool {
        await Self.pingHealth(port: port)
    }

    private func waitForHealthyServer(
        on port: Int,
        attempts: Int,
        intervalNanoseconds: UInt64
    ) async -> Bool {
        for _ in 0..<attempts {
            if await Self.pingHealth(port: port) {
                return true
            }
            try? await Task.sleep(nanoseconds: intervalNanoseconds)
        }
        return false
    }

    private func waitForReadyServer(
        on port: Int,
        attempts: Int,
        intervalNanoseconds: UInt64
    ) async -> Bool {
        for _ in 0..<attempts {
            if await Self.liveProvidersProbeResult(port: port) == .compatible {
                return true
            }
            try? await Task.sleep(nanoseconds: intervalNanoseconds)
        }
        return false
    }

    static func resolveExecutable(
        bundleURL: URL = Bundle.main.bundleURL,
        pathEnv: String = ProcessInfo.processInfo.environment["PATH"] ?? ""
    ) -> URL? {
        let bundle = Self.bundledHelperURL(bundleURL: bundleURL)
        if FileManager.default.isExecutableFile(atPath: bundle.path) {
            return bundle
        }

        guard Self.shouldAllowPATHFallback(bundleURL: bundleURL) else {
            return nil
        }

        for path in pathEnv.split(separator: ":") {
            let candidate = URL(fileURLWithPath: String(path)).appendingPathComponent("claude-usage-tracker")
            if FileManager.default.isExecutableFile(atPath: candidate.path) {
                return candidate
            }
        }
        return nil
    }

    static func bundledHelperURL(bundleURL: URL = Bundle.main.bundleURL) -> URL {
        bundleURL
            .appendingPathComponent("Contents", isDirectory: true)
            .appendingPathComponent("Helpers", isDirectory: true)
            .appendingPathComponent("claude-usage-tracker", isDirectory: false)
    }

    static func shouldAllowPATHFallback(bundleURL: URL = Bundle.main.bundleURL) -> Bool {
        let path = bundleURL.path
        return path.contains("/.derived/") || path.contains("/DerivedData/")
    }

    static func fingerprint(for executable: URL) -> HelperBinaryFingerprint? {
        let values = try? executable.resourceValues(forKeys: [.contentModificationDateKey, .fileSizeKey])
        let size = values?.fileSize.flatMap(UInt64.init)
        return HelperBinaryFingerprint(
            path: executable.path,
            modificationTimeInterval: values?.contentModificationDate?.timeIntervalSince1970,
            size: size
        )
    }

    private static func pingHealth(port: Int) async -> Bool {
        guard let url = URL(string: "http://127.0.0.1:\(port)/api/health") else {
            return false
        }
        let configuration = URLSessionConfiguration.ephemeral
        configuration.timeoutIntervalForRequest = 1
        configuration.timeoutIntervalForResource = 1
        let session = URLSession(configuration: configuration)
        do {
            let (_, response) = try await session.data(from: url)
            guard let http = response as? HTTPURLResponse else {
                return false
            }
            return http.statusCode == 200
        } catch {
            return false
        }
    }

    private static func liveProvidersProbeResult(port: Int) async -> LiveProvidersProbeResult {
        guard let url = readinessProbeURL(port: port) else {
            return .unavailable
        }

        let configuration = URLSessionConfiguration.ephemeral
        configuration.timeoutIntervalForRequest = 5
        configuration.timeoutIntervalForResource = 5
        let session = URLSession(configuration: configuration)
        do {
            let (data, response) = try await session.data(from: url)
            guard let http = response as? HTTPURLResponse, http.statusCode == 200 else {
                return .unavailable
            }
            return Self.liveProvidersPayloadIsCompatible(data) ? .compatible : .incompatible
        } catch {
            return .unavailable
        }
    }

    static func liveProvidersPayloadIsCompatible(_ data: Data) -> Bool {
        guard let object = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let contractVersion = object["contract_version"] as? Int else {
            return false
        }
        return contractVersion == LiveProviderContract.version
    }

    static func readinessProbeURL(port: Int) -> URL? {
        var components = URLComponents()
        components.scheme = "http"
        components.host = "127.0.0.1"
        components.port = port
        components.path = "/api/live-providers"
        components.queryItems = [URLQueryItem(name: "startup", value: "true")]
        return components.url
    }

    static func canReuseExistingServer(
        isHealthy: Bool,
        compatibility: LiveProvidersProbeResult
    ) -> Bool {
        isHealthy && compatibility == .compatible
    }

    static func hasTrustedListener(
        on port: Int,
        expectedExecutable: URL,
        expectedFingerprint: HelperBinaryFingerprint
    ) -> Bool {
        self.hasTrustedListener(
            on: port,
            expectedExecutable: expectedExecutable,
            expectedFingerprint: expectedFingerprint,
            pidsProvider: Self.listeningPIDs(on:),
            pathProvider: Self.executablePath(for:)
        )
    }

    static func hasTrustedListener(
        on port: Int,
        expectedExecutable: URL,
        expectedFingerprint: HelperBinaryFingerprint,
        pidsProvider: (Int) -> [Int32],
        pathProvider: (Int32) -> String?
    ) -> Bool {
        let expectedPath = expectedExecutable.resolvingSymlinksInPath().path
        return pidsProvider(port).contains { pid in
            guard let path = pathProvider(pid) else {
                return false
            }
            let normalizedPath = URL(fileURLWithPath: path).resolvingSymlinksInPath().path
            guard normalizedPath == expectedPath else {
                return false
            }
            return Self.fingerprint(for: URL(fileURLWithPath: normalizedPath)) == expectedFingerprint
        }
    }

    private static func stopProcessListening(on port: Int) async {
        for pid in Self.listeningPIDs(on: port) {
            _ = Darwin.kill(pid_t(pid), SIGTERM)
        }

        for _ in 0..<10 {
            if Self.listeningPIDs(on: port).isEmpty {
                return
            }
            try? await Task.sleep(nanoseconds: 100_000_000)
        }

        for pid in Self.listeningPIDs(on: port) {
            _ = Darwin.kill(pid_t(pid), SIGKILL)
        }
    }

    private static func listeningPIDs(on port: Int) -> [Int32] {
        let process = Process()
        let output = Pipe()
        process.executableURL = URL(fileURLWithPath: "/usr/sbin/lsof")
        process.arguments = ["-t", "-iTCP:\(port)", "-sTCP:LISTEN", "-n", "-P"]
        process.standardOutput = output
        process.standardError = Pipe()

        do {
            try process.run()
        } catch {
            return []
        }

        process.waitUntilExit()
        guard process.terminationStatus == 0 || process.terminationStatus == 1 else {
            return []
        }

        let data = output.fileHandleForReading.readDataToEndOfFile()
        guard let string = String(data: data, encoding: .utf8) else {
            return []
        }

        return string
            .split(whereSeparator: \.isNewline)
            .compactMap { Int32($0) }
    }

    private static func executablePath(for pid: Int32) -> String? {
        var buffer = [CChar](repeating: 0, count: Int(MAXPATHLEN * 4))
        let result = proc_pidpath(pid, &buffer, UInt32(buffer.count))
        guard result > 0 else {
            return nil
        }
        return String(cString: buffer)
    }
}
