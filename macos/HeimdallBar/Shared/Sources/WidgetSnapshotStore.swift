import Foundation

public enum WidgetSnapshotStoreError: Error, LocalizedError, Equatable, Sendable {
    case appGroupUnavailable
    case snapshotMissing
    case readFailed(String)
    case decodeFailed(String)
    case schemaMismatch(Int)
    case validationFailed(String)
    case writeFailed(String)

    public var errorDescription: String? {
        switch self {
        case .appGroupUnavailable:
            return "Widget shared storage is unavailable."
        case .snapshotMissing:
            return "No widget snapshot has been written yet."
        case .readFailed(let detail):
            return "Failed to read widget snapshot: \(detail)"
        case .decodeFailed(let detail):
            return "Failed to decode widget snapshot: \(detail)"
        case .schemaMismatch(let version):
            return "Widget snapshot schema \(version) is not supported."
        case .validationFailed(let detail):
            return "Widget snapshot validation failed: \(detail)"
        case .writeFailed(let detail):
            return "Failed to write widget snapshot: \(detail)"
        }
    }
}

public enum WidgetSnapshotLoadResult: Sendable {
    case success(WidgetSnapshot)
    case empty
    case failure(WidgetSnapshotStoreError)
}

public enum WidgetSnapshotSaveResult: Equatable, Sendable {
    case saved
    case unchanged
}

public enum WidgetSnapshotStore {
    public static let appGroupID = "group.dev.heimdall.heimdallbar"
    private static let filename = "widget-snapshot.json"

    public static func save(
        _ snapshot: WidgetSnapshot,
        baseURLOverride: URL? = nil
    ) throws -> WidgetSnapshotSaveResult {
        let data = try self.encoded(snapshot)
        let url = try self.snapshotURL(createDirectory: true, baseURLOverride: baseURLOverride)
        if let existing = try? Data(contentsOf: url), existing == data {
            return .unchanged
        }
        do {
            try data.write(to: url, options: .atomic)
            return .saved
        } catch {
            throw WidgetSnapshotStoreError.writeFailed(error.localizedDescription)
        }
    }

    public static func load(baseURLOverride: URL? = nil) -> WidgetSnapshotLoadResult {
        let url: URL
        do {
            url = try self.snapshotURL(createDirectory: false, baseURLOverride: baseURLOverride)
        } catch let error as WidgetSnapshotStoreError {
            return .failure(error)
        } catch {
            return .failure(.readFailed(error.localizedDescription))
        }

        guard FileManager.default.fileExists(atPath: url.path) else {
            return .empty
        }

        let data: Data
        do {
            data = try Data(contentsOf: url)
        } catch {
            return .failure(.readFailed(error.localizedDescription))
        }

        do {
            let snapshot = try self.decoded(data)
            return .success(snapshot)
        } catch let error as WidgetSnapshotStoreError {
            return .failure(error)
        } catch {
            return .failure(.decodeFailed(error.localizedDescription))
        }
    }

    public static func snapshotURL(
        createDirectory: Bool,
        baseURLOverride: URL? = nil
    ) throws -> URL {
        let fm = FileManager.default
        let base: URL
        if let baseURLOverride {
            base = baseURLOverride
        } else if let container = fm.containerURL(forSecurityApplicationGroupIdentifier: self.appGroupID) {
            base = container
        } else {
            #if DEBUG
            base = fm.homeDirectoryForCurrentUser
                .appendingPathComponent("Library", isDirectory: true)
                .appendingPathComponent("Application Support", isDirectory: true)
                .appendingPathComponent("HeimdallBar", isDirectory: true)
            #else
            throw WidgetSnapshotStoreError.appGroupUnavailable
            #endif
        }

        if createDirectory {
            do {
                try fm.createDirectory(at: base, withIntermediateDirectories: true)
            } catch {
                throw WidgetSnapshotStoreError.writeFailed(error.localizedDescription)
            }
        }
        return base.appendingPathComponent(self.filename, isDirectory: false)
    }

    private static func encoded(_ snapshot: WidgetSnapshot) throws -> Data {
        try self.validate(snapshot)
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let data = try encoder.encode(snapshot)
        _ = try self.decoded(data)
        return data
    }

    private static func decoded(_ data: Data) throws -> WidgetSnapshot {
        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        do {
            let snapshot = try decoder.decode(WidgetSnapshot.self, from: data)
            try self.validate(snapshot)
            return snapshot
        } catch let error as WidgetSnapshotStoreError {
            throw error
        } catch {
            throw WidgetSnapshotStoreError.decodeFailed(error.localizedDescription)
        }
    }

    private static func validate(_ snapshot: WidgetSnapshot) throws {
        guard snapshot.schemaVersion == WidgetSnapshot.currentSchemaVersion else {
            throw WidgetSnapshotStoreError.schemaMismatch(snapshot.schemaVersion)
        }
        if snapshot.generatedAt.isEmpty {
            throw WidgetSnapshotStoreError.validationFailed("generatedAt is empty")
        }
        if snapshot.defaultRefreshIntervalSeconds <= 0 {
            throw WidgetSnapshotStoreError.validationFailed("defaultRefreshIntervalSeconds must be positive")
        }
        for provider in snapshot.providers.values {
            guard provider.provider.rawValue == provider.id else {
                throw WidgetSnapshotStoreError.validationFailed("provider id mismatch for \(provider.provider.rawValue)")
            }
        }
    }
}
