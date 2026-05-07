import Foundation
import HeimdallDomain
import HeimdallServices
import Observation

@MainActor
@Observable
public final class ActivityFeatureModel {
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

    /// Merged daily costs across all providers, summed by day.
    public var dailyCosts: [CostHistoryPoint] {
        var byDay: [String: CostHistoryPoint] = [:]
        for item in self.overview.projection.items {
            for point in item.dailyCosts {
                if let existing = byDay[point.day] {
                    let merged = existing.breakdown.flatMap { a in
                        point.breakdown.map { a.merging($0) }
                    } ?? existing.breakdown ?? point.breakdown
                    byDay[point.day] = CostHistoryPoint(
                        day: point.day,
                        totalTokens: existing.totalTokens + point.totalTokens,
                        costUSD: existing.costUSD + point.costUSD,
                        breakdown: merged
                    )
                } else {
                    byDay[point.day] = point
                }
            }
        }
        return byDay.values.sorted { $0.day < $1.day }
    }

    /// Merged token history breakdowns aligned by index.
    public var historyBreakdowns: [TokenBreakdown] {
        let all = self.overview.projection.items.map { $0.historyBreakdowns }
        guard let maxLen = all.map(\.count).max(), maxLen > 0 else { return [] }
        return (0..<maxLen).map { i in
            let slices = all.compactMap { $0.indices.contains(i) ? $0[i] : nil }
            return slices.dropFirst().reduce(slices[0]) { $0.merging($1) }
        }
    }

    /// Combined model rows from all providers, merged by model name.
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

    /// Combined project rows, merged by project name.
    public var byProject: [ProviderProjectRow] {
        var byName: [String: ProviderProjectRow] = [:]
        for item in self.overview.projection.items {
            for row in item.byProject {
                if let existing = byName[row.project] {
                    byName[row.project] = ProviderProjectRow(
                        project: row.project,
                        displayName: row.displayName,
                        costUSD: existing.costUSD + row.costUSD,
                        turns: existing.turns + row.turns,
                        sessions: existing.sessions + row.sessions
                    )
                } else {
                    byName[row.project] = row
                }
            }
        }
        return byName.values.sorted { $0.costUSD > $1.costUSD }
    }

    /// Merged hourly activity across all providers.
    public var hourlyActivity: [ProviderHourlyBucket] {
        var byHour: [Int: ProviderHourlyBucket] = [:]
        for item in self.overview.projection.items {
            for bucket in item.hourlyActivity {
                if let existing = byHour[bucket.hour] {
                    byHour[bucket.hour] = ProviderHourlyBucket(
                        hour: bucket.hour,
                        turns: existing.turns + bucket.turns,
                        costUSD: existing.costUSD + bucket.costUSD,
                        tokens: existing.tokens + bucket.tokens
                    )
                } else {
                    byHour[bucket.hour] = bucket
                }
            }
        }
        return byHour.values.sorted { $0.hour < $1.hour }
    }

    /// Merged activity heatmap cells across all providers.
    public var activityHeatmap: [ProviderHeatmapCell] {
        var byKey: [String: ProviderHeatmapCell] = [:]
        for item in self.overview.projection.items {
            for cell in item.activityHeatmap {
                let key = "\(cell.dayOfWeek)-\(cell.hour)"
                if let existing = byKey[key] {
                    byKey[key] = ProviderHeatmapCell(
                        dayOfWeek: cell.dayOfWeek,
                        hour: cell.hour,
                        turns: existing.turns + cell.turns
                    )
                } else {
                    byKey[key] = cell
                }
            }
        }
        return Array(byKey.values)
    }

    public func refreshAll() async {
        await self.overview.refreshAll()
    }
}
