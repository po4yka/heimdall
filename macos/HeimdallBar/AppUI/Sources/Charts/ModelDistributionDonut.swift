import Charts
import HeimdallDomain
import SwiftUI

/// Model-mix donut chart: cost share by model family on the left, call-share
/// donut on the right, shared legend in the middle. Both donuts share a single
/// hover state so highlighting cascades across all three views.
struct ModelDistributionDonut: View {
    let rows: [ProviderModelRow]
    let dailyByModel: [ProviderDailyModelRow]

    init(rows: [ProviderModelRow], dailyByModel: [ProviderDailyModelRow] = []) {
        self.rows = rows
        self.dailyByModel = dailyByModel
    }

    private static let displayCap = 8
    private static let donutSize: CGFloat = 80
    nonisolated private static let donutInnerRatio: CGFloat = 0.62
    nonisolated private static let donutOuterRatio: CGFloat = 0.98
    private static let dimMultiplier: Double = 0.45

    enum DonutMetric: String, Equatable {
        case cost
        case calls
        case tokens

        var captionLabel: String {
            switch self {
            case .cost: return "Cost share"
            case .calls: return "Call share"
            case .tokens: return "Token share"
            }
        }

        var accessibilityLabel: String {
            switch self {
            case .cost: return "cost"
            case .calls: return "calls"
            case .tokens: return "tokens"
            }
        }
    }

    @State private var hoveredFamily: String?

    var body: some View {
        let capped = Array(self.rows.prefix(Self.displayCap))
        let families = Self.families(from: capped)
        let colorMap = Self.colorMap(for: families.map(\.label))
        VStack(alignment: .leading, spacing: 6) {
            ChartHeader(
                title: "Model mix · 30 days",
                caption: "\(capped.count) model\(capped.count == 1 ? "" : "s") · cost vs. calls vs. tokens."
            )
            if capped.isEmpty {
                Text("No model data yet.")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .padding(.vertical, 8)
            } else {
                HStack(alignment: .center, spacing: 12) {
                    self.captionedDonut(
                        families: families,
                        colorMap: colorMap,
                        metric: .cost
                    )
                    self.captionedDonut(
                        families: families,
                        colorMap: colorMap,
                        metric: .calls
                    )
                    self.captionedDonut(
                        families: families,
                        colorMap: colorMap,
                        metric: .tokens
                    )
                    Spacer(minLength: 12)
                    self.legend(families: families, colorMap: colorMap)
                }
                if !self.dailyByModel.isEmpty {
                    ModelFamilyHistoryChart(
                        rows: self.dailyByModel,
                        orderedFamilies: families.map(\.label),
                        colorMap: colorMap,
                        hoveredFamily: self.$hoveredFamily
                    )
                    .padding(.top, 4)
                }
            }
        }
        .padding(8)
        .menuCardBackground(
            opacity: ChartStyle.cardBackgroundOpacity,
            cornerRadius: ChartStyle.cardCornerRadius
        )
        .accessibilityElement(children: .combine)
        .accessibilityLabel("Model mix donut, \(rows.count) models, by cost, calls, and tokens")
    }

    @ViewBuilder
    private func captionedDonut(
        families: [FamilyEntry],
        colorMap: [String: Color],
        metric: DonutMetric
    ) -> some View {
        VStack(spacing: 4) {
            self.donut(families: families, colorMap: colorMap, metric: metric)
            Text(metric.captionLabel)
                .font(.system(size: 8, weight: .bold))
                .tracking(0.5)
                .foregroundStyle(Color.primary.opacity(0.48))
        }
    }

    @ViewBuilder
    private func donut(
        families: [FamilyEntry],
        colorMap: [String: Color],
        metric: DonutMetric
    ) -> some View {
        let displayRange: [Color] = families.map { family in
            let base = colorMap[family.label] ?? Color.primary.opacity(0.14)
            if let hovered = self.hoveredFamily, hovered != family.label {
                return base.opacity(Self.dimMultiplier)
            }
            return base
        }
        Chart {
            ForEach(families) { family in
                SectorMark(
                    angle: .value(metric.accessibilityLabel, family.value(for: metric)),
                    innerRadius: .ratio(Self.donutInnerRatio),
                    outerRadius: .ratio(Self.donutOuterRatio)
                )
                .foregroundStyle(by: .value("Model", family.label))
            }
        }
        .chartForegroundStyleScale(
            domain: families.map(\.label),
            range: displayRange
        )
        .chartLegend(.hidden)
        .frame(width: Self.donutSize, height: Self.donutSize)
        .chartOverlay { _ in
            GeometryReader { geo in
                Rectangle()
                    .fill(Color.clear)
                    .contentShape(Rectangle())
                    .onContinuousHover { phase in
                        switch phase {
                        case .active(let location):
                            let next = Self.familyAt(
                                point: location,
                                in: geo.size,
                                families: families,
                                metric: metric
                            )
                            ChartStyle.updateHoverSelection(&self.hoveredFamily, to: next)
                        case .ended:
                            ChartStyle.updateHoverSelection(&self.hoveredFamily, to: nil)
                        }
                    }
            }
        }
        .animation(ChartStyle.animation, value: families)
        .animation(ChartStyle.hoverAnimation, value: self.hoveredFamily)
    }

