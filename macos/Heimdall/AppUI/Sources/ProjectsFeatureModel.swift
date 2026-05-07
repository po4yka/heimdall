import Foundation
import HeimdallDomain
import HeimdallServices
import Observation

public enum ProjectSortOrder: String, CaseIterable, Sendable {
    case cost = "Cost"
    case turns = "Turns"
    case sessions = "Sessions"
    case name = "Name"
}

@MainActor
@Observable
public final class ProjectsFeatureModel {
    private let overview: OverviewFeatureModel

    public var sortOrder: ProjectSortOrder = .cost

    public init(overview: OverviewFeatureModel) {
        self.overview = overview
    }

    public var isRefreshing: Bool {
        self.overview.projection.isRefreshing
    }

    public var globalIssueLabel: String? {
        self.overview.projection.globalIssueLabel
    }

    /// Merged per-project rows across providers, ordered by `sortOrder`.
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
        let rows = byKey.values
        switch self.sortOrder {
        case .cost:     return rows.sorted { $0.costUSD > $1.costUSD }
        case .turns:    return rows.sorted { $0.turns > $1.turns }
        case .sessions: return rows.sorted { $0.sessions > $1.sessions }
        case .name:     return rows.sorted { $0.displayName < $1.displayName }
        }
    }

    public var totalCostUSD: Double {
        self.byProject.reduce(0) { $0 + $1.costUSD }
    }

    public var totalTurns: Int {
        self.byProject.reduce(0) { $0 + $1.turns }
    }

    public var totalSessions: Int {
        self.byProject.reduce(0) { $0 + $1.sessions }
    }

    public func refreshAll() async {
        await self.overview.refreshAll()
    }
}
