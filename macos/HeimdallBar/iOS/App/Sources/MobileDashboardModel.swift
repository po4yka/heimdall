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

@MainActor
@Observable
final class MobileDashboardModel {
    var aggregate: SyncedAggregateEnvelope?
    var isLoading = false
    var lastRefreshError: String?
    var selectedProvider: ProviderID = .claude
    var cloudSyncState = CloudSyncSpaceState()
    var lastSuccessfulRefreshAt: String?

    private let store: any SnapshotSyncStore
    private let cache: any SyncedAggregateCaching
    private let widgetSnapshotCoordinator: WidgetSnapshotCoordinator?
    private let now: @Sendable () -> Date
    private let foregroundRefreshThrottle: TimeInterval
    private let widgetRefreshIntervalSeconds: Int

    init(
        store: any SnapshotSyncStore,
        cache: any SyncedAggregateCaching = NoopSyncedAggregateCache(),
        widgetSnapshotCoordinator: WidgetSnapshotCoordinator? = nil,
        now: @escaping @Sendable () -> Date = Date.init,
        foregroundRefreshThrottle: TimeInterval = 60,
        widgetRefreshIntervalSeconds: Int = 900
    ) {
        self.store = store
        self.cache = cache
        self.widgetSnapshotCoordinator = widgetSnapshotCoordinator
        self.now = now
        self.foregroundRefreshThrottle = foregroundRefreshThrottle
        self.widgetRefreshIntervalSeconds = widgetRefreshIntervalSeconds
    }

    var providerSnapshots: [ProviderSnapshot] {
        self.aggregate?.aggregateProviderViews.map(\.providerSnapshot) ?? []
    }

    var selectedProviderSnapshot: ProviderSnapshot? {
        self.providerSnapshots.first(where: { $0.providerID == self.selectedProvider })
            ?? self.providerSnapshots.first
    }

    var selectedHistorySeries: MobileProviderHistorySeries? {
        self.aggregate?.aggregateHistorySeries(for: self.selectedProvider)
            ?? self.aggregate?.aggregateHistory90d().first
    }

    var installations: [SyncedInstallationSnapshot] {
        self.aggregate?.installations ?? []
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
        self.aggregate?.installations.map(\.publishedAt).max()
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

            if let fallbackAggregate = try await self.store.loadAggregateSnapshot() {
                await self.applyFreshAggregate(fallbackAggregate)
                return
            }

            self.aggregate = nil
            self.lastRefreshError = nil
        } catch {
            let message = error.localizedDescription
            if await self.restoreCachedAggregate(errorMessage: message) {
                return
            }
            if let fallbackAggregate = try? await self.store.loadAggregateSnapshot() {
                await self.applyFreshAggregate(fallbackAggregate)
                self.lastRefreshError = message
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
        self.syncSelectedProvider()
    }

    private func restoreCachedAggregate(errorMessage: String?) async -> Bool {
        guard let cached = try? await self.cache.loadCachedAggregate() else {
            return false
        }
        self.aggregate = cached.aggregate
        self.lastSuccessfulRefreshAt = cached.lastSuccessfulRefreshAt
        self.lastRefreshError = errorMessage
        self.syncSelectedProvider()
        return true
    }

    private func applyFreshAggregate(_ aggregate: SyncedAggregateEnvelope) async {
        self.aggregate = aggregate
        self.lastRefreshError = nil
        self.lastSuccessfulRefreshAt = Self.isoFormatter.string(from: self.now())
        self.syncSelectedProvider()

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

    private func syncSelectedProvider() {
        guard let aggregate else { return }
        let availableProviders = aggregate.aggregateProviderViews.compactMap(\.providerID)
        if availableProviders.contains(self.selectedProvider) {
            return
        }
        self.selectedProvider = availableProviders.first ?? .claude
    }

    private static let isoFormatter = ISO8601DateFormatter()
}

actor FileBackedSyncedAggregateCache: SyncedAggregateCaching {
    private static let filename = "mobile-synced-aggregate.json"
    private let baseURLOverride: URL?
    private let encoder: JSONEncoder
    private let decoder: JSONDecoder

    init(baseURLOverride: URL? = nil) {
        self.baseURLOverride = baseURLOverride
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        self.encoder = encoder
        self.decoder = JSONDecoder()
    }

    func loadCachedAggregate() async throws -> CachedSyncedAggregateEnvelope? {
        let url = try self.cacheURL(createDirectory: false)
        guard FileManager.default.fileExists(atPath: url.path) else {
            return nil
        }
        let data = try Data(contentsOf: url)
        return try self.decoder.decode(CachedSyncedAggregateEnvelope.self, from: data)
    }

    func saveCachedAggregate(_ cached: CachedSyncedAggregateEnvelope) async throws {
        let url = try self.cacheURL(createDirectory: true)
        let data = try self.encoder.encode(cached)
        try data.write(to: url, options: .atomic)
    }

    private func cacheURL(createDirectory: Bool) throws -> URL {
        let baseURL = self.baseURLOverride ?? (
            FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first
            ?? FileManager.default.temporaryDirectory
        )
        let directory = baseURL
            .appendingPathComponent("HeimdallMobile", isDirectory: true)
        if createDirectory {
            try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        }
        return directory.appendingPathComponent(Self.filename, isDirectory: false)
    }
}

private actor NoopSyncedAggregateCache: SyncedAggregateCaching {
    func loadCachedAggregate() async throws -> CachedSyncedAggregateEnvelope? {
        nil
    }

    func saveCachedAggregate(_: CachedSyncedAggregateEnvelope) async throws {}
}
