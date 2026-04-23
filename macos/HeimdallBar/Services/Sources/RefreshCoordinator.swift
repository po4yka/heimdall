import Foundation
import HeimdallDomain
import os.log

@MainActor
public final class RefreshCoordinator {
    private struct AdjunctLoadRequest {
        let provider: ProviderID
        let config: ProviderConfig
        let snapshot: ProviderSnapshot?
    }

    private let sessionStore: AppSessionStore
    private let repository: ProviderRepository
    private let helperRuntime: any HelperRuntime
    private let adjunctLoader: any DashboardAdjunctLoading
    private let browserSessionManager: any BrowserSessionManaging
    private let widgetSnapshotCoordinator: WidgetSnapshotCoordinator
    private let providerDataSource: any ProviderDataSource
    private let snapshotSyncer: (any SnapshotSyncing)?
    private let localNotificationCoordinator: any LocalNotificationCoordinating
    private var pollTask: Task<Void, Never>?
    private var enrichmentTask: Task<Void, Never>?
    private var started = false
    private static let logger = Logger(subsystem: "dev.heimdall.HeimdallBar", category: "RefreshCoordinator")

    public init(
        sessionStore: AppSessionStore,
        repository: ProviderRepository,
        helperRuntime: any HelperRuntime,
        adjunctLoader: any DashboardAdjunctLoading,
        browserSessionManager: any BrowserSessionManaging,
        widgetSnapshotCoordinator: WidgetSnapshotCoordinator,
        providerDataSource: any ProviderDataSource,
        snapshotSyncer: (any SnapshotSyncing)? = nil,
        localNotificationCoordinator: any LocalNotificationCoordinating = NoopLocalNotificationCoordinator()
    ) {
        self.sessionStore = sessionStore
        self.repository = repository
        self.helperRuntime = helperRuntime
        self.adjunctLoader = adjunctLoader
        self.browserSessionManager = browserSessionManager
        self.widgetSnapshotCoordinator = widgetSnapshotCoordinator
        self.providerDataSource = providerDataSource
        self.snapshotSyncer = snapshotSyncer
        self.localNotificationCoordinator = localNotificationCoordinator
    }

    public func start() {
        guard !self.started else { return }
        self.started = true
        Task { @MainActor [weak self] in
            guard let self else { return }
            await self.refresh(force: false, provider: nil, startupOptimized: true)
            self.startPollingLoop()
        }
    }

    public func stop() async {
        self.pollTask?.cancel()
        self.pollTask = nil
        self.enrichmentTask?.cancel()
        self.enrichmentTask = nil
        await self.helperRuntime.stopOwnedHelper()
    }

    public func refresh(force: Bool, provider: ProviderID? = nil) async {
        await self.refresh(force: force, provider: provider, startupOptimized: false)
    }

