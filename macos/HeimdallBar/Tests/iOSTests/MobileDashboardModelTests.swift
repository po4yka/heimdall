import Foundation
import HeimdallDomain
import HeimdallServices
@testable import HeimdallMobileApp
import Testing

@MainActor
struct MobileDashboardModelTests {
    @Test
    func startupLoadsCachedAggregateBeforeLiveRefresh() async {
        let cachedAggregate = SyncedAggregateEnvelope.legacy(
            mobileSnapshot: .fixture(
                generatedAt: "2026-04-20T11:00:00Z",
                providers: [.fixture(provider: .claude)]
            ),
            installationID: "cached-installation"
        )
        let liveAggregate = SyncedAggregateEnvelope.legacy(
            mobileSnapshot: .fixture(
                generatedAt: "2026-04-21T11:00:00Z",
                providers: [.fixture(provider: .codex)]
            ),
            installationID: "live-installation"
        )
        let cache = StubAggregateCache(
            cached: CachedSyncedAggregateEnvelope(
                aggregate: cachedAggregate,
                cachedAt: "2026-04-20T11:01:00Z",
                lastSuccessfulRefreshAt: "2026-04-20T11:01:00Z"
            )
        )
        let store = ControlledSnapshotStore(
            liveAggregate: liveAggregate,
            liveDelayNanoseconds: 25_000_000
        )
        let model = MobileDashboardModel(store: store, cache: cache)

        let refreshTask = Task {
            await model.refresh(reason: .startup)
        }

        await waitUntil { await store.liveAggregateLoadCount == 1 }

        #expect(model.aggregate?.generatedAt == cachedAggregate.generatedAt)

        await refreshTask.value

        #expect(model.aggregate?.generatedAt == liveAggregate.generatedAt)
        #expect(model.selectedProvider == .codex)
    }

    @Test
    func foregroundRefreshRespectsThrottle() async {
        let clock = MutableNow(Date(timeIntervalSince1970: 1_000))
        let aggregate = SyncedAggregateEnvelope.legacy(
            mobileSnapshot: .fixture(generatedAt: "2026-04-21T11:00:00Z"),
            installationID: "fixture-installation"
        )
        let store = ControlledSnapshotStore(liveAggregate: aggregate)
        let model = MobileDashboardModel(
            store: store,
            now: { clock.current },
            foregroundRefreshThrottle: 60
        )

        await model.refresh(reason: .manual)
        clock.current = clock.current.addingTimeInterval(30)
        await model.refresh(reason: .foreground)

        #expect(await store.liveAggregateLoadCount == 1)
    }

    @Test
    func manualRefreshBypassesThrottle() async {
        let clock = MutableNow(Date(timeIntervalSince1970: 1_000))
        let aggregate = SyncedAggregateEnvelope.legacy(
            mobileSnapshot: .fixture(generatedAt: "2026-04-21T11:00:00Z"),
            installationID: "fixture-installation"
        )
        let store = ControlledSnapshotStore(liveAggregate: aggregate)
        let model = MobileDashboardModel(
            store: store,
            now: { clock.current },
            foregroundRefreshThrottle: 60
        )

        await model.refresh(reason: .manual)
        clock.current = clock.current.addingTimeInterval(30)
        await model.refresh(reason: .manual)

        #expect(await store.liveAggregateLoadCount == 2)
    }

    @Test
    func shareAcceptanceForcesRefreshAndUpdatesCloudSyncState() async {
        let aggregate = SyncedAggregateEnvelope.legacy(
            mobileSnapshot: .fixture(generatedAt: "2026-04-21T11:00:00Z"),
            installationID: "fixture-installation"
        )
        let store = ControlledSnapshotStore(
            liveAggregate: aggregate,
            acceptShareState: CloudSyncSpaceState(
                role: .participant,
                status: .participantJoined,
                lastAcceptedAt: "2026-04-21T11:05:00Z"
            )
        )
        let model = MobileDashboardModel(store: store)

        await model.acceptShareURL(URL(string: "https://example.com/share")!)

        #expect(model.aggregate?.generatedAt == aggregate.generatedAt)
        #expect(model.cloudSyncState.status == .participantJoined)
        #expect(await store.acceptShareCount == 1)
        #expect(await store.liveAggregateLoadCount == 1)
    }

