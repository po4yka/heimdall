import Foundation
import HeimdallDomain
import HeimdallServices
import Testing
@testable import HeimdallPlatformMac

struct ProviderDataSourceTests {
    @Test
    func syncedProviderDataSourceFiltersProviderSnapshots() async throws {
        let dataSource = SyncedProviderDataSource(client: StubSyncProviderClient())

        let envelope = try await dataSource.fetchSnapshots(
            config: .default,
            refresh: true,
            provider: .codex
        )

        #expect(envelope.providers.count == 1)
        #expect(envelope.providers.first?.providerID == .codex)
        #expect(envelope.requestedProvider == "codex")
        #expect(envelope.responseScope == "provider")
    }

    @Test
    func syncedProviderDataSourceDerivesCostSummaryFromSnapshots() async throws {
        let dataSource = SyncedProviderDataSource(client: StubSyncProviderClient())

        let envelope = try await dataSource.fetchCostSummary(
            config: .default,
            provider: .claude
        )

        #expect(envelope.provider == "claude")
        #expect(envelope.summary.todayTokens == 1200)
        #expect(envelope.summary.last30DaysCostUSD == 45.0)
    }
}

struct RefreshCoordinatorTests {
    @MainActor
    @Test
    func startUsesStartupOptimizedSnapshotFetch() async {
        let providerDataSource = StartupOptimizedProviderDataSourceSpy()
        let coordinator = RefreshCoordinator(
            sessionStore: AppSessionStore(persistence: NoopAppSessionStateStore()),
            repository: ProviderRepository(),
            helperRuntime: ReadyHelperRuntime(),
            adjunctLoader: NoopAdjunctLoader(),
            browserSessionManager: BrowserSessionManagerSpy(),
            widgetSnapshotCoordinator: WidgetSnapshotCoordinator(
                writer: StubWidgetSnapshotWriter(),
                reloader: NoopWidgetReloader()
            ),
            providerDataSource: providerDataSource
        )

        coordinator.start()

        let startupFetchObserved = await Self.waitUntil {
            await providerDataSource.callOrder.first == "startup"
        }

        #expect(startupFetchObserved)
        #expect(await providerDataSource.callOrder.first == "startup")
        #expect(await providerDataSource.startupFetchCallCount == 1)

        await coordinator.stop()
    }

    @MainActor
    @Test
    func refreshKeepsBrowserCandidateDiscoveryOutOfSteadyStatePath() async throws {
        let browserSessionManager = BrowserSessionManagerSpy()
        let repository = ProviderRepository()
        let coordinator = RefreshCoordinator(
            sessionStore: AppSessionStore(persistence: NoopAppSessionStateStore()),
            repository: repository,
            helperRuntime: ReadyHelperRuntime(),
            adjunctLoader: NoopAdjunctLoader(),
            browserSessionManager: browserSessionManager,
            widgetSnapshotCoordinator: WidgetSnapshotCoordinator(
                writer: StubWidgetSnapshotWriter(),
                reloader: NoopWidgetReloader()
            ),
            providerDataSource: StubProviderDataSource()
        )

        await coordinator.refresh(force: false, provider: nil)

        let enrichmentFinished = await Self.waitUntil {
            await browserSessionManager.importedSessionCallCount == ProviderID.allCases.count &&
            repository.importedSessions.keys.count == ProviderID.allCases.count
        }

        #expect(enrichmentFinished)
        #expect(await browserSessionManager.discoverCandidatesCallCount == 0)
        #expect(repository.browserImportCandidates.isEmpty)
    }

    @MainActor
    @Test
    func refreshBrowserImportsDiscoversCandidatesOnDemand() async {
        let browserSessionManager = BrowserSessionManagerSpy()
        let repository = ProviderRepository()
        let coordinator = RefreshCoordinator(
            sessionStore: AppSessionStore(persistence: NoopAppSessionStateStore()),
            repository: repository,
            helperRuntime: ReadyHelperRuntime(),
            adjunctLoader: NoopAdjunctLoader(),
            browserSessionManager: browserSessionManager,
            widgetSnapshotCoordinator: WidgetSnapshotCoordinator(
                writer: StubWidgetSnapshotWriter(),
                reloader: NoopWidgetReloader()
            ),
            providerDataSource: StubProviderDataSource()
        )

        await coordinator.refreshBrowserImports()

        #expect(await browserSessionManager.importedSessionCallCount == ProviderID.allCases.count)
        #expect(await browserSessionManager.discoverCandidatesCallCount == ProviderID.allCases.count)
        #expect(repository.browserImportCandidates.keys.count == ProviderID.allCases.count)
        #expect(repository.browserImportCandidates[.claude] == [BrowserSessionManagerSpy.candidate])
    }

