import Foundation
import Testing
import HeimdallDomain
@testable import HeimdallServices

struct WidgetSnapshotStoreTests {
    @Test
    func snapshotStoreRejectsSchemaMismatch() throws {
        let directory = try Self.makeTempDirectory()
        let snapshot = WidgetSnapshot(
            schemaVersion: 999,
            generatedAt: ISO8601DateFormatter().string(from: Date()),
            defaultRefreshIntervalSeconds: 900,
            providers: [:]
        )

        #expect(throws: WidgetSnapshotStoreError.self) {
            _ = try WidgetSnapshotStore.save(snapshot, baseURLOverride: directory)
        }
    }

    @Test
    func snapshotStoreSkipsUnchangedWrites() throws {
        let directory = try Self.makeTempDirectory()
        let snapshot = WidgetSnapshot(
            generatedAt: ISO8601DateFormatter().string(from: Date()),
            defaultRefreshIntervalSeconds: 900,
            providers: [:]
        )

        let first = try WidgetSnapshotStore.save(snapshot, baseURLOverride: directory)
        let second = try WidgetSnapshotStore.save(snapshot, baseURLOverride: directory)
        let loaded = WidgetSnapshotStore.load(baseURLOverride: directory)

        #expect(first == .saved)
        #expect(second == .unchanged)
        if case .success(let restored) = loaded {
            #expect(restored.schemaVersion == WidgetSnapshot.currentSchemaVersion)
        } else {
            Issue.record("Expected a stored snapshot to load successfully.")
        }
    }

    private static func makeTempDirectory() throws -> URL {
        let base = FileManager.default.temporaryDirectory
        let url = base.appendingPathComponent(UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(at: url, withIntermediateDirectories: true)
        return url
    }
}
