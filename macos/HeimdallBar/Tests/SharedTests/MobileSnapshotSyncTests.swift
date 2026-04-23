import CloudKit
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

        #expect(synced.installations.first?.sourceDevice == "macbook-pro")
        #expect(persisted?.sourceDevice == "macbook-pro")
        #expect(persisted?.providers.count == 2)
    }

    @Test
    func cloudKitSnapshotStoreRoundTripsAggregateThroughBackingStore() async throws {
        let backingStore = InMemoryCloudSnapshotBackingStore()
        let persistence = UserDefaultsCloudSyncStateStore(defaults: UserDefaults(suiteName: "MobileSnapshotSyncTests.roundtrip")!)
        let store = CloudKitSnapshotSyncStore(backingStore: backingStore, persistence: persistence)
        let snapshot = MobileSnapshotEnvelope.fixture(sourceDevice: "studio")

        try await store.saveLatestSnapshot(snapshot)
        let loaded = try await store.loadAggregateSnapshot()

        #expect(loaded?.installations.first?.sourceDevice == "studio")
        #expect(loaded?.aggregateHistory90d().count == 2)
        #expect(loaded?.aggregateTotals.last90DaysTokens == snapshot.totals.last90DaysTokens)
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

    @Test
    func aggregateEnvelopeMergesProviderBreakdownsAcrossInstallations() {
        let firstSummary = ProviderCostSummary(
            todayTokens: 100,
            todayCostUSD: 10,
            last30DaysTokens: 300,
            last30DaysCostUSD: 30,
            daily: [],
            cacheHitRateToday: 25,
            cacheHitRate30d: 40,
            byModel: [
                ProviderModelRow(
                    model: "claude-sonnet",
                    costUSD: 10,
                    input: 100,
                    output: 40,
                    cacheRead: 10,
                    cacheCreation: 5,
                    reasoningOutput: 2,
                    turns: 3
                )
            ],
            byProject: [
                ProviderProjectRow(project: "personal", displayName: "Personal", costUSD: 10, turns: 3, sessions: 1)
            ],
            byTool: [
                ProviderToolRow(
                    toolName: "Read",
                    category: "fs",
                    mcpServer: nil,
                    invocations: 2,
                    errors: 0,
                    turnsUsed: 2,
                    sessionsUsed: 1
                )
            ],
            byMcp: [
                ProviderMcpRow(server: "filesystem", invocations: 2, toolsUsed: 1, sessionsUsed: 1)
            ],
            hourlyActivity: [
                ProviderHourlyBucket(hour: 9, turns: 3, costUSD: 10, tokens: 100)
            ],
            activityHeatmap: [
                ProviderHeatmapCell(dayOfWeek: 1, hour: 9, turns: 3)
            ],
            recentSessions: [
                ProviderSession(
                    sessionID: "session-1",
                    displayName: "Personal Session",
                    startedAt: "2026-04-21T10:00:00Z",
                    durationMinutes: 15,
                    turns: 3,
                    costUSD: 10,
                    model: "claude-sonnet"
                )
            ],
            subagentBreakdown: ProviderSubagentBreakdown(totalTurns: 4, totalCostUSD: 4, sessionCount: 1, agentCount: 1),
            versionBreakdown: [
                ProviderVersionRow(version: "1.0.0", turns: 3, sessions: 1, costUSD: 10)
            ]
        )
        let secondSummary = ProviderCostSummary(
            todayTokens: 100,
            todayCostUSD: 6,
            last30DaysTokens: 300,
            last30DaysCostUSD: 18,
            daily: [],
            cacheHitRateToday: 75,
            cacheHitRate30d: 60,
            byModel: [
                ProviderModelRow(
                    model: "claude-sonnet",
                    costUSD: 6,
                    input: 60,
                    output: 30,
                    cacheRead: 15,
                    cacheCreation: 0,
                    reasoningOutput: 1,
                    turns: 2
                )
            ],
            byProject: [
                ProviderProjectRow(project: "personal", displayName: "Personal", costUSD: 6, turns: 2, sessions: 1)
            ],
            byTool: [
                ProviderToolRow(
                    toolName: "Read",
                    category: "fs",
                    mcpServer: nil,
                    invocations: 1,
                    errors: 1,
                    turnsUsed: 1,
                    sessionsUsed: 1
                )
            ],
            byMcp: [
                ProviderMcpRow(server: "filesystem", invocations: 1, toolsUsed: 1, sessionsUsed: 1)
            ],
            hourlyActivity: [
                ProviderHourlyBucket(hour: 9, turns: 2, costUSD: 6, tokens: 100)
            ],
            activityHeatmap: [
                ProviderHeatmapCell(dayOfWeek: 1, hour: 9, turns: 2)
            ],
            recentSessions: [
                ProviderSession(
                    sessionID: "session-2",
                    displayName: "Work Session",
                    startedAt: "2026-04-21T11:00:00Z",
                    durationMinutes: 20,
                    turns: 2,
                    costUSD: 6,
                    model: "claude-sonnet"
                )
            ],
            subagentBreakdown: ProviderSubagentBreakdown(totalTurns: 2, totalCostUSD: 2, sessionCount: 1, agentCount: 1),
            versionBreakdown: [
                ProviderVersionRow(version: "1.0.0", turns: 2, sessions: 1, costUSD: 6)
            ]
        )

        let aggregate = SyncedAggregateEnvelope.aggregate(
            installations: [
                SyncedInstallationSnapshot.from(
                    mobileSnapshot: .fixture(sourceDevice: "personal-mac", providers: [.fixture(provider: .claude, costSummary: firstSummary)]),
                    installationID: "personal"
                ),
                SyncedInstallationSnapshot.from(
                    mobileSnapshot: .fixture(sourceDevice: "work-mac", providers: [.fixture(provider: .claude, costSummary: secondSummary)]),
                    installationID: "work"
                ),
            ],
            generatedAt: "2026-04-21T11:00:00Z"
        )

        let provider = try! #require(aggregate.aggregateProviderViews.first)
        #expect(provider.providerSnapshot.costSummary.byModel.count == 1)
        #expect(provider.providerSnapshot.costSummary.byModel.first?.costUSD == 16)
        #expect(provider.providerSnapshot.costSummary.byProject.count == 1)
        #expect(provider.providerSnapshot.costSummary.byProject.first?.turns == 5)
        #expect(provider.providerSnapshot.costSummary.byTool.count == 1)
        #expect(provider.providerSnapshot.costSummary.byTool.first?.invocations == 3)
        #expect(provider.providerSnapshot.costSummary.byMcp.count == 1)
        #expect(provider.providerSnapshot.costSummary.byMcp.first?.invocations == 3)
        #expect(provider.providerSnapshot.costSummary.hourlyActivity.count == 1)
        #expect(provider.providerSnapshot.costSummary.hourlyActivity.first?.tokens == 200)
        #expect(provider.providerSnapshot.costSummary.activityHeatmap.count == 1)
        #expect(provider.providerSnapshot.costSummary.activityHeatmap.first?.turns == 5)
        #expect(provider.providerSnapshot.costSummary.versionBreakdown.count == 1)
        #expect(provider.providerSnapshot.costSummary.versionBreakdown.first?.turns == 5)
        #expect(provider.providerSnapshot.costSummary.recentSessions.count == 2)
        #expect(provider.providerSnapshot.costSummary.subagentBreakdown?.totalTurns == 6)
        #expect(provider.providerSnapshot.costSummary.cacheHitRateToday == 50)
        #expect(provider.providerSnapshot.costSummary.cacheHitRate30d == 50)
    }
}

