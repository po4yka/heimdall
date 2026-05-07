import Foundation
import HeimdallDomain
import HeimdallServices
import Observation

@MainActor
@Observable
public final class SessionsFeatureModel {
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

    /// All recent sessions across providers, most recent first.
    public var recentSessions: [ProviderSession] {
        self.overview.projection.items
            .flatMap { $0.recentSessions }
            .sorted { $0.startedAt > $1.startedAt }
    }

    /// Merged per-model cost rows across providers, sorted by cost descending.
    public var byModel: [ProviderModelRow] {
        var byKey: [String: ProviderModelRow] = [:]
        for item in self.overview.projection.items {
            for row in item.byModel {
                if let existing = byKey[row.model] {
                    byKey[row.model] = ProviderModelRow(
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
                    byKey[row.model] = row
                }
            }
        }
        return byKey.values.sorted { $0.costUSD > $1.costUSD }
    }

    /// Merged per-project cost rows across providers, sorted by cost descending.
    public var byProject: [ProviderProjectRow] {
        var byKey: [String: ProviderProjectRow] = [:]
        for item in self.overview.projection.items {
            for row in item.byProject {
                if let existing = byKey[row.project] {
                    byKey[row.project] = ProviderProjectRow(
                        project: row.project,
                        displayName: row.displayName,
                        costUSD: existing.costUSD + row.costUSD,
                        turns: existing.turns + row.turns,
                        sessions: existing.sessions + row.sessions
                    )
                } else {
                    byKey[row.project] = row
                }
            }
        }
        return byKey.values.sorted { $0.costUSD > $1.costUSD }
    }

    public func refreshAll() async {
        await self.overview.refreshAll()
    }
}