    @Test
    func failedLiveRefreshPreservesCachedDataAndShowsWarning() async {
        let cachedAggregate = SyncedAggregateEnvelope.legacy(
            mobileSnapshot: .fixture(generatedAt: "2026-04-21T10:00:00Z"),
            installationID: "cached-installation"
        )
        let cache = StubAggregateCache(
            cached: CachedSyncedAggregateEnvelope(
                aggregate: cachedAggregate,
                cachedAt: "2026-04-21T10:00:05Z",
                lastSuccessfulRefreshAt: "2026-04-21T10:00:05Z"
            )
        )
        let store = ControlledSnapshotStore(liveError: FixtureError.syncFailed)
        let model = MobileDashboardModel(store: store, cache: cache)

        await model.refresh(reason: .startup)

        #expect(model.aggregate?.generatedAt == cachedAggregate.generatedAt)
        #expect(model.lastRefreshError == "sync failed")
        #expect(model.staleSnapshotWarning == "sync failed")
    }

    @Test
    func loadFailureWithoutCacheStillShowsErrorState() async {
        let store = ControlledSnapshotStore(liveError: FixtureError.syncFailed)
        let model = MobileDashboardModel(store: store)

        await model.refresh(reason: .manual)

        #expect(model.aggregate == nil)
        #expect(model.lastRefreshError == "sync failed")
    }

    @Test
    func legacyFallbackWorksWhenNoAggregateRecordsExist() async {
        let legacy = MobileSnapshotEnvelope.fixture(
            generatedAt: "2026-04-21T11:00:00Z",
            providers: [.fixture(provider: .codex)]
        )
        let store = ControlledSnapshotStore(
            liveAggregate: nil,
            aggregateFallback: .legacy(mobileSnapshot: legacy, installationID: "legacy-installation"),
            legacySnapshot: legacy
        )
        let model = MobileDashboardModel(store: store)

        await model.refresh(reason: .startup)

        #expect(model.aggregate?.generatedAt == legacy.generatedAt)
        #expect(model.selectedProvider == .codex)
    }

    @Test
    func widgetSnapshotAdapterBuildsStableProviderPayloadsFromAggregate() {
        let aggregate = SyncedAggregateEnvelope.legacy(
            mobileSnapshot: .fixture(
                generatedAt: "2026-04-21T11:00:00Z",
                providers: [
                    .fixture(
                        provider: .claude,
                        identity: ProviderIdentity(
                            provider: ProviderID.claude.rawValue,
                            accountEmail: "private@example.com",
                            accountOrganization: "Private",
                            loginMethod: "oauth",
                            plan: "Max"
                        ),
                        status: ProviderStatusSummary(
                            indicator: "minor",
                            description: "Minor incident",
                            pageURL: "https://status.example.com"
                        ),
                        primary: ProviderRateWindow(
                            usedPercent: 62,
                            resetsAt: "2026-04-21T12:00:00Z",
                            resetsInMinutes: 42,
                            windowMinutes: 300,
                            resetLabel: "42m"
                        )
                    ),
                ]
            ),
            installationID: "fixture-installation"
        )

        let widgetSnapshot = WidgetSnapshotBuilder.snapshot(aggregate: aggregate, defaultRefreshIntervalSeconds: 900)
        let provider = widgetSnapshot.providerSnapshot(for: .claude)

        #expect(widgetSnapshot.generatedAt == aggregate.generatedAt)
        #expect(widgetSnapshot.defaultRefreshIntervalSeconds == 900)
        #expect(provider?.cost.todayTokens == 900)
        #expect(provider?.freshness.visualState == .degraded)
        #expect(provider?.identity?.accountEmail == nil)
        #expect(provider?.identity?.plan == "Max")
        #expect(provider?.lanes.first?.usedPercent == 62)
    }