    @MainActor
    @Test
    func refreshAppliesCoreSnapshotBeforeBackgroundEnrichmentFinishes() async {
        let adjunctGate = AsyncGate()
        let importedSessionGate = AsyncGate()
        let adjunctLoader = BlockingAdjunctLoader(gate: adjunctGate)
        let browserSessionManager = BlockingBrowserSessionManager(gate: importedSessionGate)
        let widgetWriter = RecordingWidgetSnapshotWriter()
        let snapshotSyncer = SnapshotSyncerSpy()
        let repository = ProviderRepository()
        let coordinator = RefreshCoordinator(
            sessionStore: AppSessionStore(persistence: NoopAppSessionStateStore()),
            repository: repository,
            helperRuntime: ReadyHelperRuntime(),
            adjunctLoader: adjunctLoader,
            browserSessionManager: browserSessionManager,
            widgetSnapshotCoordinator: WidgetSnapshotCoordinator(
                writer: widgetWriter,
                reloader: NoopWidgetReloader()
            ),
            providerDataSource: StubProviderDataSource(),
            snapshotSyncer: snapshotSyncer
        )

        await coordinator.refresh(force: false, provider: nil)

        let enrichmentStarted = await Self.waitUntil {
            let adjunctStarts = await adjunctLoader.startedProviderCount
            let importedSessionCalls = await browserSessionManager.importedSessionCallCount
            return adjunctStarts == ProviderID.allCases.count &&
                importedSessionCalls == ProviderID.allCases.count
        }

        #expect(repository.refreshActivity == .idle)
        #expect(repository.snapshotsByProvider[.claude]?.providerID == .claude)
        #expect(enrichmentStarted)
        #expect(repository.adjunctSnapshots.isEmpty)
        #expect(repository.importedSessions.isEmpty)
        #expect(widgetWriter.saveCallCount == 0)

        await adjunctGate.open()
        await importedSessionGate.open()

        let enrichmentFinished = await Self.waitUntil {
            repository.adjunctSnapshots.keys.count == ProviderID.allCases.count &&
            repository.importedSessions.keys.count == ProviderID.allCases.count &&
            widgetWriter.saveCallCount == 1
        }

        #expect(enrichmentFinished)
        #expect(await snapshotSyncer.callCount == 1)

        await coordinator.stop()
    }

    @MainActor
    private static func waitUntil(
        timeoutNanoseconds: UInt64 = 1_000_000_000,
        pollNanoseconds: UInt64 = 10_000_000,
        condition: () async -> Bool
    ) async -> Bool {
        let deadline = DispatchTime.now().uptimeNanoseconds + timeoutNanoseconds
        while DispatchTime.now().uptimeNanoseconds < deadline {
            if await condition() {
                return true
            }
            try? await Task.sleep(nanoseconds: pollNanoseconds)
        }
        return await condition()
    }
}

private struct StubSyncProviderClient: SyncProviderClient {
    func fetchSyncedSnapshots() async throws -> ProviderSnapshotEnvelope {
        ProviderSnapshotEnvelope(
            contractVersion: LiveProviderContract.version,
            providers: [
                ProviderSnapshot(
                    provider: "claude",
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
                    auth: stubAuthHealth,
                    costSummary: ProviderCostSummary(
                        todayTokens: 1200,
                        todayCostUSD: 12.5,
                        last30DaysTokens: 4500,
                        last30DaysCostUSD: 45.0,
                        daily: []
                    ),
                    claudeUsage: nil,
                    lastRefresh: "2026-04-21T09:00:00Z",
                    stale: false,
                    error: nil
                ),
                ProviderSnapshot(
                    provider: "codex",
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
                    auth: stubAuthHealth,
                    costSummary: ProviderCostSummary(
                        todayTokens: 800,
                        todayCostUSD: 8.0,
                        last30DaysTokens: 3200,
                        last30DaysCostUSD: 32.0,
                        daily: []
                    ),
                    claudeUsage: nil,
                    lastRefresh: "2026-04-21T09:00:00Z",
                    stale: false,
                    error: nil
                ),
            ],
            fetchedAt: "2026-04-21T09:00:00Z",
            requestedProvider: nil,
            responseScope: "all",
            cacheHit: false,
            refreshedProviders: ["claude", "codex"]
        )
    }

}

