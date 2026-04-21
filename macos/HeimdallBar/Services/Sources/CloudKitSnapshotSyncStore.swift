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

protocol SnapshotPayloadTransport: Sendable {
    func fetchPayload(recordType: String, recordName: String) async throws -> Data?
    func savePayload(
        _ payload: Data,
        contractVersion: Int,
        generatedAt: String,
        sourceDevice: String,
        recordType: String,
        recordName: String
    ) async throws
}

struct CloudKitSnapshotPayloadTransport: SnapshotPayloadTransport {
    private let database: CKDatabase

    init(containerIdentifier: String) {
        let container = CKContainer(identifier: containerIdentifier)
        self.database = container.privateCloudDatabase
    }

    func fetchPayload(recordType _: String, recordName: String) async throws -> Data? {
        let recordID = CKRecord.ID(recordName: recordName)
        do {
            let record = try await self.database.record(for: recordID)
            return record["payload"] as? Data
        } catch let error as CKError where error.code == .unknownItem {
            return nil
        } catch {
            throw SnapshotSyncStoreError.transportFailed(error.localizedDescription)
        }
    }

    func savePayload(
        _ payload: Data,
        contractVersion: Int,
        generatedAt: String,
        sourceDevice: String,
        recordType: String,
        recordName: String
    ) async throws {
        let recordID = CKRecord.ID(recordName: recordName)
        let record = CKRecord(recordType: recordType, recordID: recordID)
        record["payload"] = payload as CKRecordValue
        record["contractVersion"] = NSNumber(value: contractVersion)
        record["generatedAt"] = generatedAt as CKRecordValue
        record["sourceDevice"] = sourceDevice as CKRecordValue

        do {
            _ = try await self.database.save(record)
        } catch {
            throw SnapshotSyncStoreError.transportFailed(error.localizedDescription)
        }
    }
}

public struct CloudKitSnapshotSyncStore: SnapshotSyncStore {
    public static let defaultContainerIdentifier = "iCloud.dev.heimdall.heimdallbar"

    static let recordType = "MobileSnapshot"
    static let recordName = "latest"

    private let transport: any SnapshotPayloadTransport
    private let encoder: JSONEncoder
    private let decoder: JSONDecoder

    public init(containerIdentifier: String = Self.defaultContainerIdentifier) {
        self.init(transport: CloudKitSnapshotPayloadTransport(containerIdentifier: containerIdentifier))
    }

    init(transport: any SnapshotPayloadTransport) {
        self.transport = transport
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.sortedKeys]
        self.encoder = encoder

        let decoder = JSONDecoder()
        self.decoder = decoder
    }

    public func loadLatestSnapshot() async throws -> MobileSnapshotEnvelope? {
        guard let payload = try await self.transport.fetchPayload(
            recordType: Self.recordType,
            recordName: Self.recordName
        ) else {
            return nil
        }

        do {
            return try self.decoder.decode(MobileSnapshotEnvelope.self, from: payload)
        } catch {
            throw SnapshotSyncStoreError.decodeFailed(error.localizedDescription)
        }
    }

    public func saveLatestSnapshot(_ snapshot: MobileSnapshotEnvelope) async throws {
        let payload: Data
        do {
            payload = try self.encoder.encode(snapshot)
        } catch {
            throw SnapshotSyncStoreError.encodeFailed(error.localizedDescription)
        }

        try await self.transport.savePayload(
            payload,
            contractVersion: snapshot.contractVersion,
            generatedAt: snapshot.generatedAt,
            sourceDevice: snapshot.sourceDevice,
            recordType: Self.recordType,
            recordName: Self.recordName
        )
    }
}
