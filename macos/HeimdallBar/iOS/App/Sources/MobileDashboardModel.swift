import Foundation
import HeimdallDomain
import HeimdallServices
import Observation

enum MobileRefreshReason: Sendable, Equatable {
    case startup
    case foreground
    case manual
    case shareAccepted
}

enum RemotePushResult: Sendable, Equatable {
    case newData
    case failed
}

@MainActor
@Observable
final class MobileDashboardModel {
    var aggregate: SyncedAggregateEnvelope?
    var isLoading = false
    var lastRefreshError: String?
    var selectedProvider: ProviderID
    var selectedAccountScope: MobileAccountScope
    var compressionPreference: MobileCompressionPreference
    var cloudSyncState = CloudSyncSpaceState()
    var lastSuccessfulRefreshAt: String?
    var isAliasEditorPresented = false
    private(set) var aliases: [MobileAccountAlias]
    private(set) var collapsedSectionIDs: Set<String>
    private var projection = MobileScopedAggregateProjection.empty

    private let store: any SnapshotSyncStore
    private let cache: any SyncedAggregateCaching
    private let preferencesStore: any MobileDashboardPreferencesPersisting
    private let widgetSnapshotCoordinator: WidgetSnapshotCoordinator?
    private let now: @Sendable () -> Date
    private let foregroundRefreshThrottle: TimeInterval
    private let widgetRefreshIntervalSeconds: Int
    @ObservationIgnored private var accountObserver: CloudKitAccountObserver?

    init(
        store: any SnapshotSyncStore,
        cache: any SyncedAggregateCaching = NoopSyncedAggregateCache(),
        preferencesStore: any MobileDashboardPreferencesPersisting = UserDefaultsMobileDashboardPreferencesStore(),
        widgetSnapshotCoordinator: WidgetSnapshotCoordinator? = nil,
        now: @escaping @Sendable () -> Date = Date.init,
        foregroundRefreshThrottle: TimeInterval = 60,
        widgetRefreshIntervalSeconds: Int = 900,
        observesCloudKitAccount: Bool = false
    ) {
        let persistedPreferences = preferencesStore.loadPreferences()
        let initialCompressionPreference = persistedPreferences?.compressionPreference ?? .compact
        let initialCollapsedSectionIDs = persistedPreferences?.collapsedSectionIDs ?? []
        self.store = store
        self.cache = cache
        self.preferencesStore = preferencesStore
        self.widgetSnapshotCoordinator = widgetSnapshotCoordinator
        self.now = now
        self.foregroundRefreshThrottle = foregroundRefreshThrottle
        self.widgetRefreshIntervalSeconds = widgetRefreshIntervalSeconds
        self.selectedProvider = persistedPreferences?.selectedProvider ?? .claude
        self.selectedAccountScope = persistedPreferences?.selectedAccountScope ?? .all
        self.compressionPreference = initialCompressionPreference
        self.aliases = persistedPreferences?.aliases ?? []
        if initialCollapsedSectionIDs.isEmpty {
            self.collapsedSectionIDs = Self.defaultCollapsedSectionIDs(for: initialCompressionPreference)
        } else {
            self.collapsedSectionIDs = Set(initialCollapsedSectionIDs)
        }
        self.accountObserver = nil
        if observesCloudKitAccount {
            let observer = CloudKitAccountObserver { [weak self] in
                Task { @MainActor in
                    await self?.handleAccountChanged()
                }
            }
            observer.start()
            self.accountObserver = observer
        }
    }

    var providerSnapshots: [ProviderSnapshot] {
        self.projection.providerSnapshots
    }

    var selectedProviderSnapshot: ProviderSnapshot? {
        self.providerSnapshots.first(where: { $0.providerID == self.selectedProvider })
            ?? self.providerSnapshots.first
    }

    var selectedHistorySeries: MobileProviderHistorySeries? {
        self.projection.scopedAggregate?.aggregateHistorySeries(for: self.selectedProvider)
            ?? self.projection.scopedAggregate?.aggregateHistory90d().first
    }

    var installations: [SyncedInstallationSnapshot] {
        self.projection.scopedAggregate?.installations ?? []
    }

    var visibleInstallations: [SyncedInstallationSnapshot] {
        self.projection.visibleInstallations
    }

    var rolledUpInstallations: [SyncedInstallationSnapshot] {
        self.projection.rolledUpInstallations
    }

    var accountSources: [MobileAccountSourceLabel] {
        self.projection.accountSources
    }

    var accountOptions: [MobileAccountOption] {
        self.projection.accountOptions
    }

    var aliasAccountOptions: [MobileAccountOption] {
        self.accountOptions.filter(\.isAliasBacked)
    }

    var rawAccountOptions: [MobileAccountOption] {
        self.accountOptions.filter { !$0.isAliasBacked && $0.scope != .all }
    }

    var accountSummaries: [MobileAccountSummary] {
        self.projection.visibleAccountSummaries
    }

