import Foundation
import HeimdallDomain
import HeimdallServices
@testable import HeimdallMobileApp
import Testing

@MainActor
struct MobileDashboardModelTests {
    @Test
    func loadHandlesMissingSnapshot() async {
        let model = MobileDashboardModel(store: StubSnapshotStore(snapshot: nil))

        await model.load()

        #expect(model.snapshot == nil)
        #expect(model.lastError == nil)
    }

    @Test
    func loadSelectsFirstAvailableProviderWhenDefaultIsMissing() async {
        let model = MobileDashboardModel(
            store: StubSnapshotStore(
                snapshot: MobileSnapshotEnvelope.fixture(
                    providers: [.fixture(provider: .codex)],
                    staleProviders: ["codex"]
                )
            )
        )

        await model.load()

        #expect(model.selectedProvider == .codex)
        #expect(model.selectedProviderSnapshot?.providerID == .codex)
        #expect(model.snapshot?.freshness.hasStaleProviders == true)
    }

    @Test
    func loadPreservesMultipleProvidersAndHistory() async {
        let model = MobileDashboardModel(store: StubSnapshotStore(snapshot: .fixture()))

        await model.load()

        #expect(model.providerSnapshots.count == 2)
        #expect(model.selectedHistorySeries?.daily.count == 2)
    }
}

private actor StubSnapshotStore: SnapshotSyncStore {
    let snapshot: MobileSnapshotEnvelope?

    func loadLatestSnapshot() async throws -> MobileSnapshotEnvelope? {
        self.snapshot
    }

    func saveLatestSnapshot(_: MobileSnapshotEnvelope) async throws {}
}

private extension MobileSnapshotEnvelope {
    static func fixture(
        providers: [ProviderSnapshot] = [.fixture(provider: .claude), .fixture(provider: .codex)],
        staleProviders: [String] = []
    ) -> MobileSnapshotEnvelope {
        MobileSnapshotEnvelope(
            generatedAt: "2026-04-21T11:00:00Z",
            sourceDevice: "macbook-air",
            providers: providers,
            history90d: providers.map { provider in
                MobileProviderHistorySeries(
                    provider: provider.provider,
                    daily: [
                        CostHistoryPoint(day: "2026-04-20", totalTokens: 1000, costUSD: 10.0),
                        CostHistoryPoint(day: "2026-04-21", totalTokens: 800, costUSD: 8.0),
                    ],
                    totalTokens: 1800,
                    totalCostUSD: 18.0
                )
            },
            totals: MobileSnapshotTotals(
                todayTokens: 1800,
                todayCostUSD: 18.0,
                last90DaysTokens: 3600,
                last90DaysCostUSD: 36.0
            ),
            freshness: MobileSnapshotFreshness(
                newestProviderRefresh: "2026-04-21T11:00:00Z",
                oldestProviderRefresh: "2026-04-21T10:30:00Z",
                staleProviders: staleProviders,
                hasStaleProviders: !staleProviders.isEmpty
            )
        )
    }
}

private extension ProviderSnapshot {
    static func fixture(provider: ProviderID) -> ProviderSnapshot {
        ProviderSnapshot(
            provider: provider.rawValue,
            available: true,
            sourceUsed: "sync",
            lastAttemptedSource: "sync",
            resolvedViaFallback: false,
            refreshDurationMs: 0,
            sourceAttempts: [],
            identity: nil,
            primary: nil,
            secondary: nil,
            tertiary: nil,
            credits: nil,
            status: nil,
            auth: ProviderAuthHealth(
                loginMethod: "sync",
                credentialBackend: nil,
                authMode: "sync",
                isAuthenticated: true,
                isRefreshable: false,
                isSourceCompatible: true,
                requiresRelogin: false,
                managedRestriction: nil,
                diagnosticCode: nil,
                failureReason: nil,
                lastValidatedAt: "2026-04-21T11:00:00Z",
                recoveryActions: []
            ),
            costSummary: ProviderCostSummary(
                todayTokens: 900,
                todayCostUSD: 9.0,
                last30DaysTokens: 1800,
                last30DaysCostUSD: 18.0,
                daily: []
            ),
            claudeUsage: nil,
            lastRefresh: "2026-04-21T11:00:00Z",
            stale: false,
            error: nil
        )
    }
}
