import SwiftUI

struct BackupSheet: View {
    @Bindable var model: BackupFeatureModel
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            SheetHeader(title: "Snapshots", onDismiss: { self.dismiss() }) {
                if let msg = self.model.statusMessage {
                    Text(msg)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                Button {
                    Task { await self.model.createSnapshot() }
                } label: {
                    Text("Snapshot now")
                }
                .disabled(self.model.isCreating || self.model.isLoading)
            }

            Divider()

            if self.model.isLoading && self.model.snapshots.isEmpty {
                ProgressView()
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if self.model.snapshots.isEmpty {
                ContentUnavailableView(
                    "No snapshots yet",
                    systemImage: "archivebox",
                    description: Text("Click \"Snapshot now\" to create the first content-addressed snapshot.")
                )
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                Table(self.model.snapshots) {
                    TableColumn("Snapshot") { s in
                        Text(s.snapshotID)
                            .font(.caption.monospaced())
                            .lineLimit(1)
                    }
                    TableColumn("Created") { s in
                        Text(s.createdAt)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                    TableColumn("Files") { s in
                        Text("\(s.totalFiles)")
                            .font(.caption.monospacedDigit())
                    }
                    .width(60)
                    TableColumn("Size") { s in
                        Text(BackupFeatureModel.formatBytes(s.totalBytes))
                            .font(.caption.monospacedDigit())
                    }
                    .width(80)
                }
                .tableStyle(.inset)
            }
        }
        .frame(minWidth: 540, idealWidth: 600, minHeight: 380, idealHeight: 460)
        .task { await self.model.load() }
    }
}

// MARK: - Imports sheet

struct ImportsSheet: View {
    @Environment(\.dismiss) private var dismiss
    let helperPort: Int

    struct ImportMeta: Codable, Identifiable, Sendable {
        let importID: String
        let vendor: String
        let createdAt: String
        let conversationCount: Int
        let schemaFingerprint: String?
        var id: String { self.importID }

        enum CodingKeys: String, CodingKey {
            case importID = "import_id"
            case vendor
            case createdAt = "created_at"
            case conversationCount = "conversation_count"
            case schemaFingerprint = "schema_fingerprint"
        }
    }

    @State private var imports: [ImportMeta] = []
    @State private var isLoading = false

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            SheetHeader(title: "Imports", onDismiss: { self.dismiss() }) {
                Button("Refresh") { Task { await self.load() } }
                    .disabled(self.isLoading)
            }

            Divider()

            if self.isLoading && self.imports.isEmpty {
                ProgressView()
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if self.imports.isEmpty {
                ContentUnavailableView(
                    "No imports yet",
                    systemImage: "square.and.arrow.down",
                    description: Text(
                        "Request a data export from claude.ai (Settings → Export data) " +
                        "and run: heimdall import-export <zip>"
                    )
                )
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                Table(self.imports) {
                    TableColumn("Vendor") { m in
                        Text(m.vendor).font(.caption.weight(.medium))
                    }
                    .width(80)
                    TableColumn("Imported") { m in
                        Text(m.createdAt).font(.caption).foregroundStyle(.secondary)
                    }
                    TableColumn("Conversations") { m in
                        Text("\(m.conversationCount)").font(.caption.monospacedDigit())
                    }
                    .width(100)
                    TableColumn("Schema") { m in
                        Text(String((m.schemaFingerprint ?? "—").prefix(12)))
                            .font(.caption.monospaced())
                            .foregroundStyle(.secondary)
                    }
                    .width(100)
                }
                .tableStyle(.inset)
            }
        }
        .frame(minWidth: 520, idealWidth: 580, minHeight: 360, idealHeight: 440)
        .task { await self.load() }
    }

    private func load() async {
        guard !self.isLoading else { return }
        self.isLoading = true
        defer { self.isLoading = false }
        guard let url = URL(string: "http://127.0.0.1:\(self.helperPort)/api/archive/imports") else { return }
        do {
            let (data, _) = try await URLSession.shared.data(from: url)
            self.imports = try JSONDecoder().decode([ImportMeta].self, from: data)
        } catch {}
    }
}

// MARK: - Web captures sheet

struct WebCapturesSheet: View {
    @Environment(\.dismiss) private var dismiss
    let helperPort: Int

