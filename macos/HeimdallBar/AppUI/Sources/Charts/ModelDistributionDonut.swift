import Charts
import HeimdallDomain
import SwiftUI

/// Model-mix donut chart: cost share by model family, 96pt diameter,
/// monochrome/accent colour ladder, inline legend below.
struct ModelDistributionDonut: View {
    let rows: [ProviderModelRow]

    private static let displayCap = 8

    var body: some View {
        let capped = Array(self.rows.prefix(Self.displayCap))
        VStack(alignment: .leading, spacing: 6) {
            ChartHeader(
                title: "Model mix · 30 days",
                caption: "\(capped.count) model\(capped.count == 1 ? "" : "s") by cost."
            )
            if capped.isEmpty {
                Text("No model data yet.")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .padding(.vertical, 8)
            } else {
                let families = Self.families(from: capped)
                let colorMap = Self.colorMap(for: families.map(\.label))
                HStack(alignment: .center, spacing: 12) {
                    Chart {
                        ForEach(families) { family in
                            SectorMark(
                                angle: .value("Cost", family.costUSD),
                                innerRadius: .ratio(0.62),
                                outerRadius: .ratio(0.98)
                            )
                            .foregroundStyle(by: .value("Model", family.label))
                        }
                    }
                    .chartForegroundStyleScale(
                        domain: families.map(\.label),
                        range: families.map { colorMap[$0.label] ?? Color.primary.opacity(0.14) }
                    )
                    .chartLegend(.hidden)
                    .frame(width: 96, height: 96)
                    .help(Self.tooltip(for: families))
                    .animation(ChartStyle.animation, value: families)

                    // Inline legend
                    VStack(alignment: .leading, spacing: 4) {
                        ForEach(families) { family in
                            HStack(spacing: 5) {
                                RoundedRectangle(cornerRadius: 1.5, style: .continuous)
                                    .fill(colorMap[family.label] ?? Color.primary.opacity(0.14))
                                    .frame(width: 10, height: 4)
                                Text(family.label)
                                    .font(.caption2)
                                    .foregroundStyle(.secondary)
                                    .lineLimit(1)
                                    .help("\(family.label): \(Self.formatCost(family.costUSD))")
                                Spacer(minLength: 2)
                                Text(Self.formatCost(family.costUSD))
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
        .accessibilityLabel("Model mix donut, \(rows.count) models")
    }

    // MARK: - Helpers

    struct FamilyEntry: Identifiable, Hashable {
        let label: String
        let costUSD: Double
        var id: String { self.label }
    }

    /// Collapse individual models into family groups, sorted by cost descending.
    nonisolated static func families(from rows: [ProviderModelRow]) -> [FamilyEntry] {
        var grouped: [String: Double] = [:]
        for row in rows {
            let label = modelFamilyLabel(row.model)
            grouped[label, default: 0] += row.costUSD
        }
        return grouped
            .map { FamilyEntry(label: $0.key, costUSD: $0.value) }
            .sorted { $0.costUSD > $1.costUSD }
    }

    /// Map a raw model identifier to a readable family name.
    nonisolated static func modelFamilyLabel(_ model: String) -> String {
        let lower = model.lowercased()
        if lower.hasPrefix("claude-opus")    { return "Opus" }
        if lower.hasPrefix("claude-sonnet")  { return "Sonnet" }
        if lower.hasPrefix("claude-haiku")   { return "Haiku" }
        if lower.hasPrefix("gpt-5")          { return "GPT-5" }
        if lower.hasPrefix("gpt-")           { return "GPT" }
        if lower == "unknown"                { return "Unknown" }
        // Fall back to the capitalised first dash-segment.
        let first = model.split(separator: "-").first.map(String.init) ?? model
        return first.prefix(1).uppercased() + first.dropFirst()
    }

    /// Monochrome/accent colour ladder; cycles when more families than slots.
    nonisolated static func colorMap(for labels: [String]) -> [String: Color] {
        let palette: [Color] = [
            Color.accentColor,
            Color.primary.opacity(0.88),
            Color.primary.opacity(0.55),
            Color.primary.opacity(0.28),
            Color.primary.opacity(0.14),
        ]
        var map: [String: Color] = [:]
        for (idx, label) in labels.enumerated() {
            map[label] = palette[idx % palette.count]
        }
        return map
    }

    nonisolated static func tooltip(for families: [FamilyEntry]) -> String {
        families.map { family in
            "\(family.label): \(Self.formatCost(family.costUSD))"
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

#Preview("Model mix donut — 5 models") {
    let rows: [ProviderModelRow] = [
        ProviderModelRow(model: "claude-opus-4-5", costUSD: 42.80, input: 120_000, output: 18_000, cacheRead: 540_000, cacheCreation: 22_000, reasoningOutput: 8_000, turns: 190),
        ProviderModelRow(model: "claude-sonnet-4-5", costUSD: 18.35, input: 80_000, output: 14_000, cacheRead: 300_000, cacheCreation: 10_000, reasoningOutput: 0, turns: 420),
        ProviderModelRow(model: "claude-haiku-3-5", costUSD: 3.12, input: 45_000, output: 9_000, cacheRead: 120_000, cacheCreation: 4_000, reasoningOutput: 0, turns: 810),
        ProviderModelRow(model: "gpt-5", costUSD: 1.55, input: 22_000, output: 4_000, cacheRead: 0, cacheCreation: 0, reasoningOutput: 3_000, turns: 55),
        ProviderModelRow(model: "o3-mini", costUSD: 0.44, input: 8_000, output: 1_500, cacheRead: 0, cacheCreation: 0, reasoningOutput: 6_000, turns: 18),
    ]
    ModelDistributionDonut(rows: rows)
        .padding()
        .frame(width: 336)
}