    @Test
    func widgetSnapshotPersistenceReloadsOnlyWhenPayloadChanges() async throws {
        let writer = InMemoryWidgetSnapshotWriter()
        let reloader = CountingWidgetReloader()
        let coordinator = WidgetSnapshotCoordinator(writer: writer, reloader: reloader)
        let aggregate = SyncedAggregateEnvelope.legacy(
            mobileSnapshot: .fixture(generatedAt: "2026-04-21T11:00:00Z"),
            installationID: "fixture-installation"
        )
        let store = ControlledSnapshotStore(liveAggregate: aggregate)
        let model = MobileDashboardModel(
            store: store,
            widgetSnapshotCoordinator: coordinator
        )

        await model.refresh(reason: .manual)
        await model.refresh(reason: .manual)

        #expect(reloader.reloadCount == 1)
        #expect(writer.savedSnapshots.count == 2)
        switch writer.load() {
        case .success(let snapshot):
            #expect(snapshot.generatedAt == aggregate.generatedAt)
        case .empty, .failure:
            Issue.record("expected widget snapshot to be persisted")
        }
    }

    @Test
    func cloudSyncStatusCopyMapsUnavailableStates() async {
        let unavailableModel = MobileDashboardModel(
            store: ControlledSnapshotStore(
                liveAggregate: nil,
                cloudSyncState: CloudSyncSpaceState(status: .iCloudUnavailable)
            )
        )
        await unavailableModel.refresh(reason: .manual)
        #expect(unavailableModel.cloudSyncStatusTitle == "iCloud unavailable")
        #expect(unavailableModel.cloudSyncStatusDetail == "Sign in to iCloud on this iPhone to refresh synced usage data.")

        let restrictedModel = MobileDashboardModel(
            store: ControlledSnapshotStore(
                liveAggregate: nil,
                cloudSyncState: CloudSyncSpaceState(status: .sharingBlocked)
            )
        )
        await restrictedModel.refresh(reason: .manual)
        #expect(restrictedModel.cloudSyncStatusTitle == "Sharing blocked")
        #expect(restrictedModel.cloudSyncStatusDetail == "CloudKit sharing is restricted on this iPhone.")
    }
}

@MainActor
private func waitUntil(
    maxAttempts: Int = 100,
    predicate: @escaping () async -> Bool
) async {
    for _ in 0..<maxAttempts {
        if await predicate() {
            return
        }
        await Task.yield()
    }
    Issue.record("timed out waiting for async condition")
}

private enum FixtureError: LocalizedError {
    case syncFailed

    var errorDescription: String? {
        switch self {
        case .syncFailed:
            return "sync failed"
        }
    }
}

private actor ControlledSnapshotStore: SnapshotSyncStore {
    let liveAggregate: SyncedAggregateEnvelope?
    let aggregateFallback: SyncedAggregateEnvelope?
    let legacySnapshot: MobileSnapshotEnvelope?
    let cloudSyncState: CloudSyncSpaceState
    let acceptShareState: CloudSyncSpaceState
    let liveError: Error?
    let aggregateError: Error?
    let liveDelayNanoseconds: UInt64

    private(set) var liveAggregateLoadCount = 0
    private(set) var acceptShareCount = 0

    init(
        liveAggregate: SyncedAggregateEnvelope? = nil,
        aggregateFallback: SyncedAggregateEnvelope? = nil,
        legacySnapshot: MobileSnapshotEnvelope? = nil,
        cloudSyncState: CloudSyncSpaceState = CloudSyncSpaceState(role: .participant, status: .participantJoined),
        acceptShareState: CloudSyncSpaceState = CloudSyncSpaceState(role: .participant, status: .participantJoined),
        liveError: Error? = nil,
        aggregateError: Error? = nil,
        liveDelayNanoseconds: UInt64 = 0
    ) {
        self.liveAggregate = liveAggregate
        self.aggregateFallback = aggregateFallback
        self.legacySnapshot = legacySnapshot
        self.cloudSyncState = cloudSyncState
        self.acceptShareState = acceptShareState
        self.liveError = liveError
        self.aggregateError = aggregateError
        self.liveDelayNanoseconds = liveDelayNanoseconds
    }

    func loadLiveAggregateSnapshot() async throws -> SyncedAggregateEnvelope? {
        self.liveAggregateLoadCount += 1
        if self.liveDelayNanoseconds > 0 {
            try? await Task.sleep(nanoseconds: self.liveDelayNanoseconds)
        }
        if let liveError {
            throw liveError
        }
        return self.liveAggregate
    }

    func loadLegacySnapshot() async throws -> MobileSnapshotEnvelope? {
        self.legacySnapshot
    }

    func saveLatestSnapshot(_ snapshot: MobileSnapshotEnvelope) async throws -> SyncedAggregateEnvelope {
        SyncedAggregateEnvelope.legacy(mobileSnapshot: snapshot, installationID: "stub-installation")
    }

    func loadAggregateSnapshot() async throws -> SyncedAggregateEnvelope? {
        if let aggregateError {
            throw aggregateError
        }
        if let aggregateFallback {
            return aggregateFallback
        }
        if let liveAggregate {
            return liveAggregate
        }
        if let legacySnapshot {
            return SyncedAggregateEnvelope.legacy(mobileSnapshot: legacySnapshot, installationID: "legacy-installation")
        }
        return nil
    }

    func loadCloudSyncSpaceState() async throws -> CloudSyncSpaceState {
        self.cloudSyncState
    }

    func prepareOwnerShare() async throws -> CloudSyncSpaceState {
        CloudSyncSpaceState(role: .owner, status: .inviteReady, shareURL: "https://example.com/share")
    }

    func acceptShareURL(_: URL) async throws -> CloudSyncSpaceState {
        self.acceptShareCount += 1
        return self.acceptShareState
    }
}