    private func refresh(force: Bool, provider: ProviderID?, startupOptimized: Bool) async {
        self.repository.beginRefresh(provider: provider)
        self.enrichmentTask?.cancel()
        self.enrichmentTask = nil

        let helperReady = await self.helperRuntime.ensureServerRunning(port: self.sessionStore.config.helperPort)
        guard helperReady else {
            self.repository.finishRefresh(
                issue: AppIssue(kind: .helperStartup, message: "The local Heimdall server is still starting.")
            )
            return
        }

        do {
            let envelope = try await self.fetchSnapshots(
                force: force,
                provider: provider,
                startupOptimized: startupOptimized
            )
            self.repository.apply(envelope.providers, replacing: provider == nil)
            self.repository.syncSelections(sessionStore: self.sessionStore)
            self.repository.finishRefresh(issue: nil)
            if provider == nil {
                if let notificationIssue = await self.localNotificationCoordinator.process(
                    envelope: envelope,
                    config: self.sessionStore.config
                ) {
                    self.repository.recordIssue(notificationIssue)
                } else {
                    self.repository.clearIssue(kind: .localNotifications)
                }
            }
            self.scheduleEnrichment(
                envelope: envelope,
                provider: provider,
                forceRefresh: force,
                startupOptimized: startupOptimized
            )
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

    private func fetchSnapshots(
        force: Bool,
        provider: ProviderID?,
        startupOptimized: Bool
    ) async throws -> ProviderSnapshotEnvelope {
        if startupOptimized,
           !force,
           provider == nil,
           let startupDataSource = self.providerDataSource as? any StartupOptimizedProviderDataSource {
            do {
                return try await startupDataSource.fetchStartupSnapshots(config: self.sessionStore.config)
            } catch {
                Self.logger.error("Startup snapshot fetch failed, falling back to standard fetch: \(error.localizedDescription)")
            }
        }

        return try await self.providerDataSource.fetchSnapshots(
            config: self.sessionStore.config,
            refresh: force,
            provider: provider
        )
    }

    private func scheduleEnrichment(
        envelope: ProviderSnapshotEnvelope,
        provider: ProviderID?,
        forceRefresh: Bool,
        startupOptimized: Bool
    ) {
        let adjunctProviders = provider.map { [$0] } ?? self.sessionStore.visibleProviders
        let importedSessionProviders = provider.map { [$0] } ?? ProviderID.allCases

        self.enrichmentTask = Task { @MainActor [weak self] in
            guard let self else { return }
            await self.runEnrichment(
                envelope: envelope,
                provider: provider,
                adjunctProviders: adjunctProviders,
                importedSessionProviders: importedSessionProviders,
                forceRefresh: forceRefresh,
                startupOptimized: startupOptimized
            )
        }
    }

    private func runEnrichment(
        envelope: ProviderSnapshotEnvelope,
        provider: ProviderID?,
        adjunctProviders: [ProviderID],
        importedSessionProviders: [ProviderID],
        forceRefresh: Bool,
        startupOptimized: Bool
    ) async {
        var enrichmentEnvelope = envelope
        if startupOptimized, provider == nil, !forceRefresh {
            do {
                enrichmentEnvelope = try await self.providerDataSource.fetchSnapshots(
                    config: self.sessionStore.config,
                    refresh: false,
                    provider: nil
                )
                self.repository.apply(enrichmentEnvelope.providers, replacing: true)
                self.repository.syncSelections(sessionStore: self.sessionStore)
            } catch {
                Self.logger.error("Warm startup refresh failed: \(error.localizedDescription)")
            }
        }

        let adjunctRequests = adjunctProviders.map { provider in
            AdjunctLoadRequest(
                provider: provider,
                config: self.sessionStore.config.providerConfig(for: provider),
                snapshot: self.repository.snapshot(for: provider)
            )
        }

        async let adjunctResults = Self.loadAdjunctResults(
            loader: self.adjunctLoader,
            requests: adjunctRequests,
            forceRefresh: forceRefresh
        )
        async let importedSessions = Self.loadImportedSessions(
            manager: self.browserSessionManager,
            providers: importedSessionProviders
        )
        async let snapshotSyncIssue = Self.snapshotSyncIssue(
            provider: provider,
            snapshotSyncer: self.snapshotSyncer
        )

        let resolvedAdjuncts = await adjunctResults
        guard !Task.isCancelled else { return }
        for (provider, snapshot) in resolvedAdjuncts {
            self.repository.setAdjunctSnapshot(snapshot, for: provider)
        }

        let resolvedImportedSessions = await importedSessions
        guard !Task.isCancelled else { return }
        for (provider, session) in resolvedImportedSessions {
            self.repository.importedSessions[provider] = session
        }

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

        if let snapshotSyncIssue = await snapshotSyncIssue {
            Self.logger.error("Snapshot sync failed: \(snapshotSyncIssue.message)")
            self.repository.recordIssue(snapshotSyncIssue)
        } else if provider == nil, self.snapshotSyncer != nil {
            self.repository.clearIssue(kind: .snapshotSync)
        }
    }

    nonisolated private static func loadAdjunctResults(
        loader: any DashboardAdjunctLoading,
        requests: [AdjunctLoadRequest],
        forceRefresh: Bool
    ) async -> [ProviderID: DashboardAdjunctSnapshot?] {
        await withTaskGroup(of: (ProviderID, DashboardAdjunctSnapshot?).self, returning: [ProviderID: DashboardAdjunctSnapshot?].self) { group in
            for request in requests {
                group.addTask {
                    let snapshot = await loader.loadAdjunct(
                        provider: request.provider,
                        config: request.config,
                        snapshot: request.snapshot,
                        forceRefresh: forceRefresh,
                        allowLiveNavigation: true
                    )
                    return (request.provider, snapshot)
                }
            }

            var results: [ProviderID: DashboardAdjunctSnapshot?] = [:]
            for await (provider, snapshot) in group {
                results[provider] = snapshot
            }
            return results
        }
    }

    nonisolated private static func loadImportedSessions(
        manager: any BrowserSessionManaging,
        providers: [ProviderID]
    ) async -> [ProviderID: ImportedBrowserSession?] {
        await withTaskGroup(of: (ProviderID, ImportedBrowserSession?).self, returning: [ProviderID: ImportedBrowserSession?].self) { group in
            for provider in providers {
                group.addTask {
                    (provider, await manager.importedSession(provider: provider))
                }
            }

            var sessions: [ProviderID: ImportedBrowserSession?] = [:]
            for await (provider, session) in group {
                sessions[provider] = session
            }
            return sessions
        }
    }

    nonisolated private static func snapshotSyncIssue(
        provider: ProviderID?,
        snapshotSyncer: (any SnapshotSyncing)?
    ) async -> AppIssue? {
        guard provider == nil, let snapshotSyncer else { return nil }
        do {
            _ = try await snapshotSyncer.syncLatestSnapshot()
            return nil
        } catch {
            return AppIssue(kind: .snapshotSync, message: error.localizedDescription)
        }
    }

    private func loadAdjuncts(
        for providers: [ProviderID],
        forceRefresh: Bool
    ) async {
        let requests = providers.map { provider in
            AdjunctLoadRequest(
                provider: provider,
                config: self.sessionStore.config.providerConfig(for: provider),
                snapshot: self.repository.snapshot(for: provider)
            )
        }
        let results = await Self.loadAdjunctResults(
            loader: self.adjunctLoader,
            requests: requests,
            forceRefresh: forceRefresh
        )
        for (provider, snapshot) in results {
            self.repository.setAdjunctSnapshot(snapshot, for: provider)
        }
    }

    private func loadImportedSessions(for providers: [ProviderID]) async {
        let sessions = await Self.loadImportedSessions(
            manager: self.browserSessionManager,
            providers: providers
        )
        for (provider, session) in sessions {
            self.repository.importedSessions[provider] = session
        }
    }

    private func loadImportCandidates(for providers: [ProviderID]) async {
        for provider in providers {
            self.repository.browserImportCandidates[provider] = await self.browserSessionManager.discoverImportCandidates(provider: provider)
        }
    }
}
