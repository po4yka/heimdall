import Foundation
import Observation

// MARK: - Wire types (partial decode of /api/data)

public enum ContextPressureBucket: String, Codable, Sendable {
    case healthy = "healthy"
    case warm = "warm"
    case tight = "tight"
    case overCompacted = "over_compacted"
}

public struct ContextPressureRow: Codable, Sendable, Identifiable {
    public let sessionId: String
    public let project: String?
    public let model: String
    public let startedAt: String
    public let turnCount: UInt32
    public let peakInputTokens: UInt64
    public let contextWindowSize: UInt64
    public let peakFraction: Float
    public let compactionCount: UInt32
    public let bucket: ContextPressureBucket

    public var id: String { sessionId }

    enum CodingKeys: String, CodingKey {
        case sessionId = "session_id"
        case project, model
        case startedAt = "started_at"
        case turnCount = "turn_count"
        case peakInputTokens = "peak_input_tokens"
        case contextWindowSize = "context_window_size"
        case peakFraction = "peak_fraction"
        case compactionCount = "compaction_count"
        case bucket
    }
}

public struct ContextPressureSummary: Codable, Sendable {
    public let rows: [ContextPressureRow]
    public let healthyCount: UInt32
    public let warmCount: UInt32
    public let tightCount: UInt32
    public let overcompactedCount: UInt32
    public let avgPeakFraction: Float

    enum CodingKeys: String, CodingKey {
        case rows
        case healthyCount = "healthy_count"
        case warmCount = "warm_count"
        case tightCount = "tight_count"
        case overcompactedCount = "overcompacted_count"
        case avgPeakFraction = "avg_peak_fraction"
    }
}

public struct AgentTreeNode: Codable, Sendable, Identifiable {
    public let agentId: String?
    public let role: String?
    public let turnCount: UInt32
    public let inputTokens: UInt64
    public let outputTokens: UInt64
    public let cacheReadTokens: UInt64
    public let estimatedCostNanos: Int64
    public let children: [AgentTreeNode]

    public var id: String { agentId ?? "root" }

    enum CodingKeys: String, CodingKey {
        case agentId = "agent_id"
        case role
        case turnCount = "turn_count"
        case inputTokens = "input_tokens"
        case outputTokens = "output_tokens"
        case cacheReadTokens = "cache_read_tokens"
        case estimatedCostNanos = "estimated_cost_nanos"
        case children
    }
}

public struct AgentRoleCost: Codable, Sendable, Identifiable {
    public let role: String
    public let costNanos: Int64
    public var id: String { role }

    public init(from decoder: Decoder) throws {
        var c = try decoder.unkeyedContainer()
        self.role = try c.decode(String.self)
        self.costNanos = try c.decode(Int64.self)
    }

    public func encode(to encoder: Encoder) throws {
        var c = encoder.unkeyedContainer()
        try c.encode(self.role)
        try c.encode(self.costNanos)
    }
}

public struct SessionAgentTree: Codable, Sendable, Identifiable {
    public let sessionId: String
    public let project: String?
    public let root: AgentTreeNode
    public let totalCostNanos: Int64
    public let subagentCount: UInt32

    public var id: String { sessionId }

    enum CodingKeys: String, CodingKey {
        case sessionId = "session_id"
        case project, root
        case totalCostNanos = "total_cost_nanos"
        case subagentCount = "subagent_count"
    }
}

public struct AgentTreeSummary: Codable, Sendable {
    public let sessions: [SessionAgentTree]
    public let topSubagentRoles: [AgentRoleCost]

    enum CodingKeys: String, CodingKey {
        case sessions
        case topSubagentRoles = "top_subagent_roles"
    }
}

// MARK: - Partial dashboard decode

private struct DashboardDataPartial: Codable {
    let contextPressure: ContextPressureSummary?
    let agentTree: AgentTreeSummary?

    enum CodingKeys: String, CodingKey {
        case contextPressure = "context_pressure"
        case agentTree = "agent_tree"
    }
}

// MARK: - Feature model

@MainActor
@Observable
public final class DashboardDataFeatureModel {
    private let helperPort: Int

    public var contextPressure: ContextPressureSummary?
    public var agentTree: AgentTreeSummary?
    public var isLoading: Bool = false

    public init(helperPort: Int) {
        self.helperPort = helperPort
    }

    public func reload() async {
        guard !self.isLoading else { return }
        self.isLoading = true
        defer { self.isLoading = false }
        do {
            guard let url = URL(string: "http://127.0.0.1:\(self.helperPort)/api/data") else { return }
            let (data, response) = try await URLSession.shared.data(from: url)
            guard let http = response as? HTTPURLResponse, (200..<300).contains(http.statusCode) else { return }
            let partial = try JSONDecoder().decode(DashboardDataPartial.self, from: data)
            self.contextPressure = partial.contextPressure
            self.agentTree = partial.agentTree
        } catch {
            // supplemental sections — silently ignore fetch failures
        }
    }
}
