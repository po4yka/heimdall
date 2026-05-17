import Foundation
import HeimdallDomain
import HeimdallServices
import Observation

@MainActor
@Observable
public final class ActivityFeatureModel {
    private let overview: OverviewFeatureModel
    @ObservationIgnored private var derivedCache: ActivityDerivedCache?

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
        self.derivedProjection.dailyCosts
    }

    /// Merged token history breakdowns aligned by index.
    public var historyBreakdowns: [TokenBreakdown] {
        self.derivedProjection.historyBreakdowns
    }

    /// Combined model rows from all providers, merged by model name.
    public var byModel: [ProviderModelRow] {
        self.derivedProjection.byModel
    }

    /// Combined project rows, merged by project name.
    public var byProject: [ProviderProjectRow] {
        self.derivedProjection.byProject
    }

    /// Merged hourly activity across all providers.
    public var hourlyActivity: [ProviderHourlyBucket] {
        self.derivedProjection.hourlyActivity
    }

    /// Merged activity heatmap cells across all providers.
    public var activityHeatmap: [ProviderHeatmapCell] {
        self.derivedProjection.activityHeatmap
    }

    private var derivedProjection: ActivityDerivedProjection {
        let items = self.overview.projection.items
        let signature = ActivityDerivedSignature(items: items)
        if let cache = self.derivedCache, cache.signature == signature {
            return cache.projection
        }
        let projection = ActivityDerivedProjection(items: items)
        self.derivedCache = ActivityDerivedCache(signature: signature, projection: projection)
        return projection
    }

    public func refreshAll() async {
        await self.overview.refreshAll()
    }
}

private struct ActivityDerivedCache {
    var signature: ActivityDerivedSignature
    var projection: ActivityDerivedProjection
}

private struct ActivityDerivedProjection {
    var dailyCosts: [CostHistoryPoint]
    var historyBreakdowns: [TokenBreakdown]
    var byModel: [ProviderModelRow]
    var byProject: [ProviderProjectRow]
    var hourlyActivity: [ProviderHourlyBucket]
    var activityHeatmap: [ProviderHeatmapCell]

    init(items: [ProviderMenuProjection]) {
        self.dailyCosts = Self.mergeDailyCosts(items)
        self.historyBreakdowns = Self.mergeHistoryBreakdowns(items)
        self.byModel = Self.mergeByModel(items)
        self.byProject = Self.mergeByProject(items)
        self.hourlyActivity = Self.mergeHourlyActivity(items)
        self.activityHeatmap = Self.mergeActivityHeatmap(items)
    }

    private static func mergeDailyCosts(_ items: [ProviderMenuProjection]) -> [CostHistoryPoint] {
        var byDay: [String: CostHistoryPoint] = [:]
        for item in items {
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

    private static func mergeHistoryBreakdowns(_ items: [ProviderMenuProjection]) -> [TokenBreakdown] {
        let all = items.map { $0.historyBreakdowns }
        guard let maxLen = all.map(\.count).max(), maxLen > 0 else { return [] }
        return (0..<maxLen).map { i in
            let slices = all.compactMap { $0.indices.contains(i) ? $0[i] : nil }
            return slices.dropFirst().reduce(slices[0]) { $0.merging($1) }
        }
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

    private static func mergeByProject(_ items: [ProviderMenuProjection]) -> [ProviderProjectRow] {
        var byName: [String: ProviderProjectRow] = [:]
        for item in items {
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

    private static func mergeHourlyActivity(_ items: [ProviderMenuProjection]) -> [ProviderHourlyBucket] {
        var byHour: [Int: ProviderHourlyBucket] = [:]
        for item in items {
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

    private static func mergeActivityHeatmap(_ items: [ProviderMenuProjection]) -> [ProviderHeatmapCell] {
        var byKey: [String: ProviderHeatmapCell] = [:]
        for item in items {
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
}

private struct ActivityDerivedSignature: Equatable {
    var providers: [ActivityProviderSignature]

    init(items: [ProviderMenuProjection]) {
        self.providers = items.map(ActivityProviderSignature.init(item:))
    }
}

private struct ActivityProviderSignature: Equatable {
    var provider: String
    var dailyCosts: [ActivityDailyCostSignature]
    var historyBreakdowns: [TokenBreakdown]
    var byModel: [ProviderModelRow]
    var byProject: [ProviderProjectRow]
    var hourlyActivity: [ProviderHourlyBucket]
    var activityHeatmap: [ProviderHeatmapCell]

    init(item: ProviderMenuProjection) {
        self.provider = item.provider.rawValue
        self.dailyCosts = item.dailyCosts.map(ActivityDailyCostSignature.init(point:))
        self.historyBreakdowns = item.historyBreakdowns
        self.byModel = item.byModel
        self.byProject = item.byProject
        self.hourlyActivity = item.hourlyActivity
        self.activityHeatmap = item.activityHeatmap
    }
}

private struct ActivityDailyCostSignature: Equatable {
    var day: String
    var totalTokens: Int
    var costUSD: Double
    var breakdown: TokenBreakdown?

    init(point: CostHistoryPoint) {
        self.day = point.day
        self.totalTokens = point.totalTokens
        self.costUSD = point.costUSD
        self.breakdown = point.breakdown
    }
}