private actor StartupOptimizedProviderDataSourceSpy: StartupOptimizedProviderDataSource {
    private(set) var callOrder: [String] = []
    private(set) var startupFetchCallCount = 0
    private(set) var standardFetchCallCount = 0

    func fetchStartupSnapshots(config _: HeimdallBarConfig) async throws -> ProviderSnapshotEnvelope {
        self.callOrder.append("startup")
        self.startupFetchCallCount += 1
        return StubProviderDataSource.envelope(provider: .claude)
    }

    func fetchSnapshots(
        config _: HeimdallBarConfig,
        refresh _: Bool,
        provider _: ProviderID?
    ) async throws -> ProviderSnapshotEnvelope {
        self.callOrder.append("standard")
        self.standardFetchCallCount += 1
        return StubProviderDataSource.envelope(provider: .claude)
    }

    func fetchCostSummary(
        config _: HeimdallBarConfig,
        provider: ProviderID
    ) async throws -> CostSummaryEnvelope {
        CostSummaryEnvelope(provider: provider.rawValue, summary: StubProviderDataSource.snapshot(provider: provider).costSummary)
    }
}

private actor AsyncGate {
    private var continuations: [CheckedContinuation<Void, Never>] = []

    func wait() async {
        await withCheckedContinuation { continuation in
            self.continuations.append(continuation)
        }
    }

    func open() {
        let continuations = self.continuations
        self.continuations.removeAll()
        for continuation in continuations {
            continuation.resume()
        }
    }
}

private actor BlockingAdjunctLoader: DashboardAdjunctLoading {
    private let gate: AsyncGate
    private(set) var startedProviderCount = 0

    init(gate: AsyncGate) {
        self.gate = gate
    }

    func loadAdjunct(
        provider: ProviderID,
        config _: ProviderConfig,
        snapshot _: ProviderSnapshot?,
        forceRefresh _: Bool,
        allowLiveNavigation _: Bool
    ) async -> DashboardAdjunctSnapshot? {
        self.startedProviderCount += 1
        await self.gate.wait()
        return DashboardAdjunctSnapshot(
            provider: provider,
            source: .oauth,
            headline: "\(provider.rawValue)-headline",
            detailLines: []
        )
    }
}

private actor BlockingBrowserSessionManager: BrowserSessionManaging {
    private let gate: AsyncGate
    private(set) var importedSessionCallCount = 0

    init(gate: AsyncGate) {
        self.gate = gate
    }

    func importedSession(provider: ProviderID) async -> ImportedBrowserSession? {
        self.importedSessionCallCount += 1
        await self.gate.wait()
        return ImportedBrowserSession(
            provider: provider,
            browserSource: .chrome,
            profileName: "Default",
            importedAt: "2026-04-21T09:00:00Z",
            storageKind: "sqlite",
            cookies: [],
            loginRequired: false,
            expired: false,
            lastValidatedAt: "2026-04-21T09:05:00Z"
        )
    }

    func discoverImportCandidates(provider _: ProviderID) async -> [BrowserSessionImportCandidate] {
        []
    }

    func importBrowserSession(
        provider: ProviderID,
        candidate _: BrowserSessionImportCandidate
    ) async throws -> ImportedBrowserSession {
        await self.importedSession(provider: provider) ?? ImportedBrowserSession(
            provider: provider,
            browserSource: .chrome,
            profileName: "Default",
            importedAt: "2026-04-21T09:00:00Z",
            storageKind: "sqlite",
            cookies: [],
            loginRequired: false,
            expired: false,
            lastValidatedAt: "2026-04-21T09:05:00Z"
        )
    }

    func resetImportedSession(provider _: ProviderID) async throws {}
}

private actor SnapshotSyncerSpy: SnapshotSyncing {
    private(set) var callCount = 0

    func syncLatestSnapshot() async throws -> MobileSnapshotEnvelope {
        self.callCount += 1
        return MobileSnapshotEnvelope(
            generatedAt: "2026-04-21T09:00:00Z",
            sourceDevice: "test-device",
            providers: [],
            history90d: [],
            totals: MobileSnapshotTotals(
                todayTokens: 0,
                todayCostUSD: 0,
                last90DaysTokens: 0,
                last90DaysCostUSD: 0
            ),
            freshness: MobileSnapshotFreshness(
                newestProviderRefresh: nil,
                oldestProviderRefresh: nil,
                staleProviders: [],
                hasStaleProviders: false
            )
        )
    }
}

private final class RecordingWidgetSnapshotWriter: WidgetSnapshotWriter, @unchecked Sendable {
    private(set) var saveCallCount = 0

    func save(_ snapshot: WidgetSnapshot) throws -> WidgetSnapshotSaveResult {
        self.saveCallCount += 1
        return .saved
    }

    func load() -> WidgetSnapshotLoadResult {
        .empty
    }
}