private actor StubAggregateCache: SyncedAggregateCaching {
    var cached: CachedSyncedAggregateEnvelope?

    init(cached: CachedSyncedAggregateEnvelope? = nil) {
        self.cached = cached
    }

    func loadCachedAggregate() async throws -> CachedSyncedAggregateEnvelope? {
        self.cached
    }

    func saveCachedAggregate(_ cached: CachedSyncedAggregateEnvelope) async throws {
        self.cached = cached
    }
}

private final class MutableNow: @unchecked Sendable {
    var current: Date

    init(_ current: Date) {
        self.current = current
    }
}

private final class InMemoryWidgetSnapshotWriter: @unchecked Sendable, WidgetSnapshotWriter {
    private(set) var savedSnapshots: [WidgetSnapshot] = []
    private var latestSnapshot: WidgetSnapshot?
    private let encoder: JSONEncoder = {
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.sortedKeys]
        return encoder
    }()

    func save(_ snapshot: WidgetSnapshot) throws -> WidgetSnapshotSaveResult {
        self.savedSnapshots.append(snapshot)
        defer { self.latestSnapshot = snapshot }
        guard let latestSnapshot else {
            return .saved
        }
        let latestData = try self.encoder.encode(latestSnapshot)
        let nextData = try self.encoder.encode(snapshot)
        return latestData == nextData ? .unchanged : .saved
    }

    func load() -> WidgetSnapshotLoadResult {
        guard let latestSnapshot else { return .empty }
        return .success(latestSnapshot)
    }
}

private final class CountingWidgetReloader: @unchecked Sendable, WidgetReloading {
    private(set) var reloadCount = 0

    func reloadAllTimelines() {
        self.reloadCount += 1
    }
}

private extension MobileSnapshotEnvelope {
    static func fixture(
        generatedAt: String = "2026-04-21T11:00:00Z",
        providers: [ProviderSnapshot] = [.fixture(provider: .claude), .fixture(provider: .codex)],
        staleProviders: [String] = []
    ) -> MobileSnapshotEnvelope {
        MobileSnapshotEnvelope(
            generatedAt: generatedAt,
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
                newestProviderRefresh: generatedAt,
                oldestProviderRefresh: "2026-04-21T10:30:00Z",
                staleProviders: staleProviders,
                hasStaleProviders: !staleProviders.isEmpty
            )
        )
    }
}

private extension ProviderSnapshot {
    static func fixture(
        provider: ProviderID,
        identity: ProviderIdentity? = nil,
        status: ProviderStatusSummary? = nil,
        primary: ProviderRateWindow? = nil
    ) -> ProviderSnapshot {
        ProviderSnapshot(
            provider: provider.rawValue,
            available: true,
            sourceUsed: "auto",
            lastAttemptedSource: "auto",
            resolvedViaFallback: false,
            refreshDurationMs: 0,
            sourceAttempts: [],
            identity: identity,
            primary: primary,
            secondary: nil,
            tertiary: nil,
            credits: nil,
            status: status,
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
