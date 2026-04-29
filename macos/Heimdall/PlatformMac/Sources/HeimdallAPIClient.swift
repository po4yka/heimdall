import Foundation
import HeimdallDomain
import HeimdallServices

public struct HeimdallAPIClient: LiveProviderClient, LiveMonitorClient, MobileSnapshotClient, Sendable {
    public var baseURL: URL
    private let session: URLSession
    private let retryPolicy: RetryPolicy
    private static let defaultRequestTimeout: TimeInterval = 3
    private static let startupRequestTimeout: TimeInterval = 1.5
    private static let liveMonitorRequestTimeout: TimeInterval = 30
    private static let forcedRefreshTimeout: TimeInterval = 45

    struct RetryPolicy: Sendable, Equatable {
        let attempts: Int
        let backoffNanoseconds: [UInt64]
    }

    public init(port: Int) {
        self.baseURL = URL(string: "http://127.0.0.1:\(port)")!
        let configuration = URLSessionConfiguration.ephemeral
        configuration.timeoutIntervalForRequest = Self.defaultRequestTimeout
        configuration.timeoutIntervalForResource = 30
        self.session = URLSession(configuration: configuration)
        self.retryPolicy = Self.localhostRetryPolicy
    }

    init(baseURL: URL, session: URLSession, retryPolicy: RetryPolicy = Self.localhostRetryPolicy) {
        self.baseURL = baseURL
        self.session = session
        self.retryPolicy = retryPolicy
    }

    public func fetchSnapshots() async throws -> ProviderSnapshotEnvelope {
        try self.validate(try await self.fetch(path: "/api/live-providers", as: ProviderSnapshotEnvelope.self))
    }

    public func fetchStartupSnapshots() async throws -> ProviderSnapshotEnvelope {
        var components = URLComponents(
            url: self.baseURL.appendingPathComponent("/api/live-providers"),
            resolvingAgainstBaseURL: false
        )!
        components.queryItems = [URLQueryItem(name: "startup", value: "true")]

        var request = URLRequest(url: components.url!)
        request.timeoutInterval = Self.startupRequestTimeout
        let (data, response) = try await self.data(for: request, retryPolicy: Self.startupRetryPolicy)
        guard let http = response as? HTTPURLResponse, 200..<300 ~= http.statusCode else {
            throw URLError(.badServerResponse)
        }
        return try self.validate(try JSONDecoder().decode(ProviderSnapshotEnvelope.self, from: data))
    }

    public func refresh(provider: ProviderID?) async throws -> ProviderSnapshotEnvelope {
        var components = URLComponents(url: self.baseURL.appendingPathComponent("/api/live-providers/refresh"), resolvingAgainstBaseURL: false)!
        if let provider {
            components.queryItems = [URLQueryItem(name: "provider", value: provider.rawValue)]
        }

        var request = URLRequest(url: components.url!)
        request.httpMethod = "POST"
        request.timeoutInterval = Self.forcedRefreshTimeout
        let (data, response) = try await self.data(for: request)
        guard let http = response as? HTTPURLResponse, 200..<300 ~= http.statusCode else {
            throw URLError(.badServerResponse)
        }
        return try self.validate(try JSONDecoder().decode(ProviderSnapshotEnvelope.self, from: data))
    }

    public func fetchCostSummary(provider: ProviderID) async throws -> CostSummaryEnvelope {
        var components = URLComponents(url: self.baseURL.appendingPathComponent("/api/live-providers/history"), resolvingAgainstBaseURL: false)!
        components.queryItems = [URLQueryItem(name: "provider", value: provider.rawValue)]
        return try await self.fetch(url: components.url!, as: CostSummaryEnvelope.self)
    }

    public func fetchMobileSnapshot() async throws -> MobileSnapshotEnvelope {
        try await self.fetch(path: "/api/mobile-snapshot", as: MobileSnapshotEnvelope.self)
    }

    /// Fetch the dashboard payload (`/api/data`) and return the slice the
    /// macOS client decodes into `DashboardSnapshot`. Source for cross-cutting
    /// surfaces — like the all-providers daily-by-model history — that aren't
    /// scoped to a single provider.
    public func fetchDashboardData() async throws -> DashboardSnapshot {
        try await self.fetch(path: "/api/data", as: DashboardSnapshot.self)
    }

    public func fetchLiveMonitor() async throws -> LiveMonitorEnvelope {
        var request = URLRequest(url: self.baseURL.appendingPathComponent("/api/live-monitor"))
        request.timeoutInterval = Self.liveMonitorRequestTimeout
        let (data, response) = try await self.data(for: request)
        guard let http = response as? HTTPURLResponse, 200..<300 ~= http.statusCode else {
            throw URLError(.badServerResponse)
        }
        return try self.validate(try JSONDecoder().decode(LiveMonitorEnvelope.self, from: data))
    }

