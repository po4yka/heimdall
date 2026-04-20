import Foundation

public struct HeimdallAPIClient: Sendable {
    public var baseURL: URL

    public init(port: Int) {
        self.baseURL = URL(string: "http://127.0.0.1:\(port)")!
    }

    public func fetchSnapshots() async throws -> ProviderSnapshotEnvelope {
        try await self.fetch(path: "/api/live-providers", as: ProviderSnapshotEnvelope.self)
    }

    public func refresh(provider: ProviderID?) async throws -> ProviderSnapshotEnvelope {
        var components = URLComponents(url: self.baseURL.appendingPathComponent("/api/live-providers/refresh"), resolvingAgainstBaseURL: false)!
        if let provider {
            components.queryItems = [URLQueryItem(name: "provider", value: provider.rawValue)]
        }

        var request = URLRequest(url: components.url!)
        request.httpMethod = "POST"
        let (data, response) = try await URLSession.shared.data(for: request)
        guard let http = response as? HTTPURLResponse, 200..<300 ~= http.statusCode else {
            throw URLError(.badServerResponse)
        }
        return try JSONDecoder().decode(ProviderSnapshotEnvelope.self, from: data)
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
        let (data, response) = try await URLSession.shared.data(from: url)
        guard let http = response as? HTTPURLResponse, 200..<300 ~= http.statusCode else {
            throw URLError(.badServerResponse)
        }
        return try JSONDecoder().decode(T.self, from: data)
    }
}
