import Charts
import HeimdallDomain
import SwiftUI

/// Four-cell grid summarising subagent sidechain activity over 30 days.
struct SubagentSummaryCard: View {
    let breakdown: ProviderSubagentBreakdown

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            ChartHeader(
                title: "Subagents · 30 days",
                caption: "Parallel sidechain activity."
            )
            LazyVGrid(
                columns: [GridItem(.flexible()), GridItem(.flexible())],
                spacing: 8
            ) {
                StatCell(label: "Turns",    value: "\(self.breakdown.totalTurns)")
                StatCell(label: "Spend",    value: Self.formatCost(self.breakdown.totalCostUSD))
                StatCell(label: "Sessions", value: "\(self.breakdown.sessionCount)")
                StatCell(label: "Agents",   value: "\(self.breakdown.agentCount)")
            }
        }
        .padding(8)
        .menuCardBackground(
            opacity: ChartStyle.cardBackgroundOpacity,
            cornerRadius: ChartStyle.cardCornerRadius
        )
        .help(Self.tooltip(for: self.breakdown))
        .accessibilityElement(children: .combine)
        .accessibilityLabel(
            "Subagents: \(breakdown.totalTurns) turns, \(Self.formatCost(breakdown.totalCostUSD)) spend, " +
            "\(breakdown.sessionCount) sessions, \(breakdown.agentCount) agents"
        )
    }

    nonisolated static func formatCost(_ usd: Double) -> String {
        if usd >= 1000 { return String(format: "$%.0f", usd) }
        if usd >= 10   { return String(format: "$%.1f", usd) }
        return String(format: "$%.2f", usd)
    }

    nonisolated static func tooltip(for breakdown: ProviderSubagentBreakdown) -> String {
        [
            "Turns: \(breakdown.totalTurns)",
            "Spend: \(Self.formatCost(breakdown.totalCostUSD))",
            "Sessions: \(breakdown.sessionCount)",
            "Agents: \(breakdown.agentCount)",
        ]
        .joined(separator: "\n")
    }
}

private struct StatCell: View {
    let label: String
    let value: String

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(self.label)
                .font(.system(size: 10))
                .foregroundStyle(Color.primary.opacity(ChartStyle.headerCaptionOpacity))
            Text(self.value)
                .font(.body.monospacedDigit().weight(.semibold))
                .foregroundStyle(.primary)
                .minimumScaleFactor(0.8)
                .lineLimit(1)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }
}

// MARK: - Preview

#Preview("Subagent summary") {
    let breakdown = ProviderSubagentBreakdown(
        totalTurns: 1_248,
        totalCostUSD: 18.43,
        sessionCount: 94,
        agentCount: 7
    )
    SubagentSummaryCard(breakdown: breakdown)
        .padding()
        .frame(width: 220)
}
