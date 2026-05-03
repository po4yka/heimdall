import Charts
import HeimdallDomain
import SwiftUI

/// Compact per-tool invocation table with category/MCP chips and an error dot.
struct ToolUsageTable: View {
    let rows: [ProviderToolRow]
    var onErrorTap: ((String) -> Void)?

    private static let displayCap = 8

    var body: some View {
        let capped = Array(self.rows.prefix(Self.displayCap))
        VStack(alignment: .leading, spacing: 6) {
            ChartHeader(
                title: "Tool usage · 30 days",
                caption: "Top tools by invocation count."
            )
            if capped.isEmpty {
                Text("No tool data yet.")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .padding(.vertical, 8)
            } else {
                let maxInvocations = capped.map(\.invocations).max() ?? 1
                VStack(spacing: 4) {
                    ForEach(capped) { row in
                        ToolUsageRow(row: row, maxInvocations: maxInvocations, onErrorTap: self.onErrorTap)
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
        .accessibilityLabel("Tool usage, top \(min(rows.count, Self.displayCap)) tools")
    }

    nonisolated static func formatInvocations(_ count: Int) -> String {
        if count >= 1_000_000 { return String(format: "%.1fM", Double(count) / 1_000_000) }
        if count >= 1_000     { return String(format: "%.1fK", Double(count) / 1_000) }
        return "\(count)"
    }
}

private struct ToolUsageRow: View {
    let row: ProviderToolRow
    let maxInvocations: Int
    var onErrorTap: ((String) -> Void)?

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            HStack(alignment: .center, spacing: 4) {
                // Tool name
                Text(self.row.toolName)
                    .font(.body.weight(.medium))
                    .lineLimit(1)
                    .truncationMode(.middle)

                // Category chip
                if let category = self.row.category {
                    ToolChip(label: category)
                }

                // MCP server chip
                if let server = self.row.mcpServer {
                    ToolChip(label: server)
                }

                Spacer(minLength: 4)

                // Error indicator — taps open the tool-errors detail screen
                if self.row.errors > 0 {
                    Button {
                        self.onErrorTap?(self.row.toolName)
                    } label: {
                        HStack(spacing: 3) {
                            Circle()
                                .fill(Color.red)
                                .frame(width: 5, height: 5)
                            Text("\(self.row.errors) err")
                                .font(.caption2.monospacedDigit())
                                .foregroundStyle(.red)
                        }
                    }
                    .buttonStyle(.plain)
                    .help("View error details")
                }

                // Invocation count
                Text("\(ToolUsageTable.formatInvocations(self.row.invocations)) calls")
                    .font(.body.monospacedDigit().weight(.semibold))
                    .fixedSize()
            }

            // Proportional bar
            GeometryReader { geo in
                ZStack(alignment: .leading) {
                    Rectangle()
                        .fill(Color.primary.opacity(0.10))
                        .frame(height: 3)
                    Rectangle()
                        .fill(Color.primary.opacity(0.55))
                        .frame(
                            width: max(
                                2,
                                geo.size.width * CGFloat(self.row.invocations) / CGFloat(max(self.maxInvocations, 1))
                            ),
                            height: 3
                        )
                }
                .clipShape(RoundedRectangle(cornerRadius: 1.5, style: .continuous))
            }
            .frame(height: 3)
        }
    }
}

private struct ToolChip: View {
    let label: String

    var body: some View {
        Text(self.label)
            .font(.system(size: 9).weight(.medium))
            .foregroundStyle(.secondary)
            .padding(.horizontal, 4)
            .padding(.vertical, 1)
            .background(
                RoundedRectangle(cornerRadius: 3, style: .continuous)
                    .fill(Color.primary.opacity(0.08))
            )
            .overlay(
                RoundedRectangle(cornerRadius: 3, style: .continuous)
                    .stroke(Color.primary.opacity(0.10), lineWidth: 0.5)
            )
            .lineLimit(1)
            .truncationMode(.tail)
    }
}

// MARK: - Preview

#Preview("Tool usage — 5 rows") {
    let rows: [ProviderToolRow] = [
        ProviderToolRow(toolName: "Bash", category: "shell", mcpServer: nil, invocations: 3_820, errors: 12, turnsUsed: 940, sessionsUsed: 38),
        ProviderToolRow(toolName: "Read", category: "fs", mcpServer: nil, invocations: 2_150, errors: 0, turnsUsed: 780, sessionsUsed: 35),
        ProviderToolRow(toolName: "Edit", category: "fs", mcpServer: nil, invocations: 1_490, errors: 3, turnsUsed: 610, sessionsUsed: 30),
        ProviderToolRow(toolName: "query-docs", category: nil, mcpServer: "context7", invocations: 440, errors: 0, turnsUsed: 210, sessionsUsed: 18),
        ProviderToolRow(toolName: "codex", category: nil, mcpServer: "codex-mcp", invocations: 88, errors: 2, turnsUsed: 55, sessionsUsed: 9),
    ]
    ToolUsageTable(rows: rows, onErrorTap: nil)
        .padding()
        .frame(width: 336)
}
