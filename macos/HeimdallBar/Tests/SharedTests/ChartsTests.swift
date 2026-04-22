import Foundation
import Testing
@testable import HeimdallAppUI
@testable import HeimdallDomain

struct ChartsTests {
    @Test
    func historyBarChartEntriesIndexInOrderAndMarkToday() {
        let entries = HistoryBarChart.entries(from: [0.1, 0.3, -0.2, 1.4, 0.5, 0.0, 0.75])
        #expect(entries.count == 7)
        let indices = entries.map(\.index)
        #expect(indices == [0, 1, 2, 3, 4, 5, 6])
        #expect(entries.last?.label == "Today")
        let clampedLow = entries[2].fraction
        let clampedHigh = entries[3].fraction
        #expect(clampedLow == 0.0)
        #expect(clampedHigh == 1.0)
    }

    @Test
    func historyBarChartEntriesEmptyWhenNoFractions() {
        #expect(HistoryBarChart.entries(from: []).isEmpty)
    }

    @Test
    func tokenStackChartEntriesEmitOnePerNonZeroCategoryInStableOrder() {
        let breakdowns = [
            TokenBreakdown(input: 10, output: 0, cacheRead: 5, cacheCreation: 0, reasoningOutput: 0),
            TokenBreakdown(input: 0, output: 0, cacheRead: 0, cacheCreation: 0, reasoningOutput: 0),
            TokenBreakdown(input: 1, output: 2, cacheRead: 3, cacheCreation: 4, reasoningOutput: 5),
        ]
        let entries = TokenStackChart.entries(from: breakdowns)

        // Day 0: 2 categories (input, cacheRead). Day 1: 0. Day 2: 5.
        #expect(entries.count == 2 + 0 + 5)

        let day0 = entries.filter { $0.dayIndex == 0 }
        #expect(day0.map(\.category) == [.input, .cacheRead])
        #expect(day0.map(\.tokens) == [10, 5])

        let day2 = entries.filter { $0.dayIndex == 2 }
        #expect(day2.map(\.category) == TokenCategory.orderedForStack)
        #expect(day2.map(\.tokens) == [1, 2, 3, 4, 5])

        #expect(entries.last?.dayLabel == "Today")
    }

    @Test
    func tokenStackChartEntriesHandleEmptyInput() {
        #expect(TokenStackChart.entries(from: []).isEmpty)
    }

    @Test
    func tokenBreakdownDonutEntriesAndPercentLabelsStayStable() {
        let breakdown = TokenBreakdown(
            input: 50,
            output: 40,
            cacheRead: 9,
            cacheCreation: 1,
            reasoningOutput: 0
        )
        let entries = TokenBreakdownDonut.entries(from: breakdown)

        #expect(entries.map(\.category) == [.input, .output, .cacheRead, .cacheCreation])
        #expect(entries.map(\.tokens) == [50, 40, 9, 1])

        #expect(TokenBreakdownDonut.percentLabel(for: TokenBreakdownDonut.share(for: 50, total: 100)) == "50%")
        #expect(TokenBreakdownDonut.percentLabel(for: TokenBreakdownDonut.share(for: 9, total: 100)) == "9.0%")
        #expect(TokenBreakdownDonut.percentLabel(for: TokenBreakdownDonut.share(for: 1, total: 200)) == "<1%")
    }

    @Test
    func cacheMixRingDeltaLabelsHandlePositiveNegativeAndFlat() {
        #expect(CacheMixRing.deltaLabel(today: 0.95, baseline: 0.941) == "+0.9 pt")
        #expect(CacheMixRing.deltaLabel(today: 0.95, baseline: 0.96) == "-1.0 pt")
        #expect(CacheMixRing.deltaLabel(today: 0.9502, baseline: 0.95) == "Flat")
    }

    @Test
    func chartStyleSnapThresholdScalesWithDensityButStaysBounded() {
        #expect(ChartStyle.snapThreshold(plotWidth: 320, itemCount: 7) == 22.857142857142858)
        #expect(ChartStyle.snapThreshold(plotWidth: 320, itemCount: 30) == 12)
        #expect(ChartStyle.snapThreshold(plotWidth: 1_000, itemCount: 6) == 28)
    }

    @Test
    func chartStyleInspectorPlacementBiasesAwayFromEdges() {
        #expect(ChartStyle.inspectorPlacement(index: 0, totalCount: 7) == .trailing)
        #expect(ChartStyle.inspectorPlacement(index: 3, totalCount: 7) == .top)
        #expect(ChartStyle.inspectorPlacement(index: 6, totalCount: 7) == .leading)
    }

    @Test
    func dailyCostChartEntriesParseIsoDaysAndPassCostsThrough() {
        let daily = [
            CostHistoryPoint(day: "2026-04-18", totalTokens: 0, costUSD: 2.5),
            CostHistoryPoint(day: "2026-04-19", totalTokens: 0, costUSD: 3.75),
            CostHistoryPoint(day: "not-a-date", totalTokens: 0, costUSD: 9.99),
            CostHistoryPoint(day: "2026-04-20", totalTokens: 0, costUSD: 1.0),
        ]
        let entries = DailyCostChart.entries(from: daily)
        let count = entries.count
        #expect(count == 3)
        let costs = entries.map(\.costUSD)
        #expect(costs == [2.5, 3.75, 1.0])
    }

    @Test
    func dailyCostChartEntriesEmptyOnEmptyInput() {
        let empty: [CostHistoryPoint] = []
        let result = DailyCostChart.entries(from: empty)
        #expect(result.isEmpty)
    }

