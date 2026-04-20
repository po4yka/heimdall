import Darwin
import Foundation

public actor HeimdallHelperController {
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

    public func ensureServerRunning(port: Int) async {
        if let ownedHelper = self.ownedHelper, !ownedHelper.process.isRunning {
            self.ownedHelper = nil
        }

        let serverIsHealthy = await self.hasHealthyServer(on: port)
        let serverIsCompatible = serverIsHealthy ? await Self.hasCompatibleLiveProvidersPayload(port: port) : false

        if serverIsHealthy, !serverIsCompatible {
            await Self.stopProcessListening(on: port)
        }

        if serverIsHealthy, serverIsCompatible {
            if let ownedHelper = self.ownedHelper {
                let resolved = Self.resolveExecutable()
                let fingerprint = resolved.flatMap(Self.fingerprint(for:))
                if ownedHelper.port == port,
                   resolved?.path == ownedHelper.executable.path,
                   fingerprint == ownedHelper.fingerprint {
                    return
                }

                await self.stopOwnedHelper()
                if await self.hasHealthyServer(on: port) {
                    return
                }
            } else {
                return
            }
        }

        if let ownedHelper = self.ownedHelper,
           ownedHelper.port != port || !ownedHelper.process.isRunning {
            await self.stopOwnedHelper()
        }

        guard let executable = Self.resolveExecutable(),
              let fingerprint = Self.fingerprint(for: executable) else {
            return
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
            _ = await self.waitForHealthyServer(on: port, attempts: 15, intervalNanoseconds: 200_000_000)
        } catch {
            return
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

    private static func hasCompatibleLiveProvidersPayload(port: Int) async -> Bool {
        guard let url = URL(string: "http://127.0.0.1:\(port)/api/live-providers") else {
            return false
        }

        let configuration = URLSessionConfiguration.ephemeral
        configuration.timeoutIntervalForRequest = 1
        configuration.timeoutIntervalForResource = 1
        let session = URLSession(configuration: configuration)
        do {
            let (data, response) = try await session.data(from: url)
            guard let http = response as? HTTPURLResponse, http.statusCode == 200 else {
                return false
            }
            return Self.liveProvidersPayloadIncludesAuth(data)
        } catch {
            return false
        }
    }

    static func liveProvidersPayloadIncludesAuth(_ data: Data) -> Bool {
        guard let object = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let providers = object["providers"] as? [[String: Any]] else {
            return false
        }
        return providers.allSatisfy { $0["auth"] is [String: Any] }
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
}
