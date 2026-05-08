import Foundation
import Observation

// MARK: - Wire types (mirror Rust McpServerReport JSON)

public enum McpTransport: Codable, Sendable {
    case stdio(command: String, args: [String])
    case http(url: String)
    case sse(url: String)

    private enum CodingKeys: String, CodingKey { case kind, command, args, url }

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        switch try c.decode(String.self, forKey: .kind) {
        case "stdio":
            self = .stdio(
                command: try c.decode(String.self, forKey: .command),
                args: (try? c.decode([String].self, forKey: .args)) ?? []
            )
        case "http":
            self = .http(url: try c.decode(String.self, forKey: .url))
        case "sse":
            self = .sse(url: try c.decode(String.self, forKey: .url))
        default:
            self = .stdio(command: "", args: [])
        }
    }

    public func encode(to encoder: Encoder) throws {
        var c = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .stdio(let command, let args):
            try c.encode("stdio", forKey: .kind)
            try c.encode(command, forKey: .command)
            try c.encode(args, forKey: .args)
        case .http(let url):
            try c.encode("http", forKey: .kind)
            try c.encode(url, forKey: .url)
        case .sse(let url):
            try c.encode("sse", forKey: .kind)
            try c.encode(url, forKey: .url)
        }
    }
}

public enum McpRuntimeState: Codable, Sendable {
    case running(pid: Int, cpuPercent: Float, memoryBytes: UInt64)
    case notRunning
    case notApplicable

    private enum CodingKeys: String, CodingKey {
        case kind
        case pid
        case cpuPercent = "cpu_percent"
        case memoryBytes = "memory_bytes"
    }

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        switch try c.decode(String.self, forKey: .kind) {
        case "running":
            self = .running(
                pid: try c.decode(Int.self, forKey: .pid),
                cpuPercent: (try? c.decode(Float.self, forKey: .cpuPercent)) ?? 0,
                memoryBytes: (try? c.decode(UInt64.self, forKey: .memoryBytes)) ?? 0
            )
        case "not_running":
            self = .notRunning
        case "not_applicable":
            self = .notApplicable
        default:
            self = .notApplicable
        }
    }

    public func encode(to encoder: Encoder) throws {
        var c = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .running(let pid, let cpuPercent, let memoryBytes):
            try c.encode("running", forKey: .kind)
            try c.encode(pid, forKey: .pid)
            try c.encode(cpuPercent, forKey: .cpuPercent)
            try c.encode(memoryBytes, forKey: .memoryBytes)
        case .notRunning:
            try c.encode("not_running", forKey: .kind)
        case .notApplicable:
            try c.encode("not_applicable", forKey: .kind)
        }
    }
}

public enum McpRedactedValue: Codable, Sendable {
    case plain(String)
    case secret(String)
    case envFromFile(path: String, exists: Bool, bytes: UInt64)

    private enum CodingKeys: String, CodingKey {
        case kind, value, masked, path, exists, bytes
    }

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        switch try c.decode(String.self, forKey: .kind) {
        case "plain":
            self = .plain((try? c.decode(String.self, forKey: .value)) ?? "")
        case "secret":
            self = .secret((try? c.decode(String.self, forKey: .masked)) ?? "***")
        case "env_from_file":
            self = .envFromFile(
                path: (try? c.decode(String.self, forKey: .path)) ?? "",
                exists: (try? c.decode(Bool.self, forKey: .exists)) ?? false,
                bytes: (try? c.decode(UInt64.self, forKey: .bytes)) ?? 0
            )
        default:
            self = .plain("")
        }
    }

    public func encode(to encoder: Encoder) throws {
        var c = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .plain(let v):
            try c.encode("plain", forKey: .kind)
            try c.encode(v, forKey: .value)
        case .secret(let m):
            try c.encode("secret", forKey: .kind)
            try c.encode(m, forKey: .masked)
        case .envFromFile(let path, let exists, let bytes):
            try c.encode("env_from_file", forKey: .kind)
            try c.encode(path, forKey: .path)
            try c.encode(exists, forKey: .exists)
            try c.encode(bytes, forKey: .bytes)
        }
    }
}

public struct McpLogProbe: Codable, Sendable {
    public let path: String
    public let bytes: UInt64
    public let modified: String
    public let recentLineCount: Int

    enum CodingKeys: String, CodingKey {
        case path, bytes, modified
        case recentLineCount = "recent_line_count"
    }
}

public struct McpUsageStats: Codable, Sendable {
    public let totalCalls: UInt64
    public let lastUsed: String?
    public let distinctSessions: UInt64
    public let distinctTools: UInt64

    enum CodingKeys: String, CodingKey {
        case totalCalls = "total_calls"
        case lastUsed = "last_used"
        case distinctSessions = "distinct_sessions"
        case distinctTools = "distinct_tools"
    }
}

public struct McpServerEntry: Codable, Sendable, Identifiable {
    public let name: String
    public let provider: String
    public let scope: String
    public let projectLabel: String?
    public let sourcePath: String
    public let managedBy: String?
    public let transport: McpTransport
    public let env: [String: McpRedactedValue]
    public let runtime: McpRuntimeState
    public let logProbe: McpLogProbe?
    public let usage: McpUsageStats?
    public let isDormant: Bool

    public var id: String { "\(scope):\(sourcePath):\(name)" }

    enum CodingKeys: String, CodingKey {
        case name, provider, scope
        case projectLabel = "project_label"
        case sourcePath = "source_path"
        case managedBy = "managed_by"
        case transport, env, runtime
        case logProbe = "log_probe"
        case usage
        case isDormant = "is_dormant"
    }
}

public struct McpServerTotals: Codable, Sendable {
    public let configuredCount: Int
    public let runningCount: Int
    public let neverInvokedCount: Int
    public let claudeCount: Int
    public let codexCount: Int
    public let projectCount: Int
    public let dormantCount: Int

    enum CodingKeys: String, CodingKey {
        case configuredCount = "configured_count"
        case runningCount = "running_count"
        case neverInvokedCount = "never_invoked_count"
        case claudeCount = "claude_count"
        case codexCount = "codex_count"
        case projectCount = "project_count"
        case dormantCount = "dormant_count"
    }
}

public struct McpServerReport: Codable, Sendable {
    public let generatedAt: String
    public let claude: [McpServerEntry]
    public let codex: [McpServerEntry]
    public let totals: McpServerTotals

    enum CodingKeys: String, CodingKey {
        case generatedAt = "generated_at"
        case claude, codex, totals
    }
}

// MARK: - Feature model

@MainActor
@Observable
public final class McpServersFeatureModel {
    private let helperPort: Int

    public var report: McpServerReport?
    public var isLoading: Bool = false
    public var errorMessage: String?

    public init(helperPort: Int) {
        self.helperPort = helperPort
    }

    public func reload() async {
        guard !self.isLoading else { return }
        self.isLoading = true
        self.errorMessage = nil
        do {
            guard let url = URL(string: "http://127.0.0.1:\(self.helperPort)/api/mcp-servers") else {
                self.isLoading = false
                return
            }
            let (data, response) = try await URLSession.shared.data(from: url)
            guard let http = response as? HTTPURLResponse, (200..<300).contains(http.statusCode) else {
                throw URLError(.badServerResponse)
            }
            self.report = try JSONDecoder().decode(McpServerReport.self, from: data)
        } catch {
            self.errorMessage = error.localizedDescription
        }
        self.isLoading = false
    }
}
