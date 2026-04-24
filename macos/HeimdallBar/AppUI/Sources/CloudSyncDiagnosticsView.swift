import HeimdallDomain
import HeimdallServices
import SwiftUI

#if DEBUG
struct CloudSyncDiagnosticsSection: View {
    let diagnostics: CloudSyncDiagnostics

    var body: some View {
        Section("Cloud Sync Diagnostics (Debug)") {
            LabeledContent("Container", value: self.diagnostics.containerIdentifier)
                .font(.caption.monospaced())
                .textSelection(.enabled)
            LabeledContent("Zone", value: self.diagnostics.zoneName)
                .font(.caption.monospaced())
            LabeledContent("Zone Owner", value: self.diagnostics.zoneOwner)
                .font(.caption.monospaced())
            LabeledContent("Installation ID", value: self.diagnostics.truncatedInstallationID)
                .font(.caption.monospaced())
            LabeledContent("Role", value: self.roleDescription)
            LabeledContent("Status", value: self.statusDescription)
            LabeledContent("Engine State", value: self.engineStateDescription)
                .font(.caption.monospaced())
            if let lastPublishedAt = self.diagnostics.lastPublishedAt {
                LabeledContent("Last Published", value: lastPublishedAt)
                    .font(.caption.monospaced())
            }
            if let lastAcceptedAt = self.diagnostics.lastAcceptedAt {
                LabeledContent("Last Accepted", value: lastAcceptedAt)
                    .font(.caption.monospaced())
            }
        }
    }

    private var roleDescription: String {
        switch self.diagnostics.role {
        case .none: return "none"
        case .owner: return "owner"
        case .participant: return "participant"
        }
    }

    private var statusDescription: String {
        switch self.diagnostics.status {
        case .notConfigured: return "not configured"
        case .ownerReady: return "owner ready"
        case .inviteReady: return "invite ready"
        case .participantJoined: return "participant joined"
        case .iCloudUnavailable: return "iCloud unavailable"
        case .sharingBlocked: return "sharing blocked"
        }
    }

    private var engineStateDescription: String {
        guard let bytes = self.diagnostics.engineStateFileBytes else {
            return "not written yet"
        }
        if bytes < 1024 {
            return "\(bytes) B"
        }
        let kb = Double(bytes) / 1024.0
        return String(format: "%.1f KB", kb)
    }
}
#endif