    @ViewBuilder
    private func legend(families: [FamilyEntry], colorMap: [String: Color]) -> some View {
        let total = families.reduce(0) { $0 + $1.costUSD }
        Grid(alignment: .leadingFirstTextBaseline, horizontalSpacing: 8, verticalSpacing: 4) {
            ForEach(families) { family in
                let isActive = self.hoveredFamily == family.label
                let isDim = self.hoveredFamily != nil && !isActive
                let percent = total > 0 ? family.costUSD / total * 100 : 0
                GridRow {
                    RoundedRectangle(cornerRadius: 1.5, style: .continuous)
                        .fill(colorMap[family.label] ?? Color.primary.opacity(0.14))
                        .frame(width: 10, height: 4)
                        .opacity(isDim ? Self.dimMultiplier : 1)
                    Text(family.label)
                        .font(.caption2.weight(isActive ? .semibold : .regular))
                        .foregroundStyle(isActive ? Color.primary : Color.secondary)
                        .lineLimit(1)
                        .opacity(isDim ? 0.6 : 1)
                    Text(String(format: "%.1f%%", percent))
                        .font(.caption2.monospacedDigit())
                        .foregroundStyle(Color.primary.opacity(0.55))
                        .opacity(isDim ? 0.6 : 1)
                        .gridColumnAlignment(.trailing)
                    Text(Self.formatCost(family.costUSD))
                        .font(.caption2.monospacedDigit().weight(.semibold))
                        .foregroundStyle(isActive ? Color.primary : Color.primary.opacity(0.85))
                        .opacity(isDim ? 0.6 : 1)
                        .gridColumnAlignment(.trailing)
                }
                .onHover { hovering in
                    ChartStyle.updateHoverSelection(
                        &self.hoveredFamily,
                        to: hovering ? family.label : nil
                    )
                }
            }
        }
        .animation(ChartStyle.hoverAnimation, value: self.hoveredFamily)
    }

    // MARK: - Helpers

    struct FamilyEntry: Identifiable, Hashable {
        let label: String
        let costUSD: Double
        let turns: Int
        let tokens: Int

        var id: String { self.label }

        func value(for metric: DonutMetric) -> Double {
            switch metric {
            case .cost: return self.costUSD
            case .calls: return Double(self.turns)
            case .tokens: return Double(self.tokens)
            }
        }
    }

    /// Collapse individual models into family groups, sorted by cost descending.
    nonisolated static func families(from rows: [ProviderModelRow]) -> [FamilyEntry] {
        var grouped: [String: (cost: Double, turns: Int, tokens: Int)] = [:]
        for row in rows {
            let label = modelFamilyLabel(row.model)
            var entry = grouped[label] ?? (cost: 0, turns: 0, tokens: 0)
            entry.cost += row.costUSD
            entry.turns += row.turns
            entry.tokens += row.input + row.output
                + row.cacheRead + row.cacheCreation
                + row.reasoningOutput
            grouped[label] = entry
        }
        return grouped
            .map {
                FamilyEntry(
                    label: $0.key,
                    costUSD: $0.value.cost,
                    turns: $0.value.turns,
                    tokens: $0.value.tokens
                )
            }
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

    nonisolated static func formatCost(_ usd: Double) -> String {
        if usd >= 1000 { return String(format: "$%.0f", usd) }
        if usd >= 10   { return String(format: "$%.1f", usd) }
        return String(format: "$%.2f", usd)
    }

    /// Hit-test a pointer location against the donut ring; returns the family
    /// label whose sector contains the cursor, or nil if outside the ring.
    nonisolated static func familyAt(
        point: CGPoint,
        in size: CGSize,
        families: [FamilyEntry],
        metric: DonutMetric
    ) -> String? {
        guard !families.isEmpty else { return nil }
        let cx = size.width / 2
        let cy = size.height / 2
        let dx = point.x - cx
        let dy = point.y - cy
        let dist = sqrt(dx * dx + dy * dy)
        let halfMin = min(size.width, size.height) / 2
        let inner = halfMin * Self.donutInnerRatio
        let outer = halfMin * Self.donutOuterRatio
        guard dist >= inner, dist <= outer else { return nil }

        // Angle measured from 12 o'clock, clockwise, in [0, 2π).
        var angle = atan2(dx, -dy)
        if angle < 0 { angle += 2 * .pi }

        let total = families.reduce(0) { $0 + $1.value(for: metric) }
        guard total > 0 else { return nil }

        let normalized = angle / (2 * .pi)
        var cumulative = 0.0
        for family in families {
            cumulative += family.value(for: metric) / total
            if normalized <= cumulative + 1e-9 {
                return family.label
            }
        }
        return families.last?.label
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
    let formatter = DateFormatter()
    formatter.dateFormat = "yyyy-MM-dd"
    let dailyByModel: [ProviderDailyModelRow] = (0..<14).flatMap { offset -> [ProviderDailyModelRow] in
        let date = Calendar.current.date(byAdding: .day, value: -(13 - offset), to: Date()) ?? Date()
        let day = formatter.string(from: date)
        return [
            ProviderDailyModelRow(day: day, model: "claude-opus-4-5",   costUSD: Double.random(in: 25...60), input: 5_000, output: 1_000, cacheRead: 30_000, cacheCreation: 1_500, reasoningOutput: 800, turns: 12),
            ProviderDailyModelRow(day: day, model: "claude-sonnet-4-5", costUSD: Double.random(in: 5...20),  input: 3_000, output: 600,   cacheRead: 12_000, cacheCreation: 600,   reasoningOutput: 0,   turns: 28),
            ProviderDailyModelRow(day: day, model: "claude-haiku-3-5",  costUSD: Double.random(in: 0.5...4), input: 1_500, output: 300,   cacheRead: 4_000,  cacheCreation: 200,   reasoningOutput: 0,   turns: 60),
            ProviderDailyModelRow(day: day, model: "gpt-5",             costUSD: Double.random(in: 0.5...3), input: 800,   output: 200,   cacheRead: 0,      cacheCreation: 0,     reasoningOutput: 600, turns: 6),
        ]
    }
    return ModelDistributionDonut(rows: rows, dailyByModel: dailyByModel)
        .padding()
        .frame(width: 480)
}
