import Foundation
import Testing
@testable import HeimdallDomain

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
            localNotificationsEnabled: false,
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
        #expect(projection.incidentLabel == "[CRITICAL] OpenAI critical incident")
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
            costLabel: "Today: $3.20 · 30 days: $21.00",
            todayCostUSD: 3.2,
            last30DaysCostUSD: 21,
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
            costLabel: "Today: $6.80 · 30 days: $42.00",
            todayCostUSD: 6.8,
            last30DaysCostUSD: 42,
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
        #expect(overview.combinedTodayCostUSD == 10)
        #expect(overview.activitySummaryLabel == "Codex accounts for 68% of today's spend")
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
        #expect(!projection.warningLabels.contains("Live refresh failed. Showing last known data."))
        #expect(projection.authHeadline == nil)
    }

    @Test
    func projectionCarriesQuotaSuggestionsFromSnapshot() {
        var snapshot = Self.codexSnapshot()
        snapshot.quotaSuggestions = QuotaSuggestions(
            sampleCount: 4,
            recommendedKey: "p90",
            levels: [
                QuotaSuggestionLevel(key: "p90", label: "P90", limitTokens: 800_000),
                QuotaSuggestionLevel(key: "p95", label: "P95", limitTokens: 900_000),
                QuotaSuggestionLevel(key: "max", label: "Max", limitTokens: 950_000),
            ],
            note: "Based on fewer than 10 completed blocks."
        )
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
            lastGlobalError: nil
        )

        #expect(projection.quotaSuggestions?.recommendedKey == "p90")
        #expect(projection.quotaSuggestions?.levels.count == 3)
    }

    @Test
    func projectionCarriesDepletionForecastFromSnapshot() {
        var snapshot = Self.codexSnapshot()
        snapshot.depletionForecast = DepletionForecast(
            primarySignal: DepletionForecastSignal(
                kind: "primary_window",
                title: "Primary window",
                usedPercent: 64,
                remainingPercent: 36,
                resetsInMinutes: 45,
                paceLabel: "Steady",
                endTime: "2026-04-23T10:45:00Z"
            ),
            secondarySignals: [],
            summaryLabel: "Primary window currently at 64% used",
            severity: "warn"
        )
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
            lastGlobalError: nil
        )

        #expect(projection.depletionForecast?.primarySignal.kind == "primary_window")
        #expect(projection.depletionForecast?.severity == "warn")
    }

    @Test
    func projectionCarriesPredictiveInsightsFromSnapshot() {
        var snapshot = Self.codexSnapshot()
        snapshot.predictiveInsights = LivePredictiveInsights(
            rollingHourBurn: LivePredictiveRollingHourBurn(
                tokensPerMin: 12_450,
                costPerHourNanos: 1_450_000_000,
                coverageMinutes: 42,
                tier: "moderate"
            ),
            historicalEnvelope: nil,
            limitHitAnalysis: LivePredictiveLimitHitAnalysis(
                sampleCount: 9,
                hitCount: 2,
                hitRate: 0.22,
                thresholdTokens: 900_000,
                thresholdPercent: 90,
                activeCurrentHit: false,
                activeProjectedHit: true,
                riskLevel: "medium",
                summaryLabel: "Projected to hit the suggested quota in 2 of the last 9 windows."
            )
        )
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
            lastGlobalError: nil
        )

        #expect(projection.predictiveInsights?.rollingHourBurn?.tier == "moderate")
        #expect(projection.predictiveInsights?.limitHitAnalysis?.activeProjectedHit == true)
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
            localNotificationsEnabled: HeimdallBarConfig.default.localNotificationsEnabled,
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

    @Test
    func transientThrottleErrorsAreClassifiedAsDegradedNotError() {
        #expect(MenuProjectionBuilder.isTransientThrottleError("Anthropic API rate-limited us — retrying on next poll.") == true)
        #expect(MenuProjectionBuilder.isTransientThrottleError("API returned HTTP 429: {...}") == false)
        #expect(MenuProjectionBuilder.isTransientThrottleError("OAuth token expired. Run `claude login` to refresh.") == false)
        #expect(MenuProjectionBuilder.isTransientThrottleError("") == false)
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