    struct WebConversation: Codable, Identifiable, Sendable {
        let vendor: String
        let conversationID: String
        let capturedAt: String
        let historyCount: Int
        var id: String { "\(self.vendor)/\(self.conversationID)" }

        enum CodingKeys: String, CodingKey {
            case vendor
            case conversationID = "conversation_id"
            case capturedAt = "captured_at"
            case historyCount = "history_count"
        }
    }

    struct CompanionHeartbeat: Codable, Sendable {
        let vendorsSeen: [String]
        let lastSeenAt: String

        enum CodingKeys: String, CodingKey {
            case vendorsSeen = "vendors_seen"
            case lastSeenAt = "last_seen_at"
        }
    }

    struct WebCapturesResponse: Codable, Sendable {
        let conversations: [WebConversation]
        let heartbeat: CompanionHeartbeat?
    }

    @State private var conversations: [WebConversation] = []
    @State private var heartbeat: CompanionHeartbeat?
    @State private var isLoading = false

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            SheetHeader(title: "Web captures", onDismiss: { self.dismiss() }) {
                Button("Refresh") { Task { await self.load() } }
                    .disabled(self.isLoading)
            }

            if let hb = self.heartbeat {
                HStack {
                    Image(systemName: "circle.fill")
                        .foregroundStyle(.green)
                        .imageScale(.small)
                    Text("Companion connected")
                        .font(.caption)
                    if !hb.vendorsSeen.isEmpty {
                        Text("(\(hb.vendorsSeen.joined(separator: " + ")))")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                    Text("· last seen \(self.relativeTime(hb.lastSeenAt))")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                .padding(.horizontal, 16)
                .padding(.vertical, 8)
                Divider()
            }

            Divider()

            if self.isLoading && self.conversations.isEmpty {
                ProgressView()
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if self.conversations.isEmpty {
                ContentUnavailableView(
                    "No web captures yet",
                    systemImage: "globe",
                    description: Text(
                        "Install the Heimdall companion browser extension and pair it with: " +
                        "heimdall companion-token show"
                    )
                )
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                Table(self.conversations) {
                    TableColumn("Vendor") { c in
                        Text(c.vendor).font(.caption.weight(.medium))
                    }
                    .width(70)
                    TableColumn("Conversation") { c in
                        Text(c.conversationID)
                            .font(.caption.monospaced())
                            .lineLimit(1)
                    }
                    TableColumn("Captured") { c in
                        Text(self.relativeTime(c.capturedAt))
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                    .width(80)
                    TableColumn("History") { c in
                        Text("\(c.historyCount)").font(.caption.monospacedDigit())
                    }
                    .width(60)
                }
                .tableStyle(.inset)
            }
        }
        .frame(minWidth: 520, idealWidth: 600, minHeight: 360, idealHeight: 460)
        .task { await self.load() }
    }

    private func load() async {
        guard !self.isLoading else { return }
        self.isLoading = true
        defer { self.isLoading = false }
        guard let url = URL(string: "http://127.0.0.1:\(self.helperPort)/api/archive/web-conversations") else { return }
        do {
            let (data, _) = try await URLSession.shared.data(from: url)
            let resp = try JSONDecoder().decode(WebCapturesResponse.self, from: data)
            self.conversations = resp.conversations
            self.heartbeat = resp.heartbeat
        } catch {}
    }

    private func relativeTime(_ iso: String) -> String {
        let formatter = ISO8601DateFormatter()
        guard let date = formatter.date(from: iso) else { return iso }
        let mins = Int(max(0, Date().timeIntervalSince(date) / 60))
        if mins < 1 { return "just now" }
        if mins < 60 { return "\(mins)m ago" }
        let hrs = mins / 60
        if hrs < 48 { return "\(hrs)h ago" }
        return "\(hrs / 24)d ago"
    }
}

// MARK: - Shared sheet header

struct SheetHeader<Trailing: View>: View {
    let title: String
    let onDismiss: () -> Void
    @ViewBuilder let trailing: () -> Trailing

    var body: some View {
        HStack {
            Text(self.title)
                .font(.headline)
            Spacer()
            self.trailing()
            Button(action: self.onDismiss) {
                Image(systemName: "xmark.circle.fill")
                    .foregroundStyle(.secondary)
            }
            .buttonStyle(.plain)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
    }
}
