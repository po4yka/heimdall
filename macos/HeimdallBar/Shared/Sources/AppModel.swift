import Foundation
import Observation
#if canImport(WidgetKit)
import WidgetKit
#endif

@MainActor
@Observable
public final class AppModel {
    public var config: HeimdallBarConfig
    public var snapshots: [ProviderSnapshot]
    public var selectedProvider: ProviderID
    public var selectedMergeTab: MergeMenuTab
    public var adjunctSnapshots: [ProviderID: DashboardAdjunctSnapshot]
    public var importedSessions: [ProviderID: ImportedBrowserSession]
    public var browserImportCandidates: [ProviderID: [BrowserSessionImportCandidate]]
    public var lastError: String?
    public var isRefreshing: Bool
    public var refreshingProvider: ProviderID?
    public var lastRefreshCompletedAt: Date?
    public var isImportingSession: Bool

    private let configStore: ConfigStore
    private let helperController: HeimdallHelperController
    private let dashboardAdjunctController: DashboardAdjunctController
    private var hasStarted: Bool

    public init(
        configStore: ConfigStore = .shared,
        helperController: HeimdallHelperController = HeimdallHelperController(),
        dashboardAdjunctController: DashboardAdjunctController = DashboardAdjunctController()
    ) {
        self.configStore = configStore
        self.helperController = helperController
        self.dashboardAdjunctController = dashboardAdjunctController
        self.config = configStore.load()
        self.snapshots = []
        self.selectedProvider = .claude
        self.selectedMergeTab = .overview
        self.adjunctSnapshots = [:]
        self.importedSessions = [:]
        self.browserImportCandidates = [:]
        self.lastError = nil
        self.isRefreshing = false
        self.refreshingProvider = nil
        self.lastRefreshCompletedAt = nil
        self.isImportingSession = false
        self.hasStarted = false
    }

    public var visibleProviders: [ProviderID] {
        ProviderID.allCases.filter { self.config.providerConfig(for: $0).enabled }
    }

    public var visibleTabs: [MergeMenuTab] {
        MenuProjectionBuilder.availableTabs(config: self.config)
    }

    public func start() {
        guard !self.hasStarted else { return }
        self.hasStarted = true
        Task { @MainActor [weak self] in
            guard let self else { return }
            await self.refresh(force: false, provider: nil)
            self.startRefreshLoop()
        }
    }

    public func prepareForExit() async {
        await self.helperController.stopOwnedHelper()
    }

    public func refresh(force: Bool, provider: ProviderID? = nil) async {
        self.isRefreshing = true
        self.refreshingProvider = provider
        defer { self.isRefreshing = false }
        await self.helperController.ensureServerRunning(port: self.config.helperPort)
        let client = HeimdallAPIClient(port: self.config.helperPort)
        do {
            let envelope: ProviderSnapshotEnvelope
            if force {
                envelope = try await client.refresh(provider: provider)
            } else {
                envelope = try await client.fetchSnapshots()
            }
            self.apply(envelope.providers, replacing: provider == nil)
            await self.loadAdjuncts(for: provider.map { [$0] } ?? self.visibleProviders, forceRefresh: force)
            await self.loadImportedSessions(for: provider.map { [$0] } ?? ProviderID.allCases)
            self.syncSelections()
            self.lastError = nil
            self.lastRefreshCompletedAt = Date()
            try? WidgetSnapshotStore.save(self.makeWidgetSnapshot())
            #if canImport(WidgetKit)
            WidgetCenter.shared.reloadAllTimelines()
            #endif
        } catch {
            self.lastError = error.localizedDescription
            self.lastRefreshCompletedAt = Date()
        }
        self.refreshingProvider = nil
    }

    public func saveConfig() {
        do {
            try self.configStore.save(self.config)
            self.syncSelections()
            Task { @MainActor [weak self] in
                guard let self else { return }
                await self.loadAdjuncts(for: self.visibleProviders, forceRefresh: false)
                await self.loadImportedSessions(for: ProviderID.allCases)
            }
        } catch {
            self.lastError = error.localizedDescription
        }
    }

    public func importedSession(for provider: ProviderID) -> ImportedBrowserSession? {
        self.importedSessions[provider]
    }

    public func importCandidates(for provider: ProviderID) -> [BrowserSessionImportCandidate] {
        self.browserImportCandidates[provider] ?? []
    }

    public func refreshBrowserImports() async {
        await self.loadImportedSessions(for: ProviderID.allCases)
        await self.loadAdjuncts(for: self.visibleProviders, forceRefresh: false)
    }

    public func importBrowserSession(
        provider: ProviderID,
        candidate: BrowserSessionImportCandidate
    ) async {
        self.isImportingSession = true
        defer { self.isImportingSession = false }

        do {
            let session = try await self.dashboardAdjunctController.importBrowserSession(provider: provider, candidate: candidate)
            self.importedSessions[provider] = session
            await self.loadAdjuncts(for: [provider], forceRefresh: true)
            self.lastError = nil
        } catch {
            self.lastError = error.localizedDescription
        }
    }