    var rolledUpAccountSummary: MobileAccountSummary? {
        self.projection.rolledUpAccountSummary
    }

    var scopedAggregate: SyncedAggregateEnvelope? {
        self.projection.scopedAggregate
    }

    var scopedTotals: MobileSnapshotTotals? {
        self.projection.scopedAggregate?.aggregateTotals
    }

    var visibleProviders: [ProviderID] {
        self.projection.visibleProviders
    }

    var selectedScopeTitle: String {
        self.projection.selectedScopeTitle
    }

    var selectedScopeDescription: String {
        self.projection.selectedScopeDescription
    }

    var hasScopedData: Bool {
        self.projection.hasScopedData
    }

    var hasSnapshot: Bool {
        self.aggregate != nil
    }

    var lastError: String? {
        self.lastRefreshError
    }

    var staleSnapshotWarning: String? {
        guard self.aggregate != nil else { return nil }
        return self.lastRefreshError
    }

    var scopedEmptyStateTitle: String {
        "No Data In \(self.selectedScopeTitle)"
    }

    var scopedEmptyStateMessage: String {
        switch self.selectedAccountScope {
        case .all:
            return "No synced account data is available yet."
        case .account, .alias:
            return "The selected account scope does not currently have synced provider data."
        }
    }

    var cloudSyncStatusTitle: String {
        switch self.cloudSyncState.status {
        case .participantJoined:
            return "Joined"
        case .ownerReady, .inviteReady:
            return "Configured"
        case .notConfigured:
            return "Not configured"
        case .iCloudUnavailable:
            return "iCloud unavailable"
        case .sharingBlocked:
            return "Sharing blocked"
        }
    }

    var cloudSyncStatusDetail: String {
        if let statusMessage = self.cloudSyncState.statusMessage, !statusMessage.isEmpty {
            return statusMessage
        }
        switch self.cloudSyncState.status {
        case .participantJoined:
            return "This iPhone is connected to the shared Heimdall sync space."
        case .ownerReady, .inviteReady:
            return "Cloud Sync is configured for this Apple account."
        case .notConfigured:
            return "Open a Heimdall share link from macOS to join Cloud Sync on this iPhone."
        case .iCloudUnavailable:
            return "Sign in to iCloud on this iPhone to refresh synced usage data."
        case .sharingBlocked:
            return "CloudKit sharing is restricted on this iPhone."
        }
    }

    var newestPublishedAt: String? {
        self.projection.scopedAggregate?.installations.map(\.publishedAt).max()
            ?? self.aggregate?.installations.map(\.publishedAt).max()
    }

    func load() async {
        await self.refresh(reason: .startup)
    }

    func refresh(reason: MobileRefreshReason) async {
        if reason == .startup {
            await self.primeFromCache()
            // Let SwiftUI render cached content before the live CloudKit fetch continues.
            await Task.yield()
        }

        guard self.shouldRefresh(for: reason) else { return }

        self.isLoading = true
        defer { self.isLoading = false }

        do {
            self.cloudSyncState = try await self.store.loadCloudSyncSpaceState()
        } catch {
            if self.lastRefreshError == nil {
                self.lastRefreshError = error.localizedDescription
            }
        }

        do {
            if let liveAggregate = try await self.store.loadLiveAggregateSnapshot() {
                await self.applyFreshAggregate(liveAggregate)
                return
            }

            if await self.restoreCachedAggregate(errorMessage: nil) {
                return
            }

            self.aggregate = nil
            self.lastRefreshError = nil
        } catch {
            let message = error.localizedDescription
            if await self.restoreCachedAggregate(errorMessage: message) {
                return
            }
            self.lastRefreshError = message
            self.aggregate = nil
        }
    }

    func acceptShareURL(_ url: URL) async {
        self.isLoading = true
        defer { self.isLoading = false }

        do {
            self.cloudSyncState = try await self.store.acceptShareURL(url)
            self.lastRefreshError = nil
        } catch {
            self.lastRefreshError = error.localizedDescription
            return
        }

        await self.refresh(reason: .shareAccepted)
    }

    func handleAccountChanged() async {
        self.aggregate = nil
        self.lastSuccessfulRefreshAt = nil
        self.projection = .empty
        try? await self.cache.purgeCachedAggregate()
        await self.refresh(reason: .startup)
    }

    func handleRemotePush() async -> RemotePushResult {
        let errorBefore = self.lastRefreshError
        await self.refresh(reason: .manual)
        self.widgetSnapshotCoordinator?.reloadTimelines()
        let succeeded = self.lastRefreshError == nil
            || self.lastRefreshError == errorBefore
        return succeeded && self.aggregate != nil ? .newData : .failed
    }

    func selectProvider(_ provider: ProviderID) {
        self.selectedProvider = provider
        self.persistPreferences()
    }