private actor BrowserSessionManagerSpy: BrowserSessionManaging {
    private(set) var importedSessionCallCount = 0
    private(set) var discoverCandidatesCallCount = 0

    static let candidate = BrowserSessionImportCandidate(
        browserSource: .chrome,
        profileName: "Default",
        storePath: "/tmp/Default",
        storageKind: "sqlite"
    )

    func importedSession(provider: ProviderID) async -> ImportedBrowserSession? {
        self.importedSessionCallCount += 1
        return ImportedBrowserSession(
            provider: provider,
            browserSource: .chrome,
            profileName: "Default",
            importedAt: "2026-04-21T09:00:00Z",
            storageKind: "sqlite",
            cookies: [],
            loginRequired: false,
            expired: false,
            lastValidatedAt: "2026-04-21T09:05:00Z"
        )
    }

    func discoverImportCandidates(provider _: ProviderID) async -> [BrowserSessionImportCandidate] {
        self.discoverCandidatesCallCount += 1
        return [Self.candidate]
    }

    func importBrowserSession(
        provider: ProviderID,
        candidate _: BrowserSessionImportCandidate
    ) async throws -> ImportedBrowserSession {
        await self.importedSession(provider: provider) ?? ImportedBrowserSession(
            provider: provider,
            browserSource: .chrome,
            profileName: "Default",
            importedAt: "2026-04-21T09:00:00Z",
            storageKind: "sqlite",
            cookies: [],
            loginRequired: false,
            expired: false,
            lastValidatedAt: "2026-04-21T09:05:00Z"
        )
    }

    func resetImportedSession(provider _: ProviderID) async throws {}
}

private struct ReadyHelperRuntime: HelperRuntime {
    func ensureServerRunning(port _: Int) async -> Bool { true }
    func stopOwnedHelper() async {}
}

private struct NoopAdjunctLoader: DashboardAdjunctLoading {
    func loadAdjunct(
        provider _: ProviderID,
        config _: ProviderConfig,
        snapshot _: ProviderSnapshot?,
        forceRefresh _: Bool,
        allowLiveNavigation _: Bool
    ) async -> DashboardAdjunctSnapshot? {
        nil
    }
}

private struct StubProviderDataSource: ProviderDataSource {
    func fetchSnapshots(
        config _: HeimdallBarConfig,
        refresh _: Bool,
        provider _: ProviderID?
    ) async throws -> ProviderSnapshotEnvelope {
        Self.envelope(provider: .claude)
    }

    func fetchCostSummary(
        config _: HeimdallBarConfig,
        provider: ProviderID
    ) async throws -> CostSummaryEnvelope {
        CostSummaryEnvelope(provider: provider.rawValue, summary: Self.snapshot(provider: provider).costSummary)
    }

    static func envelope(provider: ProviderID) -> ProviderSnapshotEnvelope {
        ProviderSnapshotEnvelope(
            providers: [Self.snapshot(provider: provider)],
            fetchedAt: "2026-04-21T09:00:00Z",
            requestedProvider: nil,
            responseScope: "all",
            cacheHit: false,
            refreshedProviders: [provider.rawValue]
        )
    }

    static func snapshot(provider: ProviderID) -> ProviderSnapshot {
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
            auth: stubAuthHealth,
            costSummary: ProviderCostSummary(
                todayTokens: 10,
                todayCostUSD: 1,
                last30DaysTokens: 10,
                last30DaysCostUSD: 1,
                daily: []
            ),
            claudeUsage: nil,
            lastRefresh: "2026-04-21T09:00:00Z",
            stale: false,
            error: nil
        )
    }
}

private struct StubWidgetSnapshotWriter: WidgetSnapshotWriter {
    func save(_ snapshot: WidgetSnapshot) throws -> WidgetSnapshotSaveResult {
        .unchanged
    }

    func load() -> WidgetSnapshotLoadResult {
        .empty
    }
}

private struct NoopAppSessionStateStore: AppSessionStatePersisting {
    func loadAppSessionState() -> PersistedAppSessionState? { nil }
    func saveAppSessionState(_ state: PersistedAppSessionState) {}
}

private let stubAuthHealth = ProviderAuthHealth(
    loginMethod: "sync",
    credentialBackend: "remote",
    authMode: "sync",
    isAuthenticated: true,
    isRefreshable: true,
    isSourceCompatible: true,
    requiresRelogin: false,
    managedRestriction: nil,
    diagnosticCode: "authenticated-compatible",
    failureReason: nil,
    lastValidatedAt: "2026-04-21T09:00:00Z",
    recoveryActions: []
)
