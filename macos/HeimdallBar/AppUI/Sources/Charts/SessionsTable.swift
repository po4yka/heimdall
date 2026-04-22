import Charts
import HeimdallDomain
import SwiftUI

/// Compact list of recent sessions, capped at 8 rows, most recent first.
struct SessionsTable: View {
    let sessions: [ProviderSession]

    private static let displayCap = 8

    var body: some View {
        let capped = Array(self.sessions.prefix(Self.displayCap))
        VStack(alignment: .leading, spacing: 6) {
            ChartHeader(
                title: "Recent sessions",
                caption: "Most recent first."
            )
            if capped.isEmpty {
                Text("No sessions yet.")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .padding(.vertical, 8)
            } else {
                VStack(spacing: 0) {
                    ForEach(Array(capped.enumerated()), id: \.element.id) { idx, session in
                        if idx > 0 {
                            Divider()
                                .overlay(Color.primary.opacity(0.08))
                                .padding(.vertical, 4)
                        }
                        SessionRow(session: session)
                    }
                }
            }
        }
        .padding(8)
        .menuCardBackground(
            opacity: ChartStyle.cardBackgroundOpacity,
            cornerRadius: ChartStyle.cardCornerRadius
        )
        .accessibilityElement(children: .combine)
        .accessibilityLabel("Recent sessions, \(sessions.count) total")
    }

    nonisolated static func formatCost(_ usd: Double) -> String {
        if usd >= 1000 { return String(format: "$%.0f", usd) }
        if usd >= 10   { return String(format: "$%.1f", usd) }
        return String(format: "$%.2f", usd)
    }
}

private struct SessionRow: View {
    let session: ProviderSession

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            HStack(alignment: .firstTextBaseline) {
                Text(self.session.displayName)
                    .font(.body.weight(.medium))
                    .lineLimit(1)
                    .truncationMode(.middle)
                    .frame(maxWidth: .infinity, alignment: .leading)
                Text(SessionsTable.formatCost(self.session.costUSD))
                    .font(.body.monospacedDigit().weight(.semibold))
                    .fixedSize()
            }
            Text("\(self.session.turns) turns · \(self.session.durationMinutes)m · \(self.session.model ?? "—")")
                .font(.caption2)
                .foregroundStyle(.secondary)
                .monospacedDigit()
                .lineLimit(1)
        }
    }
}

// MARK: - Preview

#Preview("Recent sessions — 4 rows") {
    let sessions: [ProviderSession] = [
        ProviderSession(
            sessionID: "s1",
            displayName: "heimdall/src/scanner/parser.rs",
            startedAt: "2026-04-22T14:30:00Z",
            durationMinutes: 48,
            turns: 82,
            costUSD: 3.14,
            model: "claude-sonnet-4-5"
        ),
        ProviderSession(
            sessionID: "s2",
            displayName: "heimdall/macos/HeimdallBar/AppUI/Sources/Charts/ModelCostTable.swift",
            startedAt: "2026-04-22T11:10:00Z",
            durationMinutes: 23,
            turns: 31,
            costUSD: 0.87,
            model: "claude-haiku-3-5"
        ),
        ProviderSession(
            sessionID: "s3",
            displayName: "New conversation",
            startedAt: "2026-04-21T22:05:00Z",
            durationMinutes: 7,
            turns: 9,
            costUSD: 0.12,
            model: nil
        ),
        ProviderSession(
            sessionID: "s4",
            displayName: "heimdall/src/pricing.rs",
            startedAt: "2026-04-21T16:45:00Z",
            durationMinutes: 91,
            turns: 145,
            costUSD: 11.60,
            model: "claude-opus-4-5"
        ),
    ]
    SessionsTable(sessions: sessions)
        .padding()
        .frame(width: 320)
}