    public func resetBrowserSession(provider: ProviderID) async {
        self.isImportingSession = true
        defer { self.isImportingSession = false }

        do {
            try await self.dashboardAdjunctController.resetImportedSession(provider: provider)
            self.importedSessions.removeValue(forKey: provider)
            await self.loadAdjuncts(for: [provider], forceRefresh: true)
            self.lastError = nil
        } catch {
            self.lastError = error.localizedDescription
        }
    }

    public func snapshot(for provider: ProviderID) -> ProviderSnapshot? {
        self.snapshots.first(where: { $0.providerID == provider })
    }

    public func presentation(for provider: ProviderID) -> ProviderPresentationState {
        SourceResolver.presentation(
            for: provider,
            config: self.config.providerConfig(for: provider),
            snapshot: self.snapshot(for: provider),
            adjunct: self.adjunctSnapshots[provider]
        )
    }

    public func menuTitle(for provider: ProviderID?) -> String {
        let presentation = provider.map(self.presentation(for:))
            ?? self.visibleProviders.map(self.presentation(for:)).first
        return MenuProjectionBuilder.menuTitle(for: presentation, provider: provider, config: self.config)
    }

    public func projection(for provider: ProviderID) -> ProviderMenuProjection {
        MenuProjectionBuilder.projection(
            from: self.presentation(for: provider),
            config: self.config,
            isRefreshing: self.isRefreshing && (self.refreshingProvider == nil || self.refreshingProvider == provider),
            lastGlobalError: self.lastError
        )
    }

    public func overviewProjection() -> OverviewMenuProjection {
        MenuProjectionBuilder.overview(
            from: self.visibleProviders.map(self.projection(for:)),
            isRefreshing: self.isRefreshing,
            lastGlobalError: self.lastError
        )
    }

    public func refreshActionLabel(for tab: MergeMenuTab) -> String {
        if let provider = tab.providerID {
            return "Refresh \(provider.title)"
        }
        return "Refresh All"
    }

    public func makeWidgetSnapshot() -> WidgetSnapshot {
        WidgetProjectionBuilder.snapshot(
            entries: self.visibleProviders.compactMap { provider in
                let projection = self.projection(for: provider)
                let presentation = self.presentation(for: provider)
                guard self.snapshot(for: provider) != nil || presentation.adjunct != nil else { return nil }
                let costSummary = self.snapshot(for: provider)?.costSummary ?? ProviderCostSummary(
                    todayTokens: 0,
                    todayCostUSD: 0,
                    last30DaysTokens: 0,
                    last30DaysCostUSD: 0,
                    daily: []
                )
                return WidgetProjectionBuilder.entry(from: projection, costSummary: costSummary)
            },
            refreshIntervalSeconds: self.config.refreshIntervalSeconds,
            generatedAt: ISO8601DateFormatter().string(from: Date())
        )
    }

    private func startRefreshLoop() {
        Task { @MainActor [weak self] in
            while let self {
                let refreshIntervalSeconds = self.config.refreshIntervalSeconds
                try? await Task.sleep(for: .seconds(refreshIntervalSeconds))
                await self.refresh(force: false, provider: nil)
            }
        }
    }

    private func apply(_ incoming: [ProviderSnapshot], replacing: Bool) {
        if replacing {
            self.snapshots = incoming
            return
        }

        var merged = Dictionary(uniqueKeysWithValues: self.snapshots.compactMap { snapshot in
            snapshot.providerID.map { ($0, snapshot) }
        })
        for snapshot in incoming {
            if let provider = snapshot.providerID {
                merged[provider] = snapshot
            }
        }
        self.snapshots = ProviderID.allCases.compactMap { merged[$0] }
    }

    private func loadAdjuncts(for providers: [ProviderID], forceRefresh: Bool) async {
        for provider in providers {
            let providerConfig = self.config.providerConfig(for: provider)
            let adjunct = await self.dashboardAdjunctController.loadAdjunct(
                provider: provider,
                config: providerConfig,
                snapshot: self.snapshot(for: provider),
                forceRefresh: forceRefresh
            )
            self.adjunctSnapshots[provider] = adjunct
        }
    }

    private func loadImportedSessions(for providers: [ProviderID]) async {
        for provider in providers {
            self.importedSessions[provider] = await self.dashboardAdjunctController.importedSession(provider: provider)
            self.browserImportCandidates[provider] = await self.dashboardAdjunctController.discoverImportCandidates(provider: provider)
        }
    }

    private func syncSelections() {
        if !self.visibleProviders.contains(self.selectedProvider) {
            self.selectedProvider = self.visibleProviders.first ?? .claude
        }
        if !self.visibleTabs.contains(self.selectedMergeTab) {
            self.selectedMergeTab = self.visibleTabs.first ?? .overview
        }
    }
}
