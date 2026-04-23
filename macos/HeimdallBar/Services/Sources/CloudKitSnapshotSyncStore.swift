import CloudKit
import Foundation
import HeimdallDomain

public enum SnapshotSyncStoreError: Error, LocalizedError, Equatable, Sendable {
    case missingPayload
    case encodeFailed(String)
    case decodeFailed(String)
    case transportFailed(String)

    public var errorDescription: String? {
        switch self {
        case .missingPayload:
            return "No synced mobile snapshot payload was found."
        case .encodeFailed(let detail):
            return "Failed to encode mobile snapshot payload: \(detail)"
        case .decodeFailed(let detail):
            return "Failed to decode mobile snapshot payload: \(detail)"
        case .transportFailed(let detail):
            return "CloudKit snapshot sync failed: \(detail)"
        }
    }
}

protocol CloudSnapshotBackingStore: Sendable {
    func accountStatus() async throws -> CKAccountStatus
    func loadLegacySnapshot() async throws -> MobileSnapshotEnvelope?
    func saveLegacySnapshot(_ snapshot: MobileSnapshotEnvelope) async throws
    func fetchInstallationSnapshots(state: CloudSyncSpaceState) async throws -> [SyncedInstallationSnapshot]
    func saveInstallationSnapshot(
        _ snapshot: SyncedInstallationSnapshot,
        state: CloudSyncSpaceState
    ) async throws -> CloudSyncSpaceState
    func prepareOwnerShare(state: CloudSyncSpaceState) async throws -> CloudSyncSpaceState
    func acceptShareURL(_ url: URL, state: CloudSyncSpaceState) async throws -> CloudSyncSpaceState
}

struct CloudKitSnapshotBackingStore: CloudSnapshotBackingStore {
    static let legacyRecordType = "MobileSnapshot"
    static let legacyRecordName = "latest"
    static let installationRecordType = "SyncedInstallationSnapshot"
    static let zoneName = "heimdall-sync-space"

    private let container: CKContainer
    private let privateDatabase: CKDatabase
    private let sharedDatabase: CKDatabase
    private let encoder: JSONEncoder
    private let decoder: JSONDecoder

