import Foundation
import Testing
@testable import HeimdallBarShared

struct MenuProjectionBuilderTests {
    @Test
    func projectionBuildsIncidentStateAndResolutionWarnings() {
        let config = HeimdallBarConfig(
            claude: ProviderConfig(enabled: true, source: .oauth, cookieSource: .auto, dashboardExtrasEnabled: false),
            codex: ProviderConfig(enabled: true, source: .oauth, cookieSource: .auto, dashboardExtrasEnabled: true),
            mergeIcons: false,
            showUsedValues: false,
            refreshIntervalSeconds: 120,
            resetDisplayMode: .countdown,
            checkProviderStatus: true,
            helperPort: 8787
        )
        let presentation = ProviderPresentationState(
            provider: .codex,
            snapshot: Self.codexSnapshot(),
            adjunct: nil,
            resolution: ProviderSourceResolution(
                provider: .codex,
                requestedSource: .oauth,
                effectiveSource: .cli,
                effectiveSourceDetail: "cli-rpc",
                sourceLabel: "Source: requested oauth",
                explanation: "The requested oauth source is not the active live snapshot.",
                warnings: ["Requested oauth, but Heimdall resolved cli-rpc."],
                fallbackChain: ["oauth", "cli-rpc"],
                usageAvailable: true,
                isUnsupported: false,
                requiresLogin: false,
                usesFallback: true
            )
        )

        let projection = MenuProjectionBuilder.projection(
            from: presentation,
            config: config,
            isRefreshing: false,
            lastGlobalError: nil
        )

        #expect(projection.visualState == .incident)
        #expect(projection.stateLabel == "Incident")
        #expect(projection.incidentLabel?.contains("CRITICAL") == true)
        #expect(projection.warningLabels.contains("Requested oauth, but Heimdall resolved cli-rpc."))
        #expect(projection.warningLabels.contains("Resolution chain: oauth -> cli-rpc"))
        #expect(projection.laneDetails.first?.summary.contains("64% left") == true)
        #expect(projection.refreshStatusLabel.contains("Incident active"))
        #expect(projection.isShowingCachedData == false)
    }

    @Test
    func overviewAggregatesCombinedCostHistoryAndWarnings() {
        let first = ProviderMenuProjection(
            provider: .claude,
            title: "Claude",
            sourceLabel: "Source: oauth",
            sourceExplanationLabel: nil,
            authHeadline: nil,
            authDetail: nil,
            authDiagnosticCode: nil,
            authSummaryLabel: nil,
            authRecoveryActions: [],
            warningLabels: ["Shared warning"],
            visualState: .healthy,
            stateLabel: "Operational",
            statusLabel: nil,
            identityLabel: nil,
            lastRefreshLabel: "Last refresh: 2m ago",
            refreshStatusLabel: "Last refresh: 2m ago",
            costLabel: "Today: $3.20 · 30d: $21.00",
            laneDetails: [],
            creditsLabel: nil,
            incidentLabel: nil,
            stale: false,
            isShowingCachedData: false,
            isRefreshing: false,
            error: nil,
            globalIssueLabel: nil,
            historyFractions: [0.2, 0.8],
            claudeFactors: [],
            adjunct: nil
        )
        let second = ProviderMenuProjection(
            provider: .codex,
            title: "Codex",
            sourceLabel: "Source: web",
            sourceExplanationLabel: nil,
            authHeadline: "Authenticated, but incompatible with selected source",
            authDetail: "Current auth cannot satisfy web source",
            authDiagnosticCode: "authenticated-incompatible-source",
            authSummaryLabel: "Api Key · Env",
            authRecoveryActions: [],
            warningLabels: ["Shared warning", "Codex login required"],
            visualState: .degraded,
            stateLabel: "Degraded",
            statusLabel: "[DEGRADED] OpenAI degraded",
            identityLabel: nil,
            lastRefreshLabel: "Last refresh: 4m ago",
            refreshStatusLabel: "Provider degraded · last refresh: 4m ago",
            costLabel: "Today: $6.80 · 30d: $42.00",
            laneDetails: [],
            creditsLabel: nil,
            incidentLabel: nil,
            stale: false,
            isShowingCachedData: false,
            isRefreshing: false,
            error: nil,
            globalIssueLabel: nil,
            historyFractions: [0.6, 0.4],
            claudeFactors: [],
            adjunct: nil
        )

        let overview = MenuProjectionBuilder.overview(
            from: [first, second],
            isRefreshing: false,
            lastGlobalError: nil
        )

        #expect(overview.combinedCostLabel == "Combined today: $10.00")
        #expect(overview.activitySummaryLabel == "Most active: Codex · degraded")
        #expect(overview.warningLabels == ["Shared warning", "Codex login required"])
        #expect(overview.historyFractions == [0.4, 0.6000000000000001])
        #expect(overview.refreshStatusLabel == "Last refresh: 2m ago")
        #expect(overview.isShowingCachedData == false)
        #expect(overview.globalIssueLabel == nil)
    }

