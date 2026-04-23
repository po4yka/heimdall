import Foundation
import Testing
@testable import HeimdallDomain

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
            auth: ProviderAuthHealth(
                loginMethod: "chatgpt",
                credentialBackend: "file",
                authMode: "chatgpt",
                isAuthenticated: true,
                isRefreshable: true,
                isSourceCompatible: sourceUsed != "cli-rpc",
                requiresRelogin: false,
                managedRestriction: nil,
                diagnosticCode: sourceUsed == "cli-rpc" ? "authenticated-incompatible-source" : "authenticated-compatible",
                failureReason: sourceUsed == "cli-rpc" ? "Current auth does not satisfy the requested oauth source." : nil,
                lastValidatedAt: nil,
                recoveryActions: []
            ),
            costSummary: ProviderCostSummary(
                todayTokens: 1200,
                todayCostUSD: 3.8,
                last30DaysTokens: 9000,
                last30DaysCostUSD: 31.4,
                daily: []
            ),
            claudeUsage: nil,
            claudeAdmin: nil,
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

    @Test
    func requestedOAuthAcceptsAdminFallbackForClaude() {
        let config = ProviderConfig(
            enabled: true,
            source: .oauth,
            cookieSource: .auto,
            dashboardExtrasEnabled: false
        )
        let snapshot = ProviderSnapshot(
            provider: "claude",
            available: true,
            sourceUsed: "admin",
            lastAttemptedSource: "oauth",
            resolvedViaFallback: true,
            refreshDurationMs: 88,
            sourceAttempts: [
                ProviderSourceAttempt(source: "oauth", outcome: "unavailable", message: "oauth unavailable"),
                ProviderSourceAttempt(source: "admin", outcome: "success", message: "using admin fallback"),
            ],
            identity: nil,
            primary: nil,
            secondary: nil,
            tertiary: nil,
            credits: nil,
            status: nil,
            auth: ProviderAuthHealth(
                loginMethod: nil,
                credentialBackend: nil,
                authMode: nil,
                isAuthenticated: false,
                isRefreshable: false,
                isSourceCompatible: true,
                requiresRelogin: false,
                managedRestriction: nil,
                diagnosticCode: nil,
                failureReason: nil,
                lastValidatedAt: nil,
                recoveryActions: []
            ),
            costSummary: ProviderCostSummary(
                todayTokens: 1200,
                todayCostUSD: 3.8,
                last30DaysTokens: 9000,
                last30DaysCostUSD: 31.4,
                daily: []
            ),
            claudeUsage: nil,
            claudeAdmin: ClaudeAdminSummaryPayload(
                organizationName: "Acme Org",
                lookbackDays: 30,
                startDate: "2026-03-21",
                endDate: "2026-04-19",
                dataLatencyNote: "Org-wide · UTC daily aggregation · up to 1 hour delayed",
                todayActiveUsers: 9,
                todaySessions: 21,
                lookbackLinesAccepted: 3000,
                lookbackEstimatedCostUSD: 44.5,
                lookbackInputTokens: 1,
                lookbackOutputTokens: 2,
                lookbackCacheReadTokens: 3,
                lookbackCacheCreationTokens: 4,
                error: nil
            ),
            lastRefresh: ISO8601DateFormatter().string(from: Date()),
            stale: false,
            error: nil
        )

        let resolution = SourceResolver.resolve(
            provider: .claude,
            config: config,
            snapshot: snapshot,
            adjunct: nil
        )

        #expect(resolution.effectiveSource == .oauth)
        #expect(resolution.effectiveSourceDetail == "admin")
        #expect(resolution.usageAvailable)
        #expect(resolution.warnings.contains(where: { $0.contains("Anthropic admin analytics fallback") }))
    }
}