private actor InMemorySnapshotSyncStore: SnapshotSyncStore {
    private var snapshot: MobileSnapshotEnvelope?

    init(snapshot: MobileSnapshotEnvelope? = nil) {
        self.snapshot = snapshot
    }

    func loadLegacySnapshot() async throws -> MobileSnapshotEnvelope? {
        self.snapshot
    }

    func loadLiveAggregateSnapshot() async throws -> SyncedAggregateEnvelope? {
        self.snapshot.map { SyncedAggregateEnvelope.legacy(mobileSnapshot: $0, installationID: "test-installation") }
    }

    func saveLatestSnapshot(_ snapshot: MobileSnapshotEnvelope) async throws -> SyncedAggregateEnvelope {
        self.snapshot = snapshot
        return SyncedAggregateEnvelope.legacy(mobileSnapshot: snapshot, installationID: "test-installation")
    }

    func loadAggregateSnapshot() async throws -> SyncedAggregateEnvelope? {
        self.snapshot.map { SyncedAggregateEnvelope.legacy(mobileSnapshot: $0, installationID: "test-installation") }
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

private actor InMemoryCloudSnapshotBackingStore: CloudSnapshotBackingStore {
    private var legacy: MobileSnapshotEnvelope?
    private var installations: [SyncedInstallationSnapshot] = []

    func accountStatus() async throws -> CKAccountStatus {
        .available
    }

    func loadLegacySnapshot() async throws -> MobileSnapshotEnvelope? {
        self.legacy
    }

    func saveLegacySnapshot(_ snapshot: MobileSnapshotEnvelope) async throws {
        self.legacy = snapshot
    }

    func fetchInstallationSnapshots(state _: CloudSyncSpaceState) async throws -> [SyncedInstallationSnapshot] {
        self.installations
    }

    func saveInstallationSnapshot(
        _ snapshot: SyncedInstallationSnapshot,
        state _: CloudSyncSpaceState
    ) async throws -> CloudSyncSpaceState {
        self.installations = [snapshot]
        return CloudSyncSpaceState(role: .owner, status: .ownerReady, zoneName: "heimdall-sync-space")
    }

    func prepareOwnerShare(state _: CloudSyncSpaceState) async throws -> CloudSyncSpaceState {
        CloudSyncSpaceState(role: .owner, status: .inviteReady, shareURL: "https://example.com/share")
    }

    func acceptShareURL(_: URL, state _: CloudSyncSpaceState) async throws -> CloudSyncSpaceState {
        CloudSyncSpaceState(role: .participant, status: .participantJoined, zoneName: "heimdall-sync-space", zoneOwnerName: "_owner")
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
    static func fixture(
        provider: ProviderID,
        costSummary: ProviderCostSummary? = nil
    ) -> ProviderSnapshot {
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
            costSummary: costSummary ?? ProviderCostSummary(
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