    init(containerIdentifier: String) {
        let container = CKContainer(identifier: containerIdentifier)
        self.container = container
        self.privateDatabase = container.privateCloudDatabase
        self.sharedDatabase = container.sharedCloudDatabase
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.sortedKeys]
        self.encoder = encoder
        self.decoder = JSONDecoder()
    }

    func accountStatus() async throws -> CKAccountStatus {
        try await withCheckedThrowingContinuation { continuation in
            self.container.accountStatus { status, error in
                if let error {
                    continuation.resume(throwing: error)
                } else {
                    continuation.resume(returning: status)
                }
            }
        }
    }

    func loadLegacySnapshot() async throws -> MobileSnapshotEnvelope? {
        let recordID = CKRecord.ID(recordName: Self.legacyRecordName)
        guard let record = try await self.recordIfPresent(recordID: recordID, database: self.privateDatabase),
              let payload = record["payload"] as? Data else {
            return nil
        }
        do {
            return try self.decoder.decode(MobileSnapshotEnvelope.self, from: payload)
        } catch {
            throw SnapshotSyncStoreError.decodeFailed(error.localizedDescription)
        }
    }

    func saveLegacySnapshot(_ snapshot: MobileSnapshotEnvelope) async throws {
        let payload: Data
        do {
            payload = try self.encoder.encode(snapshot)
        } catch {
            throw SnapshotSyncStoreError.encodeFailed(error.localizedDescription)
        }

        let recordID = CKRecord.ID(recordName: Self.legacyRecordName)
        let record = CKRecord(recordType: Self.legacyRecordType, recordID: recordID)
        record["payload"] = payload as CKRecordValue
        record["contractVersion"] = NSNumber(value: snapshot.contractVersion)
        record["generatedAt"] = snapshot.generatedAt as CKRecordValue
        record["sourceDevice"] = snapshot.sourceDevice as CKRecordValue
        do {
            _ = try await self.privateDatabase.modifyRecords(
                saving: [record],
                deleting: [],
                savePolicy: .changedKeys,
                atomically: true
            )
        } catch {
            throw SnapshotSyncStoreError.transportFailed(error.localizedDescription)
        }
    }

    func fetchInstallationSnapshots(state: CloudSyncSpaceState) async throws -> [SyncedInstallationSnapshot] {
        guard let (database, zoneID) = self.databaseAndZone(for: state) else {
            return []
        }

        do {
            let records = try await self.allRecords(
                recordType: Self.installationRecordType,
                database: database,
                zoneID: zoneID
            )
            return try records.compactMap { record in
                guard let payload = record["payload"] as? Data else { return nil }
                do {
                    return try self.decoder.decode(SyncedInstallationSnapshot.self, from: payload)
                } catch {
                    throw SnapshotSyncStoreError.decodeFailed(error.localizedDescription)
                }
            }
        } catch let error as CKError where error.code == .zoneNotFound || error.code == .unknownItem {
            return []
        } catch {
            throw SnapshotSyncStoreError.transportFailed(error.localizedDescription)
        }
    }

    func saveInstallationSnapshot(
        _ snapshot: SyncedInstallationSnapshot,
        state: CloudSyncSpaceState
    ) async throws -> CloudSyncSpaceState {
        let payload: Data
        do {
            payload = try self.encoder.encode(snapshot)
        } catch {
            throw SnapshotSyncStoreError.encodeFailed(error.localizedDescription)
        }

        var nextState = state
        let target: (database: CKDatabase, zoneID: CKRecordZone.ID)
        if state.role == .participant,
           state.status == .participantJoined,
           let zoneName = state.zoneName,
           let zoneOwnerName = state.zoneOwnerName {
            target = (self.sharedDatabase, CKRecordZone.ID(zoneName: zoneName, ownerName: zoneOwnerName))
        } else {
            let zoneID = try await self.ensurePrivateZone(zoneName: state.zoneName ?? Self.zoneName)
            target = (self.privateDatabase, zoneID)
            nextState.role = .owner
            nextState.zoneName = zoneID.zoneName
            nextState.zoneOwnerName = zoneID.ownerName
            if nextState.status == .notConfigured {
                nextState.status = .ownerReady
            }
        }

        let recordID = CKRecord.ID(recordName: snapshot.installationID, zoneID: target.zoneID)
        let record = CKRecord(recordType: Self.installationRecordType, recordID: recordID)
        record["payload"] = payload as CKRecordValue
        record["installationID"] = snapshot.installationID as CKRecordValue
        record["sourceDevice"] = snapshot.sourceDevice as CKRecordValue
        record["publishedAt"] = snapshot.publishedAt as CKRecordValue

        do {
            _ = try await target.database.modifyRecords(
                saving: [record],
                deleting: [],
                savePolicy: .changedKeys,
                atomically: true
            )
        } catch {
            throw SnapshotSyncStoreError.transportFailed(error.localizedDescription)
        }

        nextState.lastPublishedAt = snapshot.publishedAt
        return nextState
    }

    func prepareOwnerShare(state: CloudSyncSpaceState) async throws -> CloudSyncSpaceState {
        let zoneID = try await self.ensurePrivateZone(zoneName: state.zoneName ?? Self.zoneName)
        let existingShare = try await self.fetchExistingZoneShare(zoneID: zoneID)

        let share: CKShare
        if let existingShare {
            share = existingShare
        } else {
            let newShare = CKShare(recordZoneID: zoneID)
            newShare.publicPermission = .readWrite
            newShare[CKShare.SystemFieldKey.title] = "Heimdall Sync" as CKRecordValue
            do {
                let result = try await self.privateDatabase.modifyRecords(
                    saving: [newShare],
                    deleting: [],
                    savePolicy: .ifServerRecordUnchanged,
                    atomically: true
                )
                if case .success(let savedShare as CKShare)? = result.saveResults[newShare.recordID] {
                    share = savedShare
                } else {
                    share = newShare
                }
            } catch {
                throw SnapshotSyncStoreError.transportFailed(error.localizedDescription)
            }
        }

        return CloudSyncSpaceState(
            role: .owner,
            status: share.url == nil ? .ownerReady : .inviteReady,
            shareURL: share.url?.absoluteString,
            zoneName: zoneID.zoneName,
            zoneOwnerName: zoneID.ownerName,
            lastPublishedAt: state.lastPublishedAt,
            lastAcceptedAt: state.lastAcceptedAt,
            statusMessage: share.url == nil ? "CloudKit zone ready. Create a share from this Mac once iCloud finishes provisioning." : "Share link ready to copy to another Mac."
        )
    }

    func acceptShareURL(_ url: URL, state: CloudSyncSpaceState) async throws -> CloudSyncSpaceState {
        do {
            let metadataResults = try await self.container.shareMetadatas(for: [url])
            guard case .success(let metadata)? = metadataResults[url] else {
                throw SnapshotSyncStoreError.transportFailed("The CloudKit share metadata could not be loaded.")
            }
            _ = try await self.container.accept([metadata])
            let zoneID = (metadata.hierarchicalRootRecordID ?? metadata.rootRecordID).zoneID
            return CloudSyncSpaceState(
                role: .participant,
                status: .participantJoined,
                shareURL: url.absoluteString,
                zoneName: zoneID.zoneName,
                zoneOwnerName: zoneID.ownerName,
                lastPublishedAt: state.lastPublishedAt,
                lastAcceptedAt: ISO8601DateFormatter().string(from: Date()),
                statusMessage: "Joined the shared Heimdall sync space."
            )
        } catch {
            throw SnapshotSyncStoreError.transportFailed(error.localizedDescription)
        }
    }

    private func databaseAndZone(for state: CloudSyncSpaceState) -> (CKDatabase, CKRecordZone.ID)? {
        switch state.role {
        case .participant:
            guard state.status == .participantJoined,
                  let zoneName = state.zoneName,
                  let zoneOwnerName = state.zoneOwnerName else {
                return nil
            }
            return (self.sharedDatabase, CKRecordZone.ID(zoneName: zoneName, ownerName: zoneOwnerName))
        case .owner:
            let zoneName = state.zoneName ?? Self.zoneName
            return (self.privateDatabase, CKRecordZone.ID(zoneName: zoneName, ownerName: CKCurrentUserDefaultName))
        case .none:
            return nil
        }
    }

    private func ensurePrivateZone(zoneName: String) async throws -> CKRecordZone.ID {
        let zoneID = CKRecordZone.ID(zoneName: zoneName, ownerName: CKCurrentUserDefaultName)
        do {
            let existing = try await self.privateDatabase.recordZones(for: [zoneID])
            if case .success = existing[zoneID] {
                return zoneID
            }
        } catch {
            // Fall through to create the zone.
        }

        do {
            _ = try await self.privateDatabase.modifyRecordZones(
                saving: [CKRecordZone(zoneID: zoneID)],
                deleting: []
            )
            return zoneID
        } catch {
            throw SnapshotSyncStoreError.transportFailed(error.localizedDescription)
        }
    }

    private func fetchExistingZoneShare(zoneID: CKRecordZone.ID) async throws -> CKShare? {
        let records = try await self.allRecords(
            recordType: CKRecord.SystemType.share,
            database: self.privateDatabase,
            zoneID: zoneID
        )
        return records.first as? CKShare
    }

    private func allRecords(
        recordType: CKRecord.RecordType,
        database: CKDatabase,
        zoneID: CKRecordZone.ID
    ) async throws -> [CKRecord] {
        var records: [CKRecord] = []
        var cursor: CKQueryOperation.Cursor?

        repeat {
            let result: (matchResults: [(CKRecord.ID, Result<CKRecord, Error>)], queryCursor: CKQueryOperation.Cursor?)
            if let cursor {
                result = try await database.records(continuingMatchFrom: cursor)
            } else {
                result = try await database.records(
                    matching: CKQuery(recordType: recordType, predicate: NSPredicate(value: true)),
                    inZoneWith: zoneID
                )
            }
            for (_, matchResult) in result.matchResults {
                if case .success(let record) = matchResult {
                    records.append(record)
                }
            }
            cursor = result.queryCursor
        } while cursor != nil

        return records
    }

    private func recordIfPresent(
        recordID: CKRecord.ID,
        database: CKDatabase
    ) async throws -> CKRecord? {
        do {
            return try await database.record(for: recordID)
        } catch let error as CKError where error.code == .unknownItem {
            return nil
        }
    }
}