    @Test
    func dailyCostChartEntriesSkipUnparseableDays() {
        let daily = [
            CostHistoryPoint(day: "not-a-date", totalTokens: 0, costUSD: 1.0),
            CostHistoryPoint(day: "2026-04-20", totalTokens: 0, costUSD: 2.0),
        ]
        let entries = DailyCostChart.entries(from: daily)
        let count = entries.count
        let cost = entries.first?.costUSD
        #expect(count == 1)
        #expect(cost == 2.0)
    }

    @Test
    func providerComparisonChartTotalEntriesRollUpSameDayAcrossProviders() {
        let entries = ProviderComparisonChart.entries(from: [
            self.makeProviderProjection(
                title: "Claude",
                provider: .claude,
                last30DaysCostUSD: 12,
                dailyCosts: [
                    CostHistoryPoint(day: "2026-04-18", totalTokens: 0, costUSD: 4.0),
                    CostHistoryPoint(day: "2026-04-19", totalTokens: 0, costUSD: 6.0),
                ]
            ),
            self.makeProviderProjection(
                title: "Codex",
                provider: .codex,
                last30DaysCostUSD: 8,
                dailyCosts: [
                    CostHistoryPoint(day: "2026-04-18", totalTokens: 0, costUSD: 1.5),
                    CostHistoryPoint(day: "2026-04-19", totalTokens: 0, costUSD: 2.5),
                ]
            ),
        ])
        let totals = ProviderComparisonChart.totalEntries(from: entries)

        #expect(totals.count == 2)
        #expect(totals.map(\.costUSD) == [5.5, 8.5])
    }

    @Test
    func providerComparisonChartSummariesSortByCostAndComputeShare() {
        let summaries = ProviderComparisonChart.providerSummaries(from: [
            self.makeProviderProjection(
                title: "Codex",
                provider: .codex,
                last30DaysCostUSD: 80,
                dailyCosts: [CostHistoryPoint(day: "2026-04-20", totalTokens: 0, costUSD: 5)]
            ),
            self.makeProviderProjection(
                title: "Claude",
                provider: .claude,
                last30DaysCostUSD: 320,
                dailyCosts: [CostHistoryPoint(day: "2026-04-20", totalTokens: 0, costUSD: 12)]
            ),
        ])

        #expect(summaries.map(\.title) == ["Claude", "Codex"])
        #expect(summaries.map(\.costUSD) == [320, 80])
        #expect(abs(summaries[0].share - 0.8) < 0.0001)
        #expect(abs(summaries[1].share - 0.2) < 0.0001)
    }

    @Test
    func providerComparisonChartAverageDailyCostReturnsNilWithoutUsage() {
        #expect(ProviderComparisonChart.averageDailyCost(totalCostUSD: 0, activeDays: 30) == nil)
        #expect(ProviderComparisonChart.averageDailyCost(totalCostUSD: 10, activeDays: 0) == nil)
        #expect(ProviderComparisonChart.averageDailyCost(totalCostUSD: 90, activeDays: 30) == 3)
    }

    @Test
    func activityHeatmapSummaryCapturesPeakAndActiveCells() {
        let summary = ActivityHeatmap.summary(from: [
            [0, 2, 0, 0],
            [1, 0, 7, 0],
            [0, 0, 0, 3],
        ])

        #expect(summary != nil)
        #expect(summary?.totalTurns == 13)
        #expect(summary?.activeCells == 4)
        #expect(summary?.peakTurns == 7)
        #expect(summary?.peakDay == 1)
        #expect(summary?.peakHour == 2)
    }

    @Test
    func historyBarChartTooltipIncludesDayLabelsAndPercentages() {
        let entries = HistoryBarChart.entries(from: [0.25, 1.0])
        let tooltip = HistoryBarChart.tooltip(for: entries)

        #expect(tooltip.contains("Today: 100% of peak"))
        #expect(tooltip.contains("25% of peak"))
    }

    @Test
    func providerComparisonChartTooltipIncludesDailyTotalsAndProviders() {
        let day = ProviderComparisonChart.dayFormatter.date(from: "2026-04-20")!
        let tooltip = ProviderComparisonChart.tooltip(
            entries: [
                .init(day: day, providerTitle: "Claude", costUSD: 12.5),
                .init(day: day, providerTitle: "Codex", costUSD: 1.25),
            ],
            totals: [
                .init(day: day, costUSD: 13.75),
            ]
        )

        #expect(tooltip.contains("Claude"))
        #expect(tooltip.contains("Codex"))
        #expect(tooltip.contains("$13.75"))
    }

    private func makeProviderProjection(
        title: String,
        provider: ProviderID,
        last30DaysCostUSD: Double,
        dailyCosts: [CostHistoryPoint]
    ) -> ProviderMenuProjection {
        ProviderMenuProjection(
            provider: provider,
            title: title,
            sourceLabel: "",
            sourceExplanationLabel: nil,
            authHeadline: nil,
            authDetail: nil,
            authDiagnosticCode: nil,
            authSummaryLabel: nil,
            authRecoveryActions: [],
            warningLabels: [],
            visualState: .healthy,
            stateLabel: "",
            statusLabel: nil,
            identityLabel: nil,
            lastRefreshLabel: "",
            refreshStatusLabel: "",
            costLabel: "",
            todayCostUSD: nil,
            last30DaysCostUSD: last30DaysCostUSD,
            laneDetails: [],
            creditsLabel: nil,
            incidentLabel: nil,
            stale: false,
            isShowingCachedData: false,
            isRefreshing: false,
            error: nil,
            globalIssueLabel: nil,
            historyFractions: [],
            claudeFactors: [],
            adjunct: nil,
            dailyCosts: dailyCosts
        )
    }
}
