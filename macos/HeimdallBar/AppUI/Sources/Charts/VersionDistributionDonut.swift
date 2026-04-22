import Charts
import HeimdallDomain
import SwiftUI

/// CLI version share donut + inline legend, by cost, capped at 5 versions.
struct VersionDistributionDonut: View {
    let rows: [ProviderVersionRow]

    private static let displayCap = 5

    var body: some View {
        let capped = Array(self.rows.prefix(Self.displayCap))
        VStack(alignment: .leading, spacing: 6) {
            ChartHeader(
                title: "CLI versions · 30 days",
                caption: "Top versions by cost."
            )
            if capped.isEmpty {
                Text("No version data yet.")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .padding(.vertical, 8)
            } else {
                let colors = Self.scale(count: capped.count)
                HStack(alignment: .center, spacing: 12) {
                    Chart {
                        ForEach(Array(capped.enumerated()), id: \.element.id) { idx, row in
                            SectorMark(
                                angle: .value("Cost", row.costUSD),
                                innerRadius: .ratio(0.62),
                                outerRadius: .ratio(0.98)
                            )
                            .foregroundStyle(by: .value("Version", row.version))
                        }
                    }
                    .chartForegroundStyleScale(
                        domain: capped.map(\.version),
                        range: colors
                    )
                    .chartLegend(.hidden)
                    .frame(width: 96, height: 96)
                    .help(Self.tooltip(for: capped))
                    .animation(ChartStyle.animation, value: capped.map(\.version))

                    // Inline legend
                    VStack(alignment: .leading, spacing: 4) {
                        ForEach(Array(capped.enumerated()), id: \.element.id) { idx, row in
                            HStack(spacing: 5) {
                                RoundedRectangle(cornerRadius: 1.5, style: .continuous)
                                    .fill(colors[idx % colors.count])
                                    .frame(width: 10, height: 4)
                                Text(row.version)
                                    .font(.caption2)
                                    .foregroundStyle(.secondary)
                                    .lineLimit(1)
                                    .help("\(row.version): \(Self.formatCost(row.costUSD)) · \(row.turns) turns · \(row.sessions) sessions")
                                Spacer(minLength: 2)
                                Text(Self.formatCost(row.costUSD))
                                    .font(.caption2.monospacedDigit().weight(.semibold))
                                    .foregroundStyle(.primary)
                            }
                        }
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                }
            }
        }
        .padding(8)
        .menuCardBackground(
            opacity: ChartStyle.cardBackgroundOpacity,
            cornerRadius: ChartStyle.cardCornerRadius
        )
        .accessibilityElement(children: .combine)
        .accessibilityLabel("CLI versions donut, \(rows.count) versions")
    }

    /// Monochrome/accent palette cycling for `count` slots.
    nonisolated static func scale(count: Int) -> [Color] {
        let palette: [Color] = [
            Color.accentColor,
            Color.primary.opacity(0.88),
            Color.primary.opacity(0.55),
            Color.primary.opacity(0.28),
            Color.primary.opacity(0.14),
        ]
        guard count > 0 else { return [] }
        return (0..<count).map { palette[$0 % palette.count] }
    }

    nonisolated static func tooltip(for rows: [ProviderVersionRow]) -> String {
        rows.map { row in
            "\(row.version): \(Self.formatCost(row.costUSD)) · \(row.turns) turns · \(row.sessions) sessions"
        }
        .joined(separator: "\n")
    }

    nonisolated static func formatCost(_ usd: Double) -> String {
        if usd >= 1000 { return String(format: "$%.0f", usd) }
        if usd >= 10   { return String(format: "$%.1f", usd) }
        return String(format: "$%.2f", usd)
    }
}

// MARK: - Preview

#Preview("CLI versions donut — 5 versions") {
    let rows: [ProviderVersionRow] = [
        ProviderVersionRow(version: "1.12.3", turns: 840, sessions: 62, costUSD: 28.50),
        ProviderVersionRow(version: "1.11.0", turns: 410, sessions: 31, costUSD: 11.20),
        ProviderVersionRow(version: "1.10.2", turns: 195, sessions: 18, costUSD: 4.75),
        ProviderVersionRow(version: "1.9.5",  turns: 88,  sessions: 9,  costUSD: 1.83),
        ProviderVersionRow(version: "1.8.1",  turns: 22,  sessions: 4,  costUSD: 0.41),
    ]
    VersionDistributionDonut(rows: rows)
        .padding()
        .frame(width: 320)
}