    public func liveMonitorEvents() -> AsyncThrowingStream<String, Error> {
        AsyncThrowingStream { continuation in
            let session = self.session
            let url = self.baseURL.appendingPathComponent("/api/stream")
            let task = Task {
                do {
                    let (bytes, response) = try await session.bytes(from: url)
                    guard let http = response as? HTTPURLResponse, 200..<300 ~= http.statusCode else {
                        throw URLError(.badServerResponse)
                    }
                    for try await line in bytes.lines {
                        if line.hasPrefix("event:") {
                            continuation.yield(line.replacingOccurrences(of: "event:", with: "").trimmingCharacters(in: .whitespaces))
                        }
                    }
                    continuation.finish()
                } catch {
                    continuation.finish(throwing: error)
                }
            }

            continuation.onTermination = { _ in
                task.cancel()
            }
        }
    }

    private func fetch<T: Decodable>(path: String, as type: T.Type) async throws -> T {
        try await self.fetch(url: self.baseURL.appendingPathComponent(path), as: type)
    }

    private func fetch<T: Decodable>(url: URL, as type: T.Type) async throws -> T {
        let (data, response) = try await self.data(from: url)
        guard let http = response as? HTTPURLResponse, 200..<300 ~= http.statusCode else {
            throw URLError(.badServerResponse)
        }
        return try JSONDecoder().decode(T.self, from: data)
    }

    private func data(from url: URL) async throws -> (Data, URLResponse) {
        try await self.withRetry(policy: self.retryPolicy) {
            try await self.session.data(from: url)
        }
    }

    private func data(for request: URLRequest) async throws -> (Data, URLResponse) {
        try await self.data(for: request, retryPolicy: self.retryPolicy)
    }

    private func data(
        for request: URLRequest,
        retryPolicy: RetryPolicy
    ) async throws -> (Data, URLResponse) {
        try await self.withRetry(policy: retryPolicy) {
            try await self.session.data(for: request)
        }
    }

    private func withRetry<T>(
        policy: RetryPolicy,
        operation: @escaping @Sendable () async throws -> T
    ) async throws -> T {
        var lastError: Error?
        for attempt in 0..<policy.attempts {
            do {
                return try await operation()
            } catch {
                lastError = error
                guard attempt + 1 < policy.attempts, Self.isRetryableLocalhostError(error) else {
                    throw error
                }
                let sleepDuration = policy.backoffNanoseconds[min(attempt, policy.backoffNanoseconds.count - 1)]
                try? await Task.sleep(nanoseconds: sleepDuration)
            }
        }
        throw lastError ?? URLError(.unknown)
    }

    private static var localhostRetryPolicy: RetryPolicy {
        RetryPolicy(
            attempts: 4,
            backoffNanoseconds: [
                250_000_000,
                500_000_000,
                1_000_000_000,
            ]
        )
    }

    private static var startupRetryPolicy: RetryPolicy {
        RetryPolicy(
            attempts: 2,
            backoffNanoseconds: [
                150_000_000,
            ]
        )
    }

    private static func isRetryableLocalhostError(_ error: Error) -> Bool {
        guard let urlError = error as? URLError else {
            return false
        }
        switch urlError.code {
        case .cannotConnectToHost, .timedOut, .networkConnectionLost, .cannotFindHost:
            return true
        default:
            return false
        }
    }

    // Source of truth for this constant: src/models.rs LIVE_PROVIDERS_CONTRACT_VERSION.
    // This value must match the Rust constant for the version of the helper this app was built against.
    // Do NOT read the Rust constant at build time — independent constants are intentional so drift is detectable.
    static let liveProvidersContractVersion = LiveProviderContract.version

    private func validate(_ envelope: ProviderSnapshotEnvelope) throws -> ProviderSnapshotEnvelope {
        // Accept downgrade silently: an older helper cannot have future fields.
        // Reject upgrade loudly: a newer helper may have removed or renamed fields this app depends on.
        if envelope.contractVersion > Self.liveProvidersContractVersion {
            throw ContractVersionError.tooNew(
                wire: envelope.contractVersion,
                compiled: Self.liveProvidersContractVersion
            )
        }
        return envelope
    }

    private func validate(_ envelope: LiveMonitorEnvelope) throws -> LiveMonitorEnvelope {
        guard envelope.contractVersion == LiveMonitorContract.version else {
            throw URLError(.cannotDecodeContentData)
        }
        return envelope
    }
}

// MARK: - Contract version error

public enum ContractVersionError: Error, CustomStringConvertible {
    /// The bundled helper speaks a newer contract version than this app was built against.
    case tooNew(wire: Int, compiled: Int)

    public var description: String {
        switch self {
        case let .tooNew(wire, compiled):
            return "The bundled helper speaks contract version \(wire), but this app was built against version \(compiled). Please update Heimdall.app to match the helper."
        }
    }
}
