import Foundation

public struct ToolErrorRow: Codable, Sendable, Identifiable {
    public let timestamp: String
    public let sessionId: String
    public let project: String
    public let model: String
    public let provider: String
    public let toolName: String
    public let mcpServer: String?
    public let toolInput: String?
    public let errorText: String?
    public let sourcePath: String

    public var id: String { "\(self.sessionId)-\(self.timestamp)-\(self.toolName)" }

    enum CodingKeys: String, CodingKey {
        case timestamp
        case sessionId = "session_id"
        case project
        case model
        case provider
        case toolName = "tool_name"
        case mcpServer = "mcp_server"
        case toolInput = "tool_input"
        case errorText = "error_text"
        case sourcePath = "source_path"
    }
}

public struct ToolErrorsResponse: Codable, Sendable {
    public let rows: [ToolErrorRow]
    public let total: Int
}
