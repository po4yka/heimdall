import Foundation
import HeimdallDomain
import HeimdallServices

public struct HeimdallAPIClient: LiveProviderClient, Sendable {
    public var baseURL: URL
    private let session: URLSession
    private let retryPolicy: RetryPolicy
    private static let defaultRequestTimeout: TimeInterval = 3
    private static let forcedRefreshTimeout: TimeInterval = 12

    struct RetryPolicy: Sendable, Equatable {
        let attempts: Int
        let backoffNanoseconds: [UInt64]
    }

    public init(port: Int) {
        self.baseURL = URL(string: "http://127.0.0.1:\(port)")!
        let configuration = URLSessionConfiguration.ephemeral
        configuration.timeoutIntervalForRequest = Self.defaultRequestTimeout
        configuration.timeoutIntervalForResource = 8
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
        try await self.withRetry(policy: self.retryPolicy) {
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

    private func validate(_ envelope: ProviderSnapshotEnvelope) throws -> ProviderSnapshotEnvelope {
        guard envelope.contractVersion == LiveProviderContract.version else {
            throw URLError(.cannotDecodeContentData)
        }
        return envelope
    }
}
