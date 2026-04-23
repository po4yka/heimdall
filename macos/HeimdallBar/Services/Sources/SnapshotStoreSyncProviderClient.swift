import Foundation
import HeimdallDomain

public struct SnapshotStoreSyncProviderClient: SyncProviderClient {
    private let store: any SnapshotSyncStore
    private let clock: @Sendable () -> Date

    public init(
        store: any SnapshotSyncStore,
        clock: @escaping @Sendable () -> Date = Date.init
    ) {
        self.store = store
        self.clock = clock
    }

    public func fetchSyncedSnapshots() async throws -> ProviderSnapshotEnvelope {
        guard let aggregate = try await self.store.loadAggregateSnapshot() else {
            return ProviderSnapshotEnvelope(
                contractVersion: LiveProviderContract.version,
                providers: [],
                fetchedAt: ISO8601DateFormatter().string(from: self.clock()),
                requestedProvider: nil,
                responseScope: "all",
                cacheHit: false,
                refreshedProviders: []
            )
        }
        return ProviderSnapshotEnvelope(
            contractVersion: LiveProviderContract.version,
            providers: aggregate.aggregateProviderViews.map(\.providerSnapshot),
            fetchedAt: aggregate.generatedAt,
            requestedProvider: nil,
            responseScope: "all",
            cacheHit: false,
            refreshedProviders: aggregate.aggregateProviderViews.map(\.provider)
        )
    }
}
