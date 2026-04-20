import Foundation
import Observation

@MainActor
@Observable
public final class AppModel {
    public var config: HeimdallBarConfig
    public var snapshots: [ProviderSnapshot]
    public var selectedProvider: ProviderID
    public var selectedMergeTab: MergeMenuTab
    public var adjunctSnapshots: [ProviderID: DashboardAdjunctSnapshot]
    public var lastError: String?
    public var isRefreshing: Bool

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
        self.lastError = nil
        self.isRefreshing = false
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

    public func refresh(force: Bool, provider: ProviderID? = nil) async {
        self.isRefreshing = true
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
            await self.loadAdjuncts(for: provider.map { [$0] } ?? self.visibleProviders)
            self.syncSelections()
            self.lastError = nil
            try? WidgetSnapshotStore.save(self.makeWidgetSnapshot())
        } catch {
            self.lastError = error.localizedDescription
        }
    }

    public func saveConfig() {
        do {
            try self.configStore.save(self.config)
            self.syncSelections()
            Task { @MainActor [weak self] in
                guard let self else { return }
                await self.loadAdjuncts(for: self.visibleProviders)
            }
        } catch {
            self.lastError = error.localizedDescription
        }
    }

    public func snapshot(for provider: ProviderID) -> ProviderSnapshot? {
        self.snapshots.first(where: { $0.providerID == provider })
    }

    public func menuTitle(for provider: ProviderID?) -> String {
        let snapshot = provider.flatMap(self.snapshot(for:)) ?? self.visibleProviders.compactMap(self.snapshot(for:)).first
        return MenuProjectionBuilder.menuTitle(for: snapshot, provider: provider, config: self.config)
    }

    public func projection(for provider: ProviderID) -> ProviderMenuProjection {
        MenuProjectionBuilder.projection(
            for: provider,
            snapshot: self.snapshot(for: provider),
            config: self.config,
            adjunct: self.adjunctSnapshots[provider]
        )
    }

    public func overviewProjection() -> OverviewMenuProjection {
        MenuProjectionBuilder.overview(from: self.visibleProviders.map(self.projection(for:)))
    }

    public func refreshActionLabel(for tab: MergeMenuTab) -> String {
        if let provider = tab.providerID {
            return "Refresh \(provider.title)"
        }
        return "Refresh All"
    }

    public func makeWidgetSnapshot() -> WidgetSnapshot {
        WidgetSnapshot(
            generatedAt: ISO8601DateFormatter().string(from: Date()),
            entries: self.visibleProviders.compactMap { provider in
                guard let snapshot = self.snapshot(for: provider) else { return nil }
                return WidgetProviderEntry(
                    provider: provider,
                    title: provider.title,
                    primary: snapshot.primary,
                    secondary: snapshot.secondary,
                    credits: snapshot.credits,
                    costSummary: snapshot.costSummary,
                    updatedAt: snapshot.lastRefresh
                )
            }
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

    private func loadAdjuncts(for providers: [ProviderID]) async {
        for provider in providers {
            let providerConfig = self.config.providerConfig(for: provider)
            let adjunct = await self.dashboardAdjunctController.loadAdjunct(
                provider: provider,
                config: providerConfig,
                snapshot: self.snapshot(for: provider)
            )
            self.adjunctSnapshots[provider] = adjunct
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
