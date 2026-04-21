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
        guard let snapshot = try await self.store.loadLatestSnapshot() else {
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
        return snapshot.providerEnvelope
    }
}
