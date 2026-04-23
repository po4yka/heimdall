import Foundation
import HeimdallDomain

public struct SnapshotSyncCoordinator: SnapshotSyncing {
    private let client: any MobileSnapshotClient
    private let store: any SnapshotSyncStore

    public init(
        client: any MobileSnapshotClient,
        store: any SnapshotSyncStore
    ) {
        self.client = client
        self.store = store
    }

    public func syncLatestSnapshot() async throws -> SyncedAggregateEnvelope {
        let snapshot = try await self.client.fetchMobileSnapshot()
        return try await self.store.saveLatestSnapshot(snapshot)
    }

    public func loadCloudSyncSpaceState() async throws -> CloudSyncSpaceState {
        try await self.store.loadCloudSyncSpaceState()
    }
}
