import Foundation
import Testing
@testable import HeimdallBarShared

struct WidgetProjectionTests {
    @Test
    func widgetProjectionCarriesUsageWarningsAndHistory() {
        let projection = ProviderMenuProjection(
            provider: .codex,
            title: "Codex",
            sourceLabel: "WEB",
            sourceExplanationLabel: "Using imported dashboard session.",
            authHeadline: "Authenticated, but incompatible with selected source",
            authDetail: "Browser session is missing for requested web source.",
            authDiagnosticCode: "authenticated-incompatible-source",
            authSummaryLabel: "Chatgpt · File",
            authRecoveryActions: [],
            warningLabels: ["Login required to refresh web extras."],
            visualState: .incident,
            stateLabel: "Incident",
            statusLabel: "OpenAI degraded",
            identityLabel: "user@example.com",
            lastRefreshLabel: "Updated 2m ago",
            refreshStatusLabel: "Refresh overdue",
            costLabel: "$4.80 today",
            laneDetails: [
                LaneDetailProjection(
                    title: "Session",
                    summary: "64% remaining",
                    remainingPercent: 64,
                    resetDetail: "resets in 18m",
                    paceLabel: "Stable"
                ),
                LaneDetailProjection(
                    title: "Weekly",
                    summary: "42% remaining",
                    remainingPercent: 42,
                    resetDetail: "resets tomorrow",
                    paceLabel: "Fast"
                ),
            ],
            creditsLabel: "$12.40 credits",
            incidentLabel: "OpenAI degraded",
            stale: false,
            isRefreshing: false,
            error: nil,
            historyFractions: [0.1, 0.5, 0.8],
            claudeFactors: [],
            adjunct: nil
        )
        let costSummary = ProviderCostSummary(
            todayTokens: 1234,
            todayCostUSD: 4.8,
            last30DaysTokens: 5678,
            last30DaysCostUSD: 48.2,
            daily: []
        )

        let entry = WidgetProjectionBuilder.entry(from: projection, costSummary: costSummary)

        #expect(entry.provider == .codex)
        #expect(entry.visualState == .incident)
        #expect(entry.loginRequired)
        #expect(entry.warningLabel == "Login required to refresh web extras.")
        #expect(entry.usageLines.count == 2)
        #expect(entry.usageLines.first?.valueLabel == "64%")
        #expect(entry.usageLines.first?.detailLabel == "pace stable · resets in 18m")
        #expect(entry.historyFractions == [0.1, 0.5, 0.8])
        #expect(entry.todayCostLabel == "$4.80 today")
        #expect(entry.last30DaysCostLabel == "$48.20 in 30d")
        #expect(entry.todayTokensLabel == "1234 tokens today")
    }

    @Test
    func widgetSelectionReturnsRequestedProviderAndCadenceOverrides() {
        let claude = WidgetProviderEntry(
            provider: .claude,
            title: "Claude",
            visualState: .healthy,
            statusLabel: "Healthy",
            refreshLabel: "Updated just now",
            usageLines: [],
            creditsLabel: nil,
            warningLabel: nil,
            unavailableLabel: nil,
            loginRequired: false,
            historyFractions: [],
            costSummary: ProviderCostSummary(todayTokens: 0, todayCostUSD: 0, last30DaysTokens: 0, last30DaysCostUSD: 0, daily: []),
            todayCostLabel: "$0.00 today",
            last30DaysCostLabel: "$0.00 in 30d",
            todayTokensLabel: "Tokens unavailable",
            activityLabel: "No recent activity",
            sourceLabel: "OAUTH",
            updatedAt: "Updated just now"
        )
        let codex = WidgetProviderEntry(
            provider: .codex,
            title: "Codex",
            visualState: .error,
            statusLabel: "Error",
            refreshLabel: "Login required",
            usageLines: [],
            creditsLabel: nil,
            warningLabel: "Login required.",
            unavailableLabel: "No usable provider data.",
            loginRequired: true,
            historyFractions: [],
            costSummary: ProviderCostSummary(todayTokens: 0, todayCostUSD: 0, last30DaysTokens: 0, last30DaysCostUSD: 0, daily: []),
            todayCostLabel: "$0.00 today",
            last30DaysCostLabel: "$0.00 in 30d",
            todayTokensLabel: "Tokens unavailable",
            activityLabel: "No recent activity",
            sourceLabel: "WEB",
            updatedAt: "Updated 10m ago"
        )
        let snapshot = WidgetSnapshot(
            generatedAt: ISO8601DateFormatter().string(from: Date()),
            refreshIntervalSeconds: 1200,
            entries: [claude, codex]
        )

        #expect(WidgetSelection.providerEntry(in: snapshot, provider: .codex)?.provider == .codex)
        #expect(WidgetSelection.cadenceSeconds(snapshot: snapshot, provider: .claude) == 1200)
        #expect(WidgetSelection.cadenceSeconds(snapshot: snapshot, provider: .codex) == 300)
    }
}
