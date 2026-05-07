import Foundation
import HeimdallDomain
import HeimdallServices
import Observation

@MainActor
@Observable
public final class CostModelsFeatureModel {
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

    /// Merged model rows across all providers, sorted by cost.
    public var byModel: [ProviderModelRow] {
        var byName: [String: ProviderModelRow] = [:]
        for item in self.overview.projection.items {
            for row in item.byModel {
                if let existing = byName[row.model] {
                    byName[row.model] = ProviderModelRow(
                        model: row.model,
                        costUSD: existing.costUSD + row.costUSD,
                        input: existing.input + row.input,
                        output: existing.output + row.output,
                        cacheRead: existing.cacheRead + row.cacheRead,
                        cacheCreation: existing.cacheCreation + row.cacheCreation,
                        reasoningOutput: existing.reasoningOutput + row.reasoningOutput,
                        turns: existing.turns + row.turns
                    )
                } else {
                    byName[row.model] = row
                }
            }
        }
        return byName.values.sorted { $0.costUSD > $1.costUSD }
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

    /// Merged MCP server rows across all providers, sorted by invocations.
    public var byMcp: [ProviderMcpRow] {
        var byServer: [String: ProviderMcpRow] = [:]
        for item in self.overview.projection.items {
            for row in item.byMcp {
                if let existing = byServer[row.server] {
                    byServer[row.server] = ProviderMcpRow(
                        server: row.server,
                        invocations: existing.invocations + row.invocations,
                        toolsUsed: existing.toolsUsed + row.toolsUsed,
                        sessionsUsed: existing.sessionsUsed + row.sessionsUsed
                    )
                } else {
                    byServer[row.server] = row
                }
            }
        }
        return byServer.values.sorted { $0.invocations > $1.invocations }
    }

    /// Merged CLI version breakdown across all providers, sorted by cost.
    public var versionBreakdown: [ProviderVersionRow] {
        var byVersion: [String: ProviderVersionRow] = [:]
        for item in self.overview.projection.items {
            for row in item.versionBreakdown {
                if let existing = byVersion[row.version] {
                    byVersion[row.version] = ProviderVersionRow(
                        version: row.version,
                        turns: existing.turns + row.turns,
                        sessions: existing.sessions + row.sessions,
                        costUSD: existing.costUSD + row.costUSD
                    )
                } else {
                    byVersion[row.version] = row
                }
            }
        }
        return byVersion.values.sorted { $0.costUSD > $1.costUSD }
    }

    public func refreshAll() async {
        await self.overview.refreshAll()
    }
}
