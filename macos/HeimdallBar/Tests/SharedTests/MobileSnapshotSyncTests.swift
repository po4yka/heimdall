import Foundation
import HeimdallDomain
@testable import HeimdallServices
import Testing

struct MobileSnapshotSyncTests {
    @Test
    func snapshotSyncCoordinatorFetchesAndPersistsSnapshot() async throws {
        let store = InMemorySnapshotSyncStore()
        let coordinator = SnapshotSyncCoordinator(
            client: StubMobileSnapshotClient(snapshot: .fixture(sourceDevice: "macbook-pro")),
            store: store
        )

        let synced = try await coordinator.syncLatestSnapshot()
        let persisted = try await store.loadLatestSnapshot()

        #expect(synced.sourceDevice == "macbook-pro")
        #expect(persisted?.sourceDevice == "macbook-pro")
        #expect(persisted?.providers.count == 2)
    }

    @Test
    func cloudKitSnapshotStoreRoundTripsPayloadThroughTransport() async throws {
        let transport = InMemorySnapshotPayloadTransport()
        let store = CloudKitSnapshotSyncStore(transport: transport)
        let snapshot = MobileSnapshotEnvelope.fixture(sourceDevice: "studio")

        try await store.saveLatestSnapshot(snapshot)
        let loaded = try await store.loadLatestSnapshot()

        #expect(loaded?.sourceDevice == "studio")
        #expect(loaded?.history90d.count == 2)
        #expect(loaded?.totals.last90DaysTokens == snapshot.totals.last90DaysTokens)
    }

    @Test
    func snapshotStoreSyncProviderClientUsesStoredMobileSnapshot() async throws {
        let store = InMemorySnapshotSyncStore(
            snapshot: .fixture(sourceDevice: "mac-mini", providers: [.fixture(provider: .codex)])
        )
        let client = SnapshotStoreSyncProviderClient(store: store)

        let envelope = try await client.fetchSyncedSnapshots()

        #expect(envelope.providers.count == 1)
        #expect(envelope.providers.first?.providerID == .codex)
        #expect(envelope.responseScope == "all")
    }
}

private actor InMemorySnapshotSyncStore: SnapshotSyncStore {
    private var snapshot: MobileSnapshotEnvelope?

    init(snapshot: MobileSnapshotEnvelope? = nil) {
        self.snapshot = snapshot
    }

    func loadLatestSnapshot() async throws -> MobileSnapshotEnvelope? {
        self.snapshot
    }

    func saveLatestSnapshot(_ snapshot: MobileSnapshotEnvelope) async throws {
        self.snapshot = snapshot
    }
}

private actor InMemorySnapshotPayloadTransport: SnapshotPayloadTransport {
    private var payload: Data?

    func fetchPayload(recordType _: String, recordName _: String) async throws -> Data? {
        self.payload
    }

    func savePayload(
        _ payload: Data,
        contractVersion _: Int,
        generatedAt _: String,
        sourceDevice _: String,
        recordType _: String,
        recordName _: String
    ) async throws {
        self.payload = payload
    }
}

private struct StubMobileSnapshotClient: MobileSnapshotClient {
    let snapshot: MobileSnapshotEnvelope

    func fetchMobileSnapshot() async throws -> MobileSnapshotEnvelope {
        self.snapshot
    }
}

private extension MobileSnapshotEnvelope {
    static func fixture(
        sourceDevice: String,
        providers: [ProviderSnapshot] = [.fixture(provider: .claude), .fixture(provider: .codex)]
    ) -> MobileSnapshotEnvelope {
        MobileSnapshotEnvelope(
            generatedAt: "2026-04-21T10:00:00Z",
            sourceDevice: sourceDevice,
            providers: providers,
            history90d: providers.map { provider in
                MobileProviderHistorySeries(
                    provider: provider.provider,
                    daily: [
                        CostHistoryPoint(day: "2026-04-20", totalTokens: 1200, costUSD: 12.0),
                        CostHistoryPoint(day: "2026-04-21", totalTokens: 900, costUSD: 9.0),
                    ],
                    totalTokens: 2100,
                    totalCostUSD: 21.0
                )
            },
            totals: MobileSnapshotTotals(
                todayTokens: 1900,
                todayCostUSD: 19.0,
                last90DaysTokens: 4200,
                last90DaysCostUSD: 42.0,
                todayBreakdown: TokenBreakdown(input: 100, output: 200),
                last90DaysBreakdown: TokenBreakdown(input: 300, output: 500)
            ),
            freshness: MobileSnapshotFreshness(
                newestProviderRefresh: "2026-04-21T10:00:00Z",
                oldestProviderRefresh: "2026-04-21T09:45:00Z",
                staleProviders: [],
                hasStaleProviders: false
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
            identity: ProviderIdentity(
                provider: provider.rawValue,
                accountEmail: "\(provider.rawValue)@example.com",
                accountOrganization: nil,
                loginMethod: "sync",
                plan: "pro"
            ),
            primary: ProviderRateWindow(
                usedPercent: 50,
                resetsAt: "2026-04-21T12:00:00Z",
                resetsInMinutes: 120,
                windowMinutes: 300,
                resetLabel: "2h"
            ),
            secondary: nil,
            tertiary: nil,
            credits: provider == .codex ? 14 : nil,
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
                lastValidatedAt: "2026-04-21T10:00:00Z",
                recoveryActions: []
            ),
            costSummary: ProviderCostSummary(
                todayTokens: 900,
                todayCostUSD: 9.0,
                last30DaysTokens: 2100,
                last30DaysCostUSD: 21.0,
                daily: []
            ),
            claudeUsage: nil,
            lastRefresh: "2026-04-21T10:00:00Z",
            stale: false,
            error: nil
        )
    }
}
