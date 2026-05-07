import Foundation
import HeimdallDomain
import HeimdallServices
import Observation

@MainActor
@Observable
public final class AgentsFeatureModel {
    private let overview: OverviewFeatureModel

    public init(overview: OverviewFeatureModel) {
        self.overview = overview
    }

    public var isRefreshing: Bool {
        self.overview.projection.isRefreshing
    }

    public var globalIssueLabel: String? {
        self.overview.projection.globalIssueLabel
    }

    public var providerItems: [ProviderMenuProjection] {
        self.overview.projection.items
    }

    /// Merged subagent breakdown across all providers.
    public var subagentBreakdown: ProviderSubagentBreakdown? {
        let all = self.overview.projection.items.compactMap { $0.subagentBreakdown }
        guard !all.isEmpty else { return nil }
        return all.dropFirst().reduce(all[0]) { acc, next in
            ProviderSubagentBreakdown(
                totalTurns: acc.totalTurns + next.totalTurns,
                totalCostUSD: acc.totalCostUSD + next.totalCostUSD,
                sessionCount: acc.sessionCount + next.sessionCount,
                agentCount: acc.agentCount + next.agentCount
            )
        }
    }

    public var hasAgentData: Bool {
        self.subagentBreakdown != nil
    }

    /// Merged tool usage across all providers, sorted by invocations.
    public var byTool: [ProviderToolRow] {
        var byKey: [String: ProviderToolRow] = [:]
        for item in self.overview.projection.items {
            for row in item.byTool {
                let key = "\(row.mcpServer ?? "_")/\(row.toolName)"
                if let existing = byKey[key] {
                    byKey[key] = ProviderToolRow(
                        toolName: row.toolName,
                        category: row.category,
                        mcpServer: row.mcpServer,
                        invocations: existing.invocations + row.invocations,
                        errors: existing.errors + row.errors,
                        turnsUsed: existing.turnsUsed + row.turnsUsed,
                        sessionsUsed: existing.sessionsUsed + row.sessionsUsed
                    )
                } else {
                    byKey[key] = row
                }
            }
        }
        return byKey.values.sorted { $0.invocations > $1.invocations }
    }

    /// Merged recent sessions across all providers, most recent first.
    public var recentSessions: [ProviderSession] {
        self.overview.projection.items
            .flatMap { $0.recentSessions }
            .sorted { $0.startedAt > $1.startedAt }
    }

    public func refreshAll() async {
        await self.overview.refreshAll()
    }
}
