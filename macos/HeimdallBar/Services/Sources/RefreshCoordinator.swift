import Foundation
import HeimdallDomain

@MainActor
public final class RefreshCoordinator {
    private let sessionStore: AppSessionStore
    private let repository: ProviderRepository
    private let helperRuntime: any HelperRuntime
    private let adjunctProvider: any AdjunctProvider
    private let widgetSnapshotCoordinator: WidgetSnapshotCoordinator
    private let liveProviderClientFactory: @Sendable (Int) -> any LiveProviderClient
    private var pollTask: Task<Void, Never>?
    private var started = false

    public init(
        sessionStore: AppSessionStore,
        repository: ProviderRepository,
        helperRuntime: any HelperRuntime,
        adjunctProvider: any AdjunctProvider,
        widgetSnapshotCoordinator: WidgetSnapshotCoordinator,
        liveProviderClientFactory: @escaping @Sendable (Int) -> any LiveProviderClient
    ) {
        self.sessionStore = sessionStore
        self.repository = repository
        self.helperRuntime = helperRuntime
        self.adjunctProvider = adjunctProvider
        self.widgetSnapshotCoordinator = widgetSnapshotCoordinator
        self.liveProviderClientFactory = liveProviderClientFactory
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
        self.repository.isRefreshing = true
        self.repository.refreshingProvider = provider
        defer {
            self.repository.isRefreshing = false
            self.repository.refreshingProvider = nil
        }

        let helperReady = await self.helperRuntime.ensureServerRunning(port: self.sessionStore.config.helperPort)
        guard helperReady else {
            self.repository.lastError = "The local Heimdall server is still starting."
            self.repository.lastRefreshCompletedAt = Date()
            return
        }

        let client = self.liveProviderClientFactory(self.sessionStore.config.helperPort)
        do {
            let envelope = if force {
                try await client.refresh(provider: provider)
            } else {
                try await client.fetchSnapshots()
            }
            self.repository.apply(envelope.providers, replacing: provider == nil)
            await self.loadAdjuncts(for: provider.map { [$0] } ?? self.sessionStore.visibleProviders, forceRefresh: force)
            await self.loadImportedSessions(for: provider.map { [$0] } ?? ProviderID.allCases)
            self.repository.syncSelections(sessionStore: self.sessionStore)
            self.repository.lastError = nil
            self.repository.lastRefreshCompletedAt = Date()

            do {
                let snapshot = WidgetSnapshotBuilder.snapshot(
                    providers: self.sessionStore.visibleProviders,
                    snapshots: self.repository.snapshotsByProvider,
                    adjuncts: self.repository.adjunctSnapshots,
                    config: self.sessionStore.config,
                    generatedAt: ISO8601DateFormatter().string(from: Date())
                )
                _ = try self.widgetSnapshotCoordinator.persist(snapshot)
            } catch {
                self.repository.lastError = error.localizedDescription
            }
        } catch {
            self.repository.lastError = error.localizedDescription
            self.repository.lastRefreshCompletedAt = Date()
        }
    }

    public func refreshBrowserImports() async {
        await self.loadImportedSessions(for: ProviderID.allCases)
        await self.loadAdjuncts(for: self.sessionStore.visibleProviders, forceRefresh: false)
    }

    public func importBrowserSession(
        provider: ProviderID,
        candidate: BrowserSessionImportCandidate
    ) async {
        self.repository.isImportingSession = true
        defer { self.repository.isImportingSession = false }

        do {
            let session = try await self.adjunctProvider.importBrowserSession(provider: provider, candidate: candidate)
            self.repository.importedSessions[provider] = session
            await self.loadAdjuncts(for: [provider], forceRefresh: true)
            self.repository.lastError = nil
        } catch {
            self.repository.lastError = error.localizedDescription
        }
    }

    public func resetBrowserSession(provider: ProviderID) async {
        self.repository.isImportingSession = true
        defer { self.repository.isImportingSession = false }

        do {
            try await self.adjunctProvider.resetImportedSession(provider: provider)
            self.repository.importedSessions.removeValue(forKey: provider)
            await self.loadAdjuncts(for: [provider], forceRefresh: true)
            self.repository.lastError = nil
        } catch {
            self.repository.lastError = error.localizedDescription
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
            let adjunct = await self.adjunctProvider.loadAdjunct(
                provider: provider,
                config: providerConfig,
                snapshot: self.repository.snapshot(for: provider),
                forceRefresh: forceRefresh,
                allowLiveNavigation: true
            )
            self.repository.adjunctSnapshots[provider] = adjunct
        }
    }

    private func loadImportedSessions(for providers: [ProviderID]) async {
        for provider in providers {
            self.repository.importedSessions[provider] = await self.adjunctProvider.importedSession(provider: provider)
            self.repository.browserImportCandidates[provider] = await self.adjunctProvider.discoverImportCandidates(provider: provider)
        }
    }
}
