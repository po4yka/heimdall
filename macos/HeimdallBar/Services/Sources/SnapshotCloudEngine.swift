import CloudKit
import Foundation
import HeimdallDomain
import os

private let snapshotCloudEngineLogger = Logger(subsystem: "dev.heimdall.heimdallbar", category: "CloudKit")

public actor SnapshotCloudEngine: CloudSnapshotBackingStore, CKSyncEngineDelegate {
    public static let zoneName = "heimdall-sync-space"
    public static let legacyRecordType = "MobileSnapshot"
    public static let legacyRecordName = "latest"

    private let container: CKContainer
    private let privateDatabase: CKDatabase
    private let sharedDatabase: CKDatabase
    private let stateStore: any CloudKitSyncEngineStatePersisting
    private let zoneID: CKRecordZone.ID
    private let encoder: JSONEncoder
    private let decoder: JSONDecoder

    private var engine: CKSyncEngine?
    private var cachedSnapshots: [String: SyncedInstallationSnapshot] = [:]
    private var pendingRecords: [String: SnapshotCloudRecord] = [:]
    private var systemFields: [String: Data] = [:]

    public init(
        containerIdentifier: String,
        stateStore: any CloudKitSyncEngineStatePersisting
    ) {
        let container = CKContainer(identifier: containerIdentifier)
        self.container = container
        self.privateDatabase = container.privateCloudDatabase
        self.sharedDatabase = container.sharedCloudDatabase
        self.stateStore = stateStore
        self.zoneID = CKRecordZone.ID(zoneName: Self.zoneName, ownerName: CKCurrentUserDefaultName)
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.sortedKeys]
        self.encoder = encoder
        self.decoder = JSONDecoder()
        snapshotCloudEngineLogger.info("SnapshotCloudEngine created for container \(containerIdentifier, privacy: .public) zone \(Self.zoneName, privacy: .public)")
    }

    public func accountStatus() async throws -> CKAccountStatus {
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

    public func loadLegacySnapshot() async throws -> MobileSnapshotEnvelope? {
        let recordID = CKRecord.ID(recordName: Self.legacyRecordName)
        do {
            let record = try await self.privateDatabase.record(for: recordID)
            let payload: Data?
            if let encrypted = record.encryptedValues[SnapshotCloudRecord.Field.payload] as? Data {
                payload = encrypted
            } else if let plain = record[SnapshotCloudRecord.Field.payload] as? Data {
                payload = plain
            } else {
                payload = nil
            }
            guard let payload else { return nil }
            do {
                return try self.decoder.decode(MobileSnapshotEnvelope.self, from: payload)
            } catch {
                throw SnapshotSyncStoreError.decodeFailed(error.localizedDescription)
            }
        } catch let error as CKError where error.code == .unknownItem {
            return nil
        } catch {
            throw SnapshotSyncStoreError.transportFailed(error.localizedDescription)
        }
    }

    public func saveLegacySnapshot(_ snapshot: MobileSnapshotEnvelope) async throws {
        let payload: Data
        do {
            payload = try self.encoder.encode(snapshot)
        } catch {
            throw SnapshotSyncStoreError.encodeFailed(error.localizedDescription)
        }
        let recordID = CKRecord.ID(recordName: Self.legacyRecordName)
        let record = CKRecord(recordType: Self.legacyRecordType, recordID: recordID)
        record.encryptedValues[SnapshotCloudRecord.Field.payload] = payload as CKRecordValue
        record[SnapshotCloudRecord.Field.contractVersion] = NSNumber(value: snapshot.contractVersion)
        record["generatedAt"] = snapshot.generatedAt as CKRecordValue
        record[SnapshotCloudRecord.Field.sourceDevice] = snapshot.sourceDevice as CKRecordValue
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

    public func fetchInstallationSnapshots(state: CloudSyncSpaceState) async throws -> [SyncedInstallationSnapshot] {
        if state.role == .participant,
           state.status == .participantJoined,
           let zoneName = state.zoneName,
           let zoneOwnerName = state.zoneOwnerName {
            let sharedZoneID = CKRecordZone.ID(zoneName: zoneName, ownerName: zoneOwnerName)
            snapshotCloudEngineLogger.debug("fetch via shared database zone=\(zoneName, privacy: .public)")
            return try await self.fetchSharedZoneSnapshots(zoneID: sharedZoneID)
        }
        let engine = try await self.requireEngine()
        snapshotCloudEngineLogger.debug("fetch via CKSyncEngine.fetchChanges")
        do {
            try await engine.fetchChanges()
        } catch {
            Self.logCKError("fetchChanges failed", error)
            throw SnapshotSyncStoreError.transportFailed(error.localizedDescription)
        }
        snapshotCloudEngineLogger.debug("fetch complete cached=\(self.cachedSnapshots.count)")
        return self.cachedSnapshots.values.sorted(by: { $0.publishedAt > $1.publishedAt })
    }

    public func saveInstallationSnapshot(
        _ snapshot: SyncedInstallationSnapshot,
        state: CloudSyncSpaceState
    ) async throws -> CloudSyncSpaceState {
        let payload: Data
        do {
            payload = try self.encoder.encode(snapshot)
        } catch {
            throw SnapshotSyncStoreError.encodeFailed(error.localizedDescription)
        }

        if state.role == .participant,
           state.status == .participantJoined,
           let zoneName = state.zoneName,
           let zoneOwnerName = state.zoneOwnerName {
            let sharedZoneID = CKRecordZone.ID(zoneName: zoneName, ownerName: zoneOwnerName)
            try await self.saveSharedZoneSnapshot(snapshot: snapshot, payload: payload, zoneID: sharedZoneID)
            var nextState = state
            nextState.lastPublishedAt = snapshot.publishedAt
            return nextState
        }

        let record = SnapshotCloudRecord(
            installationID: snapshot.installationID,
            sourceDevice: snapshot.sourceDevice,
            publishedAt: snapshot.publishedAt,
            payload: payload,
            systemFieldsData: self.systemFields[snapshot.installationID]
        )
        self.pendingRecords[snapshot.installationID] = record
        self.cachedSnapshots[snapshot.installationID] = snapshot

        let engine = try await self.requireEngine()
        let recordID = CKRecord.ID(recordName: snapshot.installationID, zoneID: self.zoneID)
        engine.state.add(pendingRecordZoneChanges: [.saveRecord(recordID)])
        snapshotCloudEngineLogger.debug("queued save installation=\(snapshot.installationID, privacy: .public) pending=\(self.pendingRecords.count)")
        do {
            try await engine.sendChanges()
        } catch {
            Self.logCKError("sendChanges failed for \(snapshot.installationID)", error)
            throw SnapshotSyncStoreError.transportFailed(error.localizedDescription)
        }

        var nextState = state
        if nextState.role != .participant {
            nextState.role = .owner
            nextState.zoneName = self.zoneID.zoneName
            nextState.zoneOwnerName = self.zoneID.ownerName
            if nextState.status == .notConfigured {
                nextState.status = .ownerReady
            }
        }
        nextState.lastPublishedAt = snapshot.publishedAt
        return nextState
    }

    public func prepareOwnerShare(state: CloudSyncSpaceState) async throws -> CloudSyncSpaceState {
        let zoneID = try await self.ensurePrivateZone(zoneName: state.zoneName ?? Self.zoneName)
        let existingShare = try await self.fetchExistingZoneShare(zoneID: zoneID)

        let share: CKShare
        if let existingShare {
            share = existingShare
        } else {
            let newShare = CKShare(recordZoneID: zoneID)
            newShare.publicPermission = .readOnly
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

    public func acceptShareURL(_ url: URL, state: CloudSyncSpaceState) async throws -> CloudSyncSpaceState {
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

    public func handleRemoteNotification(_ userInfo: [AnyHashable: Any]) async {
        _ = userInfo
        guard let engine = try? await self.requireEngine() else { return }
        try? await engine.fetchChanges()
    }

    // MARK: - CKSyncEngineDelegate

    public func handleEvent(_ event: CKSyncEngine.Event, syncEngine: CKSyncEngine) async {
        _ = syncEngine
        switch event {
        case .stateUpdate(let update):
            snapshotCloudEngineLogger.debug("event: stateUpdate")
            await self.persist(stateSerialization: update.stateSerialization)
        case .fetchedRecordZoneChanges(let fetched):
            snapshotCloudEngineLogger.debug("event: fetchedRecordZoneChanges modifications=\(fetched.modifications.count) deletions=\(fetched.deletions.count)")
            self.applyFetched(fetched)
        case .sentRecordZoneChanges(let sent):
            snapshotCloudEngineLogger.debug("event: sentRecordZoneChanges saved=\(sent.savedRecords.count) failedSaves=\(sent.failedRecordSaves.count)")
            self.applySent(sent)
        case .accountChange:
            snapshotCloudEngineLogger.info("event: accountChange — clearing local sync state")
            await self.clearLocalState()
        case .fetchedDatabaseChanges(let fetched):
            snapshotCloudEngineLogger.debug("event: fetchedDatabaseChanges deletions=\(fetched.deletions.count)")
            self.applyDatabaseChanges(fetched)
        case .sentDatabaseChanges,
             .willFetchChanges,
             .willFetchRecordZoneChanges,
             .didFetchRecordZoneChanges,
             .didFetchChanges,
             .willSendChanges,
             .didSendChanges:
            break
        @unknown default:
            break
        }
    }

    public func nextRecordZoneChangeBatch(
        _ context: CKSyncEngine.SendChangesContext,
        syncEngine: CKSyncEngine
    ) async -> CKSyncEngine.RecordZoneChangeBatch? {
        _ = syncEngine
        _ = context
        return self.buildBatch()
    }

    // MARK: - Raw CloudKit helpers (shared zone reads, legacy, sharing)

    private func fetchSharedZoneSnapshots(zoneID: CKRecordZone.ID) async throws -> [SyncedInstallationSnapshot] {
        do {
            let records = try await self.allRecords(
                recordType: SnapshotCloudRecord.recordType,
                database: self.sharedDatabase,
                zoneID: zoneID
            )
            return try records.compactMap { record in
                let payload: Data?
                if let encrypted = record.encryptedValues[SnapshotCloudRecord.Field.payload] as? Data {
                    payload = encrypted
                } else if let plain = record[SnapshotCloudRecord.Field.payload] as? Data {
                    payload = plain
                } else {
                    payload = nil
                }
                guard let payload else { return nil }
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

    private func saveSharedZoneSnapshot(
        snapshot: SyncedInstallationSnapshot,
        payload: Data,
        zoneID: CKRecordZone.ID
    ) async throws {
        let recordID = CKRecord.ID(recordName: snapshot.installationID, zoneID: zoneID)
        let record = CKRecord(recordType: SnapshotCloudRecord.recordType, recordID: recordID)
        record.encryptedValues[SnapshotCloudRecord.Field.payload] = payload as CKRecordValue
        record[SnapshotCloudRecord.Field.installationID] = snapshot.installationID as CKRecordValue
        record[SnapshotCloudRecord.Field.sourceDevice] = snapshot.sourceDevice as CKRecordValue
        record[SnapshotCloudRecord.Field.publishedAt] = snapshot.publishedAt as CKRecordValue
        do {
            _ = try await self.sharedDatabase.modifyRecords(
                saving: [record],
                deleting: [],
                savePolicy: .changedKeys,
                atomically: true
            )
        } catch {
            throw SnapshotSyncStoreError.transportFailed(error.localizedDescription)
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

    // MARK: - Event handlers

    private func buildBatch() -> CKSyncEngine.RecordZoneChangeBatch? {
        guard !self.pendingRecords.isEmpty else { return nil }
        let records = self.pendingRecords.values.map { $0.toCKRecord(zoneID: self.zoneID) }
        return CKSyncEngine.RecordZoneChangeBatch(
            recordsToSave: records,
            recordIDsToDelete: [],
            atomicByZone: false
        )
    }

    private func persist(stateSerialization: CKSyncEngine.State.Serialization) async {
        try? await self.stateStore.saveState(stateSerialization)
    }

    private func applyFetched(_ fetched: CKSyncEngine.Event.FetchedRecordZoneChanges) {
        for modification in fetched.modifications {
            let record = modification.record
            guard record.recordType == SnapshotCloudRecord.recordType else { continue }
            guard let snapshotRecord = SnapshotCloudRecord.from(ckRecord: record) else { continue }
            self.systemFields[snapshotRecord.installationID] = snapshotRecord.systemFieldsData
            if let snapshot = try? self.decoder.decode(
                SyncedInstallationSnapshot.self,
                from: snapshotRecord.payload
            ) {
                self.cachedSnapshots[snapshotRecord.installationID] = snapshot
            }
        }
        for deletion in fetched.deletions {
            let name = deletion.recordID.recordName
            self.cachedSnapshots.removeValue(forKey: name)
            self.systemFields.removeValue(forKey: name)
        }
    }

    private func applySent(_ sent: CKSyncEngine.Event.SentRecordZoneChanges) {
        for saved in sent.savedRecords {
            let name = saved.recordID.recordName
            self.systemFields[name] = SnapshotCloudRecord.encodeSystemFields(of: saved)
            self.pendingRecords.removeValue(forKey: name)
        }
        for failed in sent.failedRecordSaves {
            snapshotCloudEngineLogger.error("save failed recordName=\(failed.record.recordID.recordName, privacy: .public) code=\(failed.error.code.rawValue, privacy: .public)")
        }
    }

    static func logCKError(_ message: String, _ error: Error) {
        if let ckError = error as? CKError {
            snapshotCloudEngineLogger.error("\(message, privacy: .public) CKError code=\(ckError.code.rawValue, privacy: .public)")
        } else {
            snapshotCloudEngineLogger.error("\(message, privacy: .public) error=\(String(describing: error), privacy: .public)")
        }
    }

    private func applyDatabaseChanges(_ fetched: CKSyncEngine.Event.FetchedDatabaseChanges) {
        for deletion in fetched.deletions where deletion.zoneID == self.zoneID {
            self.cachedSnapshots.removeAll()
            self.systemFields.removeAll()
            self.pendingRecords.removeAll()
        }
    }

    private func clearLocalState() async {
        self.cachedSnapshots.removeAll()
        self.pendingRecords.removeAll()
        self.systemFields.removeAll()
        try? await self.stateStore.purgeState()
        self.engine = nil
    }

    private func requireEngine() async throws -> CKSyncEngine {
        if let engine = self.engine {
            return engine
        }
        let state = try? await self.stateStore.loadState()
        let configuration = CKSyncEngine.Configuration(
            database: self.privateDatabase,
            stateSerialization: state,
            delegate: self
        )
        let engine = CKSyncEngine(configuration)
        self.engine = engine
        return engine
    }
}
