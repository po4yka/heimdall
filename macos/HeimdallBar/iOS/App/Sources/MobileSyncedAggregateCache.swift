import Foundation
import HeimdallDomain
import HeimdallServices

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

    func purgeCachedAggregate() async throws {
        let url = try self.cacheURL(createDirectory: false)
        guard FileManager.default.fileExists(atPath: url.path) else {
            return
        }
        try FileManager.default.removeItem(at: url)
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

actor NoopSyncedAggregateCache: SyncedAggregateCaching {
    func loadCachedAggregate() async throws -> CachedSyncedAggregateEnvelope? {
        nil
    }

    func saveCachedAggregate(_: CachedSyncedAggregateEnvelope) async throws {}

    func purgeCachedAggregate() async throws {}
}
