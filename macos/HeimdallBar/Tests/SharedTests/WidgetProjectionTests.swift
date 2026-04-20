import Foundation
import Testing
@testable import HeimdallBarShared

struct WidgetProjectionTests {
    @Test
    func widgetSnapshotBuilderDerivesTypedProviderPayload() {
        let snapshot = ProviderSnapshot(
            provider: "codex",
            available: true,
            sourceUsed: "cli",
            lastAttemptedSource: "cli",
            resolvedViaFallback: false,
            refreshDurationMs: 150,
            sourceAttempts: [ProviderSourceAttempt(source: "cli", outcome: "success", message: nil)],
            identity: ProviderIdentity(provider: "codex", accountEmail: "user@example.com", accountOrganization: nil, loginMethod: "chatgpt", plan: "pro"),
            primary: ProviderRateWindow(usedPercent: 36, resetsAt: "2026-04-20T10:00:00Z", resetsInMinutes: 18, windowMinutes: 300, resetLabel: nil),
            secondary: ProviderRateWindow(usedPercent: 58, resetsAt: "2026-04-24T10:00:00Z", resetsInMinutes: 4000, windowMinutes: 10_080, resetLabel: nil),
            tertiary: nil,
            credits: 12.4,
            status: ProviderStatusSummary(indicator: "minor", description: "OpenAI degraded", pageURL: "https://status.openai.com"),
            auth: ProviderAuthHealth(
                loginMethod: "api-key",
                credentialBackend: "env",
                authMode: "api-key",
                isAuthenticated: true,
                isRefreshable: false,
                isSourceCompatible: false,
                requiresRelogin: false,
                managedRestriction: nil,
                diagnosticCode: "env-override",
                failureReason: "OPENAI_API_KEY is active.",
                lastValidatedAt: "2026-04-20T10:00:00Z",
                recoveryActions: []
            ),
            costSummary: ProviderCostSummary(
                todayTokens: 1234,
                todayCostUSD: 4.8,
                last30DaysTokens: 5678,
                last30DaysCostUSD: 48.2,
                daily: [
                    CostHistoryPoint(day: "2026-04-18", totalTokens: 200, costUSD: 1.2),
                    CostHistoryPoint(day: "2026-04-19", totalTokens: 300, costUSD: 2.4),
                    CostHistoryPoint(day: "2026-04-20", totalTokens: 734, costUSD: 4.8),
                ]
            ),
            claudeUsage: nil,
            lastRefresh: "2026-04-20T10:00:00Z",
            stale: false,
            error: nil
        )

        let typed = WidgetSnapshotBuilder.providerSnapshot(
            provider: .codex,
            config: ProviderConfig(enabled: true, source: .cli, cookieSource: .auto, dashboardExtrasEnabled: false),
            snapshot: snapshot,
            adjunct: nil
        )

        #expect(typed.provider == .codex)
        #expect(typed.source.requested == .cli)
        #expect(typed.source.effective == .cli)
        #expect(typed.lanes.count == 2)
        #expect(typed.lanes.first?.remainingPercent == 64)
        #expect(typed.auth?.diagnosticCode == "env-override")
        #expect(typed.identity?.accountEmail == nil)
        #expect(typed.cost.todayCostUSD == 4.8)
        #expect(typed.issues.contains(where: { $0.code == "auth-incompatible" }))
    }

    @Test
    func widgetSelectionSortsBySeverityAndUsesCadenceOverrides() {
        let snapshot = WidgetSnapshot(
            generatedAt: ISO8601DateFormatter().string(from: Date()),
            defaultRefreshIntervalSeconds: 1200,
            providers: [
                "claude": WidgetProviderSnapshot(
                    provider: .claude,
                    source: WidgetProviderSourceSnapshot(requested: .oauth, effective: .oauth, detail: nil, usesFallback: false, isUnsupported: false, usageAvailable: true),
                    freshness: WidgetProviderFreshnessSnapshot(visualState: .healthy, available: true, stale: false, lastRefreshAt: nil, error: nil, statusIndicator: nil, statusDescription: nil),
                    auth: WidgetProviderAuthSnapshot(loginMethod: "subscription-oauth", credentialBackend: "keychain", authMode: "subscription-oauth", isAuthenticated: true, isSourceCompatible: true, requiresRelogin: false, diagnosticCode: "authenticated-compatible", failureReason: nil, lastValidatedAt: nil),
                    identity: nil,
                    lanes: [],
                    credits: nil,
                    cost: WidgetProviderCostSnapshot(todayTokens: 200, todayCostUSD: 2.5, last30DaysTokens: 0, last30DaysCostUSD: 0, daily: []),
                    issues: [],
                    adjunct: nil
                ),
                "codex": WidgetProviderSnapshot(
                    provider: .codex,
                    source: WidgetProviderSourceSnapshot(requested: .web, effective: nil, detail: nil, usesFallback: false, isUnsupported: false, usageAvailable: false),
                    freshness: WidgetProviderFreshnessSnapshot(visualState: .incident, available: false, stale: true, lastRefreshAt: nil, error: nil, statusIndicator: "major", statusDescription: "Major outage"),
                    auth: WidgetProviderAuthSnapshot(loginMethod: "chatgpt", credentialBackend: "file", authMode: "chatgpt", isAuthenticated: true, isSourceCompatible: true, requiresRelogin: true, diagnosticCode: "requires-relogin", failureReason: "Login expired.", lastValidatedAt: nil),
                    identity: nil,
                    lanes: [],
                    credits: nil,
                    cost: WidgetProviderCostSnapshot(todayTokens: 100, todayCostUSD: 1.0, last30DaysTokens: 0, last30DaysCostUSD: 0, daily: []),
                    issues: [WidgetSnapshotIssue(code: "login-required", message: "Login expired.", severity: .warning)],
                    adjunct: WidgetProviderAdjunctSnapshot(source: .web, isLoginRequired: true, hasWebExtras: false, lastUpdatedAt: nil)
                ),
            ]
        )

        let ordered = WidgetSelection.orderedProviders(in: snapshot)
        #expect(ordered.first?.provider == .codex)
        #expect(WidgetSelection.cadenceSeconds(snapshot: snapshot, provider: .claude) == 1200)
        #expect(WidgetSelection.cadenceSeconds(snapshot: snapshot, provider: .codex) == 300)
        #expect(WidgetSelection.cadenceSeconds(snapshot: snapshot, provider: nil) == 300)
    }
}
