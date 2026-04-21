import Foundation
import HeimdallDomain
import HeimdallServices
import Testing

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

        #expect(await browserSessionManager.importedSessionCallCount == ProviderID.allCases.count)
        #expect(await browserSessionManager.discoverCandidatesCallCount == 0)
        #expect(repository.importedSessions.keys.count == ProviderID.allCases.count)
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
        ProviderSnapshotEnvelope(
            providers: [Self.snapshot(provider: .claude)],
            fetchedAt: "2026-04-21T09:00:00Z",
            requestedProvider: nil,
            responseScope: "all",
            cacheHit: false,
            refreshedProviders: ["claude"]
        )
    }

    func fetchCostSummary(
        config _: HeimdallBarConfig,
        provider: ProviderID
    ) async throws -> CostSummaryEnvelope {
        CostSummaryEnvelope(provider: provider.rawValue, summary: Self.snapshot(provider: provider).costSummary)
    }

    private static func snapshot(provider: ProviderID) -> ProviderSnapshot {
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
