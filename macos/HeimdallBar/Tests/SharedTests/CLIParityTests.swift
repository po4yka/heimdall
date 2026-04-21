import Foundation
import Testing
import HeimdallDomain
@testable import HeimdallCLI

struct CLIParityTests {
    @Test
    func usageParserAcceptsProviderSourceRefreshAndPrettyJSON() throws {
        let invocation = try CLIArgumentParser.parse(arguments: [
            "heimdallbar",
            "usage",
            "--provider", "codex",
            "--source", "web",
            "--status",
            "--refresh",
            "--pretty",
        ])

        #expect(invocation.command == .usage)
        #expect(invocation.helpTopic == nil)
        #expect(invocation.options.providers == [.codex])
        #expect(invocation.options.preferredSource == .web)
        #expect(invocation.options.includeStatus)
        #expect(invocation.options.refresh)
        #expect(invocation.options.format == .json)
        #expect(invocation.options.pretty)
    }

    @Test
    func configDumpRejectsStatusFlag() {
        #expect(throws: CLIArgumentError.self) {
            try CLIArgumentParser.parse(arguments: [
                "heimdallbar",
                "config",
                "dump",
                "--status",
            ])
        }
    }

    @Test
    func usageFormatterIncludesSourceStatusWarningsAndLanes() {
        let section = Self.makeSection()

        let text = CLITextFormatter.usageText(
            sections: [section],
            refresh: CLIRefreshMetadata(
                requestedRefresh: true,
                responseScope: "provider",
                requestedProvider: "codex",
                refreshedProviders: ["codex"],
                cacheHit: false,
                fetchedAt: "2026-04-20T10:20:00Z"
            ),
            includeStatus: true
        )

        #expect(text.contains("Usage snapshot: forced refresh"))
        #expect(text.contains("Requested source: web"))
        #expect(text.contains("Source: WEB"))
        #expect(text.contains("Fallbacks: web -> oauth"))
        #expect(text.contains("Login: browser session required for requested source"))
        #expect(text.contains("Warning: Login required to refresh web extras."))
        #expect(text.contains("Status: [degraded] OpenAI degraded"))
        #expect(text.contains("Lane: Session · 64% left · pace stable · resets in 18m"))
        #expect(text.contains("Credits: $12.40 credits"))
        #expect(text.contains("Today: $4.80 · 1234 tokens"))
    }

    @Test
    func costFormatterIncludesThirtyDaySummary() {
        let text = CLITextFormatter.costText(
            sections: [Self.makeSection()],
            refresh: nil,
            includeStatus: false
        )

        #expect(text.contains("30d: $48.20 · 5678 tokens"))
        #expect(text.contains("Refresh: Updated 2m ago"))
    }

    private static func makeSection() -> CLIProviderSection {
        let resolution = ProviderSourceResolution(
            provider: .codex,
            requestedSource: .web,
            effectiveSource: .web,
            effectiveSourceDetail: "WEB",
            sourceLabel: "Source: WEB",
            explanation: "Using imported dashboard session.",
            warnings: ["Login required to refresh web extras."],
            fallbackChain: ["web", "oauth"],
            usageAvailable: true,
            isUnsupported: false,
            requiresLogin: true,
            usesFallback: true
        )
        let presentation = ProviderPresentationState(
            provider: .codex,
            snapshot: ProviderSnapshot(
                provider: "codex",
                available: true,
                sourceUsed: "web",
                lastAttemptedSource: "web",
                resolvedViaFallback: true,
                refreshDurationMs: 120,
                sourceAttempts: [],
                identity: nil,
                primary: ProviderRateWindow(
                    usedPercent: 36,
                    resetsAt: nil,
                    resetsInMinutes: 18,
                    windowMinutes: 300,
                    resetLabel: "resets in 18m"
                ),
                secondary: nil,
                tertiary: nil,
                credits: 12.4,
                status: ProviderStatusSummary(
                    indicator: "degraded",
                    description: "OpenAI degraded",
                    pageURL: "https://status.openai.com"
                ),
                auth: ProviderAuthHealth(
                    loginMethod: "chatgpt",
                    credentialBackend: "file",
                    authMode: "chatgpt",
                    isAuthenticated: true,
                    isRefreshable: true,
                    isSourceCompatible: false,
                    requiresRelogin: false,
                    managedRestriction: nil,
                    diagnosticCode: "authenticated-incompatible-source",
                    failureReason: "Codex is authenticated with API key, so ChatGPT credits and web quota features are unavailable.",
                    lastValidatedAt: "2026-04-20T10:18:00Z",
                    recoveryActions: []
                ),
                costSummary: ProviderCostSummary(
                    todayTokens: 1234,
                    todayCostUSD: 4.8,
                    last30DaysTokens: 5678,
                    last30DaysCostUSD: 48.2,
                    daily: []
                ),
                claudeUsage: nil,
                lastRefresh: "2026-04-20T10:18:00Z",
                stale: false,
                error: nil
            ),
            adjunct: DashboardAdjunctSnapshot(
                provider: .codex,
                source: .web,
                headline: "Codex web extras",
                detailLines: [],
                webExtras: DashboardWebExtras(
                    signedInEmail: "user@example.com",
                    accountPlan: "Pro",
                    creditsRemaining: 12.4,
                    creditsPurchaseURL: nil,
                    quotaLanes: [],
                    sourceURL: nil,
                    fetchedAt: "2026-04-20T10:18:00Z"
                ),
                statusText: nil,
                isLoginRequired: true,
                lastUpdated: "2026-04-20T10:18:00Z"
            ),
            resolution: resolution
        )
        let projection = ProviderMenuProjection(
            provider: .codex,
            title: "Codex",
            sourceLabel: "Source: WEB",
            sourceExplanationLabel: "Using imported dashboard session.",
            authHeadline: "Authenticated, but incompatible with selected source",
            authDetail: "Codex is authenticated with API key, so ChatGPT credits and web quota features are unavailable.",
            authDiagnosticCode: "authenticated-incompatible-source",
            authSummaryLabel: "Chatgpt · File",
            authRecoveryActions: [],
            warningLabels: ["Login required to refresh web extras."],
            visualState: .degraded,
            stateLabel: "Degraded",
            statusLabel: "OpenAI degraded",
            identityLabel: nil,
            lastRefreshLabel: "Updated 2m ago",
            refreshStatusLabel: "Updated 2m ago",
            costLabel: "$4.80 today",
            laneDetails: [
                LaneDetailProjection(
                    title: "Session",
                    summary: "64% left",
                    remainingPercent: 64,
                    resetDetail: "resets in 18m",
                    paceLabel: "Stable"
                )
            ],
            creditsLabel: "$12.40 credits",
            incidentLabel: "OpenAI degraded",
            stale: false,
            isShowingCachedData: false,
            isRefreshing: false,
            error: nil,
            globalIssueLabel: nil,
            historyFractions: [],
            claudeFactors: [],
            adjunct: nil
        )
        return CLIProviderSection(
            provider: .codex,
            requestedSource: .web,
            presentation: presentation,
            projection: projection,
            costSummary: ProviderCostSummary(
                todayTokens: 1234,
                todayCostUSD: 4.8,
                last30DaysTokens: 5678,
                last30DaysCostUSD: 48.2,
                daily: []
            )
        )
    }
}
