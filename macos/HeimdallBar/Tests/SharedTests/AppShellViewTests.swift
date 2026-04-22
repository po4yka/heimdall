import Foundation
import Testing
import HeimdallDomain
@testable import HeimdallAppUI

struct AppShellViewTests {
    @Test
    func windowProviderMetricSummaryUsesLeftQualifierForRemainingMode() {
        let summary = WindowProviderMetricSummary.make(
            item: self.makeProjection(
                laneDetails: [
                    LaneDetailProjection(
                        title: "Session",
                        summary: "64% left",
                        remainingPercent: 64,
                        resetDetail: "resets in 18m",
                        paceLabel: "Stable"
                    )
                ]
            ),
            showUsedValues: false
        )

        #expect(summary == WindowProviderMetricSummary(
            title: "Session remaining",
            value: "64%",
            qualifier: "LEFT",
            detail: "resets in 18m"
        ))
    }

    @Test
    func windowProviderMetricSummaryUsesUsedQualifierForUsedMode() {
        let summary = WindowProviderMetricSummary.make(
            item: self.makeProjection(
                laneDetails: [
                    LaneDetailProjection(
                        title: "Session",
                        summary: "64% left",
                        remainingPercent: 64,
                        resetDetail: "resets in 18m",
                        paceLabel: "Stable"
                    )
                ]
            ),
            showUsedValues: true
        )

        #expect(summary == WindowProviderMetricSummary(
            title: "Session usage",
            value: "36%",
            qualifier: "USED",
            detail: "resets in 18m"
        ))
    }

    @Test
    func windowProviderMetricSummaryUsesUnavailableLabelWhenQuotaIsMissing() {
        let summary = WindowProviderMetricSummary.make(
            item: self.makeProjection(
                laneDetails: [],
                sourceLabel: "Source: oauth"
            ),
            showUsedValues: false
        )

        #expect(summary == WindowProviderMetricSummary(
            title: "Session availability",
            value: "Unavailable",
            qualifier: "LIVE QUOTA",
            detail: "OAuth session data is unavailable"
        ))
    }

    private func makeProjection(
        laneDetails: [LaneDetailProjection],
        sourceLabel: String = "Source: cli",
        isShowingCachedData: Bool = false
    ) -> ProviderMenuProjection {
        ProviderMenuProjection(
            provider: .codex,
            title: "Codex",
            sourceLabel: sourceLabel,
            sourceExplanationLabel: nil,
            authHeadline: nil,
            authDetail: nil,
            authDiagnosticCode: nil,
            authSummaryLabel: nil,
            authRecoveryActions: [],
            warningLabels: [],
            visualState: .healthy,
            stateLabel: "Operational",
            statusLabel: nil,
            identityLabel: nil,
            lastRefreshLabel: "Last refresh: 2m ago",
            refreshStatusLabel: "Last refresh: 2m ago",
            costLabel: "Today: $6.80 · 30d: $42.00",
            laneDetails: laneDetails,
            creditsLabel: nil,
            incidentLabel: nil,
            stale: false,
            isShowingCachedData: isShowingCachedData,
            isRefreshing: false,
            error: nil,
            globalIssueLabel: nil,
            historyFractions: [],
            claudeFactors: [],
            adjunct: nil
        )
    }
}
