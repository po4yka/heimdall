import Foundation
import HeimdallDomain
import Observation

@MainActor
@Observable
public final class ToolErrorsFeatureModel {
    public var rows: [ToolErrorRow] = []
    public var total: Int = 0
    public var isLoading: Bool = false
    public var errorMessage: String?

    public let toolName: String
    private let port: Int

    public init(toolName: String, port: Int) {
        self.toolName = toolName
        self.port = port
    }

    public func load() async {
        guard !self.isLoading else { return }
        self.isLoading = true
        self.errorMessage = nil
        do {
            let encoded = self.toolName.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? self.toolName
            guard let url = URL(string: "http://127.0.0.1:\(self.port)/api/tool-errors?tool=\(encoded)&limit=200") else { return }
            let (data, response) = try await URLSession.shared.data(from: url)
            guard let http = response as? HTTPURLResponse, 200..<300 ~= http.statusCode else {
                throw URLError(.badServerResponse)
            }
            let decoded = try JSONDecoder().decode(ToolErrorsResponse.self, from: data)
            self.rows = decoded.rows
            self.total = decoded.total
        } catch {
            self.errorMessage = error.localizedDescription
        }
        self.isLoading = false
    }
}
