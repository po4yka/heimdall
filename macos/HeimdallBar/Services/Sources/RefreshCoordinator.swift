import Foundation
import HeimdallDomain

@MainActor
public final class RefreshCoordinator {
    private let sessionStore: AppSessionStore
    private let repository: ProviderRepository
    private let helperRuntime: any HelperRuntime
    private let adjunctLoader: any DashboardAdjunctLoading
    private let browserSessionManager: any BrowserSessionManaging
    private let widgetSnapshotCoordinator: WidgetSnapshotCoordinator
    private let providerDataSource: any ProviderDataSource
    private var pollTask: Task<Void, Never>?
    private var started = false

    public init(
        sessionStore: AppSessionStore,
        repository: ProviderRepository,
        helperRuntime: any HelperRuntime,
        adjunctLoader: any DashboardAdjunctLoading,
        browserSessionManager: any BrowserSessionManaging,
        widgetSnapshotCoordinator: WidgetSnapshotCoordinator,
        providerDataSource: any ProviderDataSource
    ) {
        self.sessionStore = sessionStore
        self.repository = repository
        self.helperRuntime = helperRuntime
        self.adjunctLoader = adjunctLoader
        self.browserSessionManager = browserSessionManager
        self.widgetSnapshotCoordinator = widgetSnapshotCoordinator
        self.providerDataSource = providerDataSource
    }

    public func start() {
        guard !self.started else { return }
        self.started = true
        Task { @MainActor [weak self] in
            guard let self else { return }
            await self.refresh(force: false, provider: nil)
            self.startPollingLoop()
        }
    }

    public func stop() async {
        self.pollTask?.cancel()
        self.pollTask = nil
        await self.helperRuntime.stopOwnedHelper()
    }

    public func refresh(force: Bool, provider: ProviderID? = nil) async {
        self.repository.beginRefresh(provider: provider)

        let helperReady = await self.helperRuntime.ensureServerRunning(port: self.sessionStore.config.helperPort)
        guard helperReady else {
            self.repository.finishRefresh(
                issue: AppIssue(kind: .helperStartup, message: "The local Heimdall server is still starting.")
            )
            return
        }

        do {
            let envelope = try await self.providerDataSource.fetchSnapshots(
                config: self.sessionStore.config,
                refresh: force,
                provider: provider
            )
            self.repository.apply(envelope.providers, replacing: provider == nil)
            await self.loadAdjuncts(for: provider.map { [$0] } ?? self.sessionStore.visibleProviders, forceRefresh: force)
            await self.loadImportedSessions(for: provider.map { [$0] } ?? ProviderID.allCases)
            self.repository.syncSelections(sessionStore: self.sessionStore)
            self.repository.finishRefresh(issue: nil)

            do {
                let snapshot = WidgetSnapshotBuilder.snapshot(
                    providers: self.sessionStore.visibleProviders,
                    snapshots: self.repository.snapshotsByProvider,
                    adjuncts: self.repository.adjunctSnapshots,
                    config: self.sessionStore.config,
                    generatedAt: ISO8601DateFormatter().string(from: Date())
                )
                _ = try self.widgetSnapshotCoordinator.persist(snapshot)
                self.repository.clearIssue(kind: .widgetPersistence)
            } catch {
                self.repository.recordIssue(
                    AppIssue(kind: .widgetPersistence, message: error.localizedDescription)
                )
            }
        } catch {
            self.repository.finishRefresh(
                issue: AppIssue(kind: .refresh, provider: provider, message: error.localizedDescription)
            )
        }
    }

    public func refreshBrowserImports() async {
        let providers = ProviderID.allCases
        await self.loadImportedSessions(for: providers)
        await self.loadImportCandidates(for: providers)
        await self.loadAdjuncts(for: self.sessionStore.visibleProviders, forceRefresh: false)
    }

    public func importBrowserSession(
        provider: ProviderID,
        candidate: BrowserSessionImportCandidate
    ) async {
        self.repository.beginImport(provider: provider, resetting: false)

        do {
            let session = try await self.browserSessionManager.importBrowserSession(provider: provider, candidate: candidate)
            self.repository.importedSessions[provider] = session
            await self.loadImportCandidates(for: [provider])
            await self.loadAdjuncts(for: [provider], forceRefresh: true)
            self.repository.finishImport(issue: nil)
        } catch {
            self.repository.finishImport(
                issue: AppIssue(kind: .browserImport, provider: provider, message: error.localizedDescription)
            )
        }
    }

    public func resetBrowserSession(provider: ProviderID) async {
        self.repository.beginImport(provider: provider, resetting: true)

        do {
            try await self.browserSessionManager.resetImportedSession(provider: provider)
            self.repository.importedSessions.removeValue(forKey: provider)
            await self.loadImportCandidates(for: [provider])
            await self.loadAdjuncts(for: [provider], forceRefresh: true)
            self.repository.finishImport(issue: nil)
        } catch {
            self.repository.finishImport(
                issue: AppIssue(kind: .browserImport, provider: provider, message: error.localizedDescription)
            )
        }
    }

    private func startPollingLoop() {
        self.pollTask?.cancel()
        self.pollTask = Task { @MainActor [weak self] in
            while let self, !Task.isCancelled {
                let refreshIntervalSeconds = self.sessionStore.config.refreshIntervalSeconds
                try? await Task.sleep(for: .seconds(refreshIntervalSeconds))
                if Task.isCancelled {
                    return
                }
                await self.refresh(force: false, provider: nil)
            }
        }
    }

    private func loadAdjuncts(
        for providers: [ProviderID],
        forceRefresh: Bool
    ) async {
        for provider in providers {
            let providerConfig = self.sessionStore.config.providerConfig(for: provider)
            let adjunct = await self.adjunctLoader.loadAdjunct(
                provider: provider,
                config: providerConfig,
                snapshot: self.repository.snapshot(for: provider),
                forceRefresh: forceRefresh,
                allowLiveNavigation: true
            )
            self.repository.setAdjunctSnapshot(adjunct, for: provider)
        }
    }

    private func loadImportedSessions(for providers: [ProviderID]) async {
        for provider in providers {
            self.repository.importedSessions[provider] = await self.browserSessionManager.importedSession(provider: provider)
        }
    }

    private func loadImportCandidates(for providers: [ProviderID]) async {
        for provider in providers {
            self.repository.browserImportCandidates[provider] = await self.browserSessionManager.discoverImportCandidates(provider: provider)
        }
    }
}