public struct CloudKitSnapshotSyncStore: SnapshotSyncStore {
    public static let defaultContainerIdentifier = "iCloud.dev.heimdall.heimdallbar"

    private let backingStore: any CloudSnapshotBackingStore
    private let persistence: any CloudSyncStatePersisting

    public init(
        containerIdentifier: String = Self.defaultContainerIdentifier,
        persistence: any CloudSyncStatePersisting = UserDefaultsCloudSyncStateStore()
    ) {
        self.init(
            backingStore: CloudKitSnapshotBackingStore(containerIdentifier: containerIdentifier),
            persistence: persistence
        )
    }

    init(
        backingStore: any CloudSnapshotBackingStore,
        persistence: any CloudSyncStatePersisting = UserDefaultsCloudSyncStateStore()
    ) {
        self.backingStore = backingStore
        self.persistence = persistence
    }

    public func loadLegacySnapshot() async throws -> MobileSnapshotEnvelope? {
        try await self.backingStore.loadLegacySnapshot()
    }

    public func loadAggregateSnapshot() async throws -> SyncedAggregateEnvelope? {
        let state = try await self.loadCloudSyncSpaceState()
        let installations = try await self.backingStore.fetchInstallationSnapshots(state: state)
        if !installations.isEmpty {
            let generatedAt = installations.map(\.publishedAt).max() ?? ISO8601DateFormatter().string(from: Date())
            return SyncedAggregateEnvelope.aggregate(
                installations: installations,
                generatedAt: generatedAt
            )
        }
        if let legacy = try await self.backingStore.loadLegacySnapshot() {
            return SyncedAggregateEnvelope.legacy(
                mobileSnapshot: legacy,
                installationID: self.installationID
            )
        }
        return nil
    }

