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

    public func syncLatestSnapshot() async throws -> MobileSnapshotEnvelope {
        let snapshot = try await self.client.fetchMobileSnapshot()
        try await self.store.saveLatestSnapshot(snapshot)
        return snapshot
    }
}