    func selectAccountScope(_ scope: MobileAccountScope) {
        self.selectedAccountScope = scope
        self.rebuildProjection()
    }

    func setCompressionPreference(_ preference: MobileCompressionPreference) {
        guard self.compressionPreference != preference else { return }
        self.compressionPreference = preference
        self.collapsedSectionIDs = Self.defaultCollapsedSectionIDs(for: preference)
        self.rebuildProjection()
    }

    func replaceAliases(_ aliases: [MobileAccountAlias]) {
        self.aliases = aliases
        self.rebuildProjection()
    }

    func presentAliasEditor() {
        self.isAliasEditorPresented = true
    }

    func dismissAliasEditor() {
        self.isAliasEditorPresented = false
    }

    func isSectionCollapsed(_ sectionID: String) -> Bool {
        self.collapsedSectionIDs.contains(sectionID)
    }

    func toggleSectionCollapsed(_ sectionID: String) {
        if self.collapsedSectionIDs.contains(sectionID) {
            self.collapsedSectionIDs.remove(sectionID)
        } else {
            self.collapsedSectionIDs.insert(sectionID)
        }
        self.persistPreferences()
    }

    private func shouldRefresh(for reason: MobileRefreshReason) -> Bool {
        switch reason {
        case .startup, .manual, .shareAccepted:
            return true
        case .foreground:
            guard let lastSuccessfulRefreshAt else { return true }
            guard let lastRefreshDate = Self.isoFormatter.date(from: lastSuccessfulRefreshAt) else {
                return true
            }
            return self.now().timeIntervalSince(lastRefreshDate) >= self.foregroundRefreshThrottle
        }
    }

    private func primeFromCache() async {
        guard let cached = try? await self.cache.loadCachedAggregate() else {
            return
        }
        self.aggregate = cached.aggregate
        self.lastSuccessfulRefreshAt = cached.lastSuccessfulRefreshAt
        self.rebuildProjection()
    }

    private func restoreCachedAggregate(errorMessage: String?) async -> Bool {
        guard let cached = try? await self.cache.loadCachedAggregate() else {
            return false
        }
        self.aggregate = cached.aggregate
        self.lastSuccessfulRefreshAt = cached.lastSuccessfulRefreshAt
        self.lastRefreshError = errorMessage
        self.rebuildProjection()
        return true
    }

    private func applyFreshAggregate(_ aggregate: SyncedAggregateEnvelope) async {
        self.aggregate = aggregate
        self.lastRefreshError = nil
        self.lastSuccessfulRefreshAt = Self.isoFormatter.string(from: self.now())
        self.rebuildProjection()

        if let lastSuccessfulRefreshAt {
            let cached = CachedSyncedAggregateEnvelope(
                aggregate: aggregate,
                cachedAt: Self.isoFormatter.string(from: self.now()),
                lastSuccessfulRefreshAt: lastSuccessfulRefreshAt
            )
            try? await self.cache.saveCachedAggregate(cached)
        }

        if let widgetSnapshotCoordinator {
            let widgetSnapshot = WidgetSnapshotBuilder.snapshot(
                aggregate: aggregate,
                defaultRefreshIntervalSeconds: self.widgetRefreshIntervalSeconds
            )
            try? widgetSnapshotCoordinator.persist(widgetSnapshot)
        }
    }

    private func rebuildProjection() {
        self.projection = MobileDashboardProjectionBuilder.build(
            aggregate: self.aggregate,
            selectedScope: self.selectedAccountScope,
            aliases: self.aliases,
            compressionPreference: self.compressionPreference
        )

        let validScopes = Set(self.projection.accountOptions.map(\.scope))
        if !validScopes.contains(self.selectedAccountScope) {
            self.selectedAccountScope = .all
            self.projection = MobileDashboardProjectionBuilder.build(
                aggregate: self.aggregate,
                selectedScope: self.selectedAccountScope,
                aliases: self.aliases,
                compressionPreference: self.compressionPreference
            )
        }

        if !self.projection.visibleProviders.contains(self.selectedProvider) {
            self.selectedProvider = self.projection.visibleProviders.first ?? .claude
        }

        self.persistPreferences()
    }

    private func persistPreferences() {
        self.preferencesStore.savePreferences(
            PersistedMobileDashboardPreferences(
                selectedProvider: self.selectedProvider,
                selectedAccountScope: self.selectedAccountScope,
                compressionPreference: self.compressionPreference,
                aliases: self.aliases,
                collapsedSectionIDs: Array(self.collapsedSectionIDs).sorted()
            )
        )
    }

    private static func defaultCollapsedSectionIDs(for preference: MobileCompressionPreference) -> Set<String> {
        switch preference {
        case .compact:
            return [
                "overview.accounts",
                "overview.installations",
                "history.days",
                "freshness.installations",
            ]
        case .expanded:
            return []
        }
    }

    private static let isoFormatter = ISO8601DateFormatter()
}
