import Foundation
import Testing
@testable import HeimdallBarShared

struct SourceResolverTests {
    @Test
    func autoUsesWebExtrasWhenLiveSnapshotIsMissing() {
        let config = ProviderConfig(
            enabled: true,
            source: .auto,
            cookieSource: .auto,
            dashboardExtrasEnabled: true
        )
        let resolution = SourceResolver.resolve(
            provider: .codex,
            config: config,
            snapshot: nil,
            adjunct: Self.codexAdjunct(loginRequired: false, withLanes: true)
        )

        #expect(resolution.effectiveSource == UsageSourcePreference.web)
        #expect(resolution.usageAvailable)
        #expect(!resolution.requiresLogin)
        #expect(resolution.explanation.contains("cached web dashboard quotas"))
    }

    @Test
    func webSourceStaysUnavailableWhenDashboardExtrasAreDisabled() {
        let config = ProviderConfig(
            enabled: true,
            source: .web,
            cookieSource: .auto,
            dashboardExtrasEnabled: false
        )
        let resolution = SourceResolver.resolve(
            provider: .codex,
            config: config,
            snapshot: nil,
            adjunct: nil
        )

        #expect(resolution.effectiveSource == nil)
        #expect(!resolution.usageAvailable)
        #expect(!resolution.requiresLogin)
        #expect(resolution.warnings.contains("Enable dashboard extras before selecting the web source."))
    }

    @Test
    func requestedOAuthRejectsFallbackCliSnapshot() {
        let config = ProviderConfig(
            enabled: true,
            source: .oauth,
            cookieSource: .auto,
            dashboardExtrasEnabled: true
        )
        let resolution = SourceResolver.resolve(
            provider: .codex,
            config: config,
            snapshot: Self.codexSnapshot(sourceUsed: "cli-rpc", resolvedViaFallback: true),
            adjunct: nil
        )

        #expect(resolution.effectiveSource == UsageSourcePreference.cli)
        #expect(!resolution.usageAvailable)
        #expect(!resolution.isUnsupported)
        #expect(resolution.usesFallback)
        #expect(resolution.warnings.contains(where: { $0.contains("Requested oauth, but Heimdall resolved cli-rpc.") }))
        #expect(resolution.warnings.contains(where: { $0.contains("helper fell back") }))
    }

    @Test
    func claudeMarksCliAsUnsupported() {
        let config = ProviderConfig(
            enabled: true,
            source: .cli,
            cookieSource: .auto,
            dashboardExtrasEnabled: false
        )
        let resolution = SourceResolver.resolve(
            provider: .claude,
            config: config,
            snapshot: Self.codexSnapshot(sourceUsed: "cli-rpc", resolvedViaFallback: false),
            adjunct: nil
        )

        #expect(resolution.isUnsupported)
        #expect(!resolution.usageAvailable)
        #expect(resolution.warnings.contains("Requested cli, but Claude has no matching live source."))
    }

    private static func codexSnapshot(sourceUsed: String, resolvedViaFallback: Bool) -> ProviderSnapshot {
        ProviderSnapshot(
            provider: "codex",
            available: true,
            sourceUsed: sourceUsed,
            lastAttemptedSource: "oauth",
            resolvedViaFallback: resolvedViaFallback,
            refreshDurationMs: 140,
            sourceAttempts: [
                ProviderSourceAttempt(source: "oauth", outcome: "error", message: "expired"),
                ProviderSourceAttempt(source: sourceUsed, outcome: "success", message: nil),
            ],
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
            credits: 11.9,
            status: nil,
            costSummary: ProviderCostSummary(
                todayTokens: 1200,
                todayCostUSD: 3.8,
                last30DaysTokens: 9000,
                last30DaysCostUSD: 31.4,
                daily: []
            ),
            claudeUsage: nil,
            lastRefresh: ISO8601DateFormatter().string(from: Date()),
            stale: false,
            error: nil
        )
    }

    private static func codexAdjunct(loginRequired: Bool, withLanes: Bool) -> DashboardAdjunctSnapshot {
        DashboardAdjunctSnapshot(
            provider: .codex,
            source: .web,
            headline: "Codex web extras",
            detailLines: [],
            webExtras: DashboardWebExtras(
                signedInEmail: "fixture@example.com",
                accountPlan: "Pro",
                creditsRemaining: 14.75,
                creditsPurchaseURL: nil,
                quotaLanes: withLanes ? [
                    DashboardWebQuotaLane(
                        title: "Session",
                        window: ProviderRateWindow(
                            usedPercent: 36,
                            resetsAt: nil,
                            resetsInMinutes: 18,
                            windowMinutes: 300,
                            resetLabel: "resets in 18m"
                        )
                    )
                ] : [],
                sourceURL: "https://chatgpt.com/codex/cloud/settings/analytics#usage",
                fetchedAt: ISO8601DateFormatter().string(from: Date())
            ),
            statusText: "ready",
            isLoginRequired: loginRequired,
            lastUpdated: ISO8601DateFormatter().string(from: Date())
        )
    }
}
