import Foundation
import HeimdallDomain
import HeimdallServices
import Observation

@MainActor
@Observable
public final class CostModelsFeatureModel {
    private let overview: OverviewFeatureModel
    @ObservationIgnored private var derivedCache: CostModelsDerivedCache?

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
        self.derivedProjection.byModel
    }

    /// Merged tool usage across all providers, sorted by invocations.
    public var byTool: [ProviderToolRow] {
        self.derivedProjection.byTool
    }

    /// Merged MCP server rows across all providers, sorted by invocations.
    public var byMcp: [ProviderMcpRow] {
        self.derivedProjection.byMcp
    }

    /// Merged CLI version breakdown across all providers, sorted by cost.
    public var versionBreakdown: [ProviderVersionRow] {
        self.derivedProjection.versionBreakdown
    }

    private var derivedProjection: CostModelsDerivedProjection {
        let items = self.overview.projection.items
        let signature = CostModelsDerivedSignature(items: items)
        if let cache = self.derivedCache, cache.signature == signature {
            return cache.projection
        }
        let projection = CostModelsDerivedProjection(items: items)
        self.derivedCache = CostModelsDerivedCache(signature: signature, projection: projection)
        return projection
    }

    public func refreshAll() async {
        await self.overview.refreshAll()
    }
}

private struct CostModelsDerivedCache {
    var signature: CostModelsDerivedSignature
    var projection: CostModelsDerivedProjection
}

private struct CostModelsDerivedProjection {
    var byModel: [ProviderModelRow]
    var byTool: [ProviderToolRow]
    var byMcp: [ProviderMcpRow]
    var versionBreakdown: [ProviderVersionRow]

    init(items: [ProviderMenuProjection]) {
        self.byModel = Self.mergeByModel(items)
        self.byTool = Self.mergeByTool(items)
        self.byMcp = Self.mergeByMcp(items)
        self.versionBreakdown = Self.mergeVersionBreakdown(items)
    }

    private static func mergeByModel(_ items: [ProviderMenuProjection]) -> [ProviderModelRow] {
        var byName: [String: ProviderModelRow] = [:]
        for item in items {
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

    private static func mergeByTool(_ items: [ProviderMenuProjection]) -> [ProviderToolRow] {
        var byKey: [String: ProviderToolRow] = [:]
        for item in items {
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

    private static func mergeByMcp(_ items: [ProviderMenuProjection]) -> [ProviderMcpRow] {
        var byServer: [String: ProviderMcpRow] = [:]
        for item in items {
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

    private static func mergeVersionBreakdown(_ items: [ProviderMenuProjection]) -> [ProviderVersionRow] {
        var byVersion: [String: ProviderVersionRow] = [:]
        for item in items {
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
}

private struct CostModelsDerivedSignature: Equatable {
    var providers: [CostModelsProviderSignature]

    init(items: [ProviderMenuProjection]) {
        self.providers = items.map(CostModelsProviderSignature.init(item:))
    }
}

private struct CostModelsProviderSignature: Equatable {
    var provider: ProviderID
    var revision: String

    init(item: ProviderMenuProjection) {
        self.provider = item.provider
        self.revision = item.derivedDataRevision
    }
}