    @Test
    func projectionShowsCachedDataInsteadOfProviderErrorWhenGlobalRefreshFails() {
        var snapshot = Self.codexSnapshot()
        snapshot.status = nil
        let presentation = ProviderPresentationState(
            provider: .codex,
            snapshot: snapshot,
            adjunct: nil,
            resolution: ProviderSourceResolution(
                provider: .codex,
                requestedSource: .auto,
                effectiveSource: .cli,
                effectiveSourceDetail: "cli-rpc",
                sourceLabel: "Source: auto",
                explanation: "Using the latest successful live snapshot.",
                warnings: [],
                fallbackChain: ["auto", "cli-rpc"],
                usageAvailable: true,
                isUnsupported: false,
                requiresLogin: false,
                usesFallback: false
            )
        )

        let projection = MenuProjectionBuilder.projection(
            from: presentation,
            config: HeimdallBarConfig.default,
            isRefreshing: false,
            lastGlobalError: "Could not connect to the server."
        )

        #expect(projection.visualState == .stale)
        #expect(projection.stateLabel == "Stale")
        #expect(projection.isShowingCachedData == true)
        #expect(projection.error == nil)
        #expect(projection.refreshStatusLabel == "Showing cached data")
        #expect(projection.globalIssueLabel == "Cannot reach the local Heimdall server.")
        #expect(projection.warningLabels.contains("Live refresh failed. Showing last known data."))
        #expect(projection.authHeadline == nil)
    }

    @Test
    func menuTitleHonorsShowUsedValues() {
        let leftConfig = HeimdallBarConfig.default
        let usedConfig = HeimdallBarConfig(
            claude: HeimdallBarConfig.default.claude,
            codex: HeimdallBarConfig.default.codex,
            mergeIcons: HeimdallBarConfig.default.mergeIcons,
            showUsedValues: true,
            refreshIntervalSeconds: HeimdallBarConfig.default.refreshIntervalSeconds,
            resetDisplayMode: HeimdallBarConfig.default.resetDisplayMode,
            checkProviderStatus: HeimdallBarConfig.default.checkProviderStatus,
            helperPort: HeimdallBarConfig.default.helperPort
        )
        let presentation = ProviderPresentationState(
            provider: .codex,
            snapshot: Self.codexSnapshot(),
            adjunct: nil,
            resolution: ProviderSourceResolution(
                provider: .codex,
                requestedSource: .oauth,
                effectiveSource: .oauth,
                effectiveSourceDetail: "oauth",
                sourceLabel: "Source: oauth",
                explanation: "Using oauth.",
                warnings: [],
                fallbackChain: ["oauth"],
                usageAvailable: true,
                isUnsupported: false,
                requiresLogin: false,
                usesFallback: false
            )
        )

        #expect(MenuProjectionBuilder.menuTitle(for: presentation, provider: .codex, config: leftConfig) == "Codex 64% left")
        #expect(MenuProjectionBuilder.menuTitle(for: presentation, provider: .codex, config: usedConfig) == "Codex 36% used")
    }

    private static func codexSnapshot() -> ProviderSnapshot {
        ProviderSnapshot(
            provider: "codex",
            available: true,
            sourceUsed: "cli-rpc",
            lastAttemptedSource: "oauth",
            resolvedViaFallback: true,
            refreshDurationMs: 90,
            sourceAttempts: [],
            identity: ProviderIdentity(
                provider: "codex",
                accountEmail: "fixture@example.com",
                accountOrganization: nil,
                loginMethod: "chatgpt",
                plan: "team"
            ),
            primary: ProviderRateWindow(
                usedPercent: 36,
                resetsAt: nil,
                resetsInMinutes: 18,
                windowMinutes: 300,
                resetLabel: "resets in 18m"
            ),
            secondary: ProviderRateWindow(
                usedPercent: 59,
                resetsAt: nil,
                resetsInMinutes: 1440,
                windowMinutes: 10_080,
                resetLabel: "resets in 1440m"
            ),
            tertiary: nil,
            credits: 11.9,
            status: ProviderStatusSummary(
                indicator: "critical",
                description: "OpenAI critical incident",
                pageURL: "https://status.openai.com"
            ),
            auth: ProviderAuthHealth(
                loginMethod: "chatgpt",
                credentialBackend: "file",
                authMode: "chatgpt",
                isAuthenticated: true,
                isRefreshable: true,
                isSourceCompatible: true,
                requiresRelogin: false,
                managedRestriction: nil,
                diagnosticCode: "authenticated-compatible",
                failureReason: nil,
                lastValidatedAt: nil,
                recoveryActions: []
            ),
            costSummary: ProviderCostSummary(
                todayTokens: 1500,
                todayCostUSD: 6.8,
                last30DaysTokens: 12_000,
                last30DaysCostUSD: 42.0,
                daily: [
                    CostHistoryPoint(day: "2026-04-18", totalTokens: 700, costUSD: 2.4),
                    CostHistoryPoint(day: "2026-04-19", totalTokens: 1500, costUSD: 6.8),
                ]
            ),
            claudeUsage: nil,
            lastRefresh: ISO8601DateFormatter().string(from: Date().addingTimeInterval(-120)),
            stale: false,
            error: nil
        )
    }
}
