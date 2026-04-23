import Foundation
import HeimdallDomain
import HeimdallServices
@testable import HeimdallMobileApp
import Testing

@MainActor
struct MobileDashboardModelTests {
    @Test
    func loadHandlesMissingSnapshot() async {
        let model = MobileDashboardModel(store: StubSnapshotStore(aggregate: nil))

        await model.load()

        #expect(model.aggregate == nil)
        #expect(model.lastError == nil)
    }

    @Test
    func loadSelectsFirstAvailableProviderWhenDefaultIsMissing() async {
        let model = MobileDashboardModel(
            store: StubSnapshotStore(
                aggregate: SyncedAggregateEnvelope.legacy(
                    mobileSnapshot: MobileSnapshotEnvelope.fixture(
                    providers: [.fixture(provider: .codex)],
                    staleProviders: ["codex"]
                    ),
                    installationID: "codex-installation"
                )
            )
        )

        await model.load()

        #expect(model.selectedProvider == .codex)
        #expect(model.selectedProviderSnapshot?.providerID == .codex)
        #expect(model.aggregate?.staleInstallations.contains("codex-installation") == true)
    }

    @Test
    func loadPreservesMultipleProvidersAndHistory() async {
        let model = MobileDashboardModel(store: StubSnapshotStore(aggregate: .legacy(mobileSnapshot: .fixture(), installationID: "fixture-installation")))

        await model.load()

        #expect(model.providerSnapshots.count == 2)
        #expect(model.selectedHistorySeries?.daily.count == 2)
    }

    @Test
    func loadFailureKeepsStaleSnapshotWarningVisible() async {
        let model = MobileDashboardModel(store: FlakySnapshotStore())

        await model.load()
        await model.load()

        #expect(model.aggregate != nil)
        #expect(model.lastError == "sync failed")
        #expect(model.staleSnapshotWarning == "sync failed")
    }
}

private actor StubSnapshotStore: SnapshotSyncStore {
    let aggregate: SyncedAggregateEnvelope?

    init(aggregate: SyncedAggregateEnvelope?) {
        self.aggregate = aggregate
    }

    func loadLegacySnapshot() async throws -> MobileSnapshotEnvelope? {
        self.aggregate?.mobileSnapshotCompatibility
    }

    func saveLatestSnapshot(_ snapshot: MobileSnapshotEnvelope) async throws -> SyncedAggregateEnvelope {
        SyncedAggregateEnvelope.legacy(mobileSnapshot: snapshot, installationID: "stub-installation")
    }

    func loadAggregateSnapshot() async throws -> SyncedAggregateEnvelope? {
        self.aggregate
    }

    func loadCloudSyncSpaceState() async throws -> CloudSyncSpaceState {
        CloudSyncSpaceState(role: .owner, status: .ownerReady)
    }

    func prepareOwnerShare() async throws -> CloudSyncSpaceState {
        CloudSyncSpaceState(role: .owner, status: .inviteReady, shareURL: "https://example.com/share")
    }

    func acceptShareURL(_: URL) async throws -> CloudSyncSpaceState {
        CloudSyncSpaceState(role: .participant, status: .participantJoined)
    }
}

private actor FlakySnapshotStore: SnapshotSyncStore {
    private var loadCount = 0

    func loadLegacySnapshot() async throws -> MobileSnapshotEnvelope? {
        self.loadCount += 1
        if self.loadCount == 1 {
            return .fixture()
        }
        throw NSError(domain: "MobileDashboardModelTests", code: 1, userInfo: [
            NSLocalizedDescriptionKey: "sync failed",
        ])
    }

    func saveLatestSnapshot(_ snapshot: MobileSnapshotEnvelope) async throws -> SyncedAggregateEnvelope {
        SyncedAggregateEnvelope.legacy(mobileSnapshot: snapshot, installationID: "flaky-installation")
    }

    func loadAggregateSnapshot() async throws -> SyncedAggregateEnvelope? {
        self.loadCount += 1
        if self.loadCount == 1 {
            return .legacy(mobileSnapshot: .fixture(), installationID: "flaky-installation")
        }
        throw NSError(domain: "MobileDashboardModelTests", code: 1, userInfo: [
            NSLocalizedDescriptionKey: "sync failed",
        ])
    }

    func loadCloudSyncSpaceState() async throws -> CloudSyncSpaceState {
        CloudSyncSpaceState(role: .owner, status: .ownerReady)
    }

    func prepareOwnerShare() async throws -> CloudSyncSpaceState {
        CloudSyncSpaceState(role: .owner, status: .inviteReady)
    }

    func acceptShareURL(_: URL) async throws -> CloudSyncSpaceState {
        CloudSyncSpaceState(role: .participant, status: .participantJoined)
    }
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