    public func saveLatestSnapshot(_ snapshot: MobileSnapshotEnvelope) async throws -> SyncedAggregateEnvelope {
        try await self.backingStore.saveLegacySnapshot(snapshot)
        let installationSnapshot = SyncedInstallationSnapshot.from(
            mobileSnapshot: snapshot,
            installationID: self.installationID
        )
        let state = try await self.loadCloudSyncSpaceState()
        let nextState = try await self.backingStore.saveInstallationSnapshot(
            installationSnapshot,
            state: state
        )
        self.persistence.saveCloudSyncSpaceState(nextState)

        let installations = try await self.backingStore.fetchInstallationSnapshots(state: nextState)
        if !installations.isEmpty {
            return SyncedAggregateEnvelope.aggregate(
                installations: installations,
                generatedAt: snapshot.generatedAt
            )
        }
        return SyncedAggregateEnvelope.legacy(
            mobileSnapshot: snapshot,
            installationID: self.installationID
        )
    }

    public func loadCloudSyncSpaceState() async throws -> CloudSyncSpaceState {
        let persisted = self.persistence.loadCloudSyncSpaceState() ?? CloudSyncSpaceState()
        let status = try await self.backingStore.accountStatus()
        switch status {
        case .available:
            return persisted
        case .restricted:
            return CloudSyncSpaceState(
                role: persisted.role,
                status: .sharingBlocked,
                shareURL: persisted.shareURL,
                zoneName: persisted.zoneName,
                zoneOwnerName: persisted.zoneOwnerName,
                lastPublishedAt: persisted.lastPublishedAt,
                lastAcceptedAt: persisted.lastAcceptedAt,
                statusMessage: "CloudKit sharing is restricted on this device."
            )
        case .noAccount, .couldNotDetermine, .temporarilyUnavailable:
            return CloudSyncSpaceState(
                role: persisted.role,
                status: .iCloudUnavailable,
                shareURL: persisted.shareURL,
                zoneName: persisted.zoneName,
                zoneOwnerName: persisted.zoneOwnerName,
                lastPublishedAt: persisted.lastPublishedAt,
                lastAcceptedAt: persisted.lastAcceptedAt,
                statusMessage: "Sign in to iCloud to enable Cloud Sync."
            )
        @unknown default:
            return persisted
        }
    }

    public func prepareOwnerShare() async throws -> CloudSyncSpaceState {
        let state = try await self.loadCloudSyncSpaceState()
        let nextState = try await self.backingStore.prepareOwnerShare(state: state)
        self.persistence.saveCloudSyncSpaceState(nextState)
        return nextState
    }

    public func acceptShareURL(_ url: URL) async throws -> CloudSyncSpaceState {
        let state = try await self.loadCloudSyncSpaceState()
        let nextState = try await self.backingStore.acceptShareURL(url, state: state)
        self.persistence.saveCloudSyncSpaceState(nextState)
        return nextState
    }

    private var installationID: String {
        if let persisted = self.persistence.loadInstallationID(), !persisted.isEmpty {
            return persisted
        }
        let generated = UUID().uuidString.lowercased()
        self.persistence.saveInstallationID(generated)
        return generated
    }
}
