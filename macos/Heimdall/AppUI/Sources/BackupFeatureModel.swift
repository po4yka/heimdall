import Foundation
import Observation

@MainActor
@Observable
public final class BackupFeatureModel {
    public struct SnapshotMeta: Codable, Identifiable, Sendable {
        public let snapshotID: String
        public let createdAt: String
        public let totalFiles: Int
        public let totalBytes: Int
        public var id: String { self.snapshotID }

        enum CodingKeys: String, CodingKey {
            case snapshotID = "snapshot_id"
            case createdAt = "created_at"
            case totalFiles = "total_files"
            case totalBytes = "total_bytes"
        }
    }

    private let helperPort: Int

    public var snapshots: [SnapshotMeta] = []
    public var isLoading = false
    public var isCreating = false
    public var statusMessage: String?

    public init(helperPort: Int) {
        self.helperPort = helperPort
    }

    public func load() async {
        guard !self.isLoading else { return }
        self.isLoading = true
        defer { self.isLoading = false }
        guard let url = URL(string: "http://127.0.0.1:\(self.helperPort)/api/archive") else { return }
        do {
            let (data, _) = try await URLSession.shared.data(from: url)
            self.snapshots = try JSONDecoder().decode([SnapshotMeta].self, from: data)
        } catch {
            // keep stale list visible
        }
    }

    public func createSnapshot() async {
        guard !self.isCreating else { return }
        self.isCreating = true
        self.statusMessage = "Snapshotting..."
        guard let url = URL(string: "http://127.0.0.1:\(self.helperPort)/api/archive/snapshot") else {
            self.isCreating = false
            return
        }
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        do {
            let (_, response) = try await URLSession.shared.data(for: request)
            let ok = (response as? HTTPURLResponse).map { (200..<300).contains($0.statusCode) } ?? false
            self.statusMessage = ok ? "Snapshot created" : "Snapshot failed"
            if ok { await self.load() }
        } catch {
            self.statusMessage = "Error: \(error.localizedDescription)"
        }
        self.isCreating = false
    }

    nonisolated static func formatBytes(_ n: Int) -> String {
        let kb = Double(n) / 1024
        if kb < 1 { return "\(n) B" }
        let mb = kb / 1024
        if mb < 1 { return String(format: "%.1f KB", kb) }
        return String(format: "%.1f MB", mb)
    }
}
