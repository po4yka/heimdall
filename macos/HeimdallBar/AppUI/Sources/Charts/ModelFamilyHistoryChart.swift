import Charts
import HeimdallDomain
import SwiftUI

/// 30-day stacked-share bar chart: one bar per day, segments per model
/// family, normalized to 100% so the trend is *mix* not absolute volume.
/// Designed to live inside `ModelDistributionDonut`'s card and share its
/// hover state, so highlighting a family in either widget cascades.
struct ModelFamilyHistoryChart: View {
    let rows: [ProviderDailyModelRow]
    let orderedFamilies: [String]
    let colorMap: [String: Color]
    @Binding var hoveredFamily: String?

    private static let dimMultiplier: Double = 0.45

    fileprivate struct Entry: Identifiable, Hashable {
        let day: String
        let family: String
        let share: Double
        var id: String { "\(self.day)|\(self.family)" }
    }

    var body: some View {
        let entries = Self.entries(rows: self.rows, families: self.orderedFamilies)
        if entries.isEmpty {
            EmptyView()
        } else {
            self.chart(entries: entries)
                .frame(height: 60)
                .accessibilityElement(children: .combine)
                .accessibilityLabel("Model-family share over time, \(self.distinctDays(entries: entries)) days")
        }
    }

    private func chart(entries: [Entry]) -> some View {
        let displayRange: [Color] = self.orderedFamilies.map { family in
            let base = self.colorMap[family] ?? Color.primary.opacity(0.14)
            if let hovered = self.hoveredFamily, hovered != family {
                return base.opacity(Self.dimMultiplier)
            }
            return base
        }
        return Chart {
            ForEach(entries) { entry in
                BarMark(
                    x: .value("Day", entry.day),
                    y: .value("Share", entry.share)
                )
                .foregroundStyle(by: .value("Family", entry.family))
                .accessibilityLabel(entry.day)
                .accessibilityValue("\(entry.family): \(Int((entry.share * 100).rounded()))%")
            }
        }
        .chartForegroundStyleScale(
            domain: self.orderedFamilies,
            range: displayRange
        )
        .chartLegend(.hidden)
        .chartYScale(domain: 0...1)
        .chartYAxis(.hidden)
        .chartXAxis {
            AxisMarks(values: .automatic(desiredCount: 3)) { value in
                AxisValueLabel {
                    if let day = value.as(String.self) {
                        Text(Self.shortDayLabel(day))
                            .font(.system(size: 8).monospacedDigit())
                            .foregroundStyle(.secondary)
                    }
                }
            }
        }
        .chartPlotStyle { plot in
            plot.background(Color.clear)
        }
        .animation(ChartStyle.animation, value: entries)
        .animation(ChartStyle.hoverAnimation, value: self.hoveredFamily)
    }

    private func distinctDays(entries: [Entry]) -> Int {
        Set(entries.map(\.day)).count
    }

    fileprivate static func entries(
        rows: [ProviderDailyModelRow],
        families: [String]
    ) -> [Entry] {
        guard !rows.isEmpty, !families.isEmpty else { return [] }
        let familySet = Set(families)
        let groupedByDay = Dictionary(grouping: rows, by: \.day)
        var result: [Entry] = []
        for (day, dayRows) in groupedByDay {
            var byFamily: [String: Double] = [:]
            for row in dayRows {
                let family = ModelDistributionDonut.modelFamilyLabel(row.model)
                guard familySet.contains(family) else { continue }
                byFamily[family, default: 0] += row.costUSD
            }
            let total = byFamily.values.reduce(0, +)
            guard total > 0 else { continue }
            for family in families {
                let share = (byFamily[family] ?? 0) / total
                if share > 0 {
                    result.append(Entry(day: day, family: family, share: share))
                }
            }
        }
        return result.sorted { lhs, rhs in
            if lhs.day == rhs.day {
                let lhsIndex = families.firstIndex(of: lhs.family) ?? Int.max
                let rhsIndex = families.firstIndex(of: rhs.family) ?? Int.max
                return lhsIndex < rhsIndex
            }
            return lhs.day < rhs.day
        }
    }

    nonisolated private static func shortDayLabel(_ day: String) -> String {
        // "YYYY-MM-DD" -> "MM-DD"
        let parts = day.split(separator: "-")
        guard parts.count == 3 else { return day }
        return "\(parts[1])-\(parts[2])"
    }
}

// MARK: - Preview

#Preview("Model family history — 14 days") {
    let families = ["Opus", "Sonnet", "GPT-5", "Haiku"]
    let dailyShares: [(String, [(String, Double)])] = (0..<14).map { offset in
        let date = Calendar.current.date(byAdding: .day, value: -(13 - offset), to: Date()) ?? Date()
        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy-MM-dd"
        let day = formatter.string(from: date)
        let opus = Double.random(in: 50...90)
        let sonnet = Double.random(in: 5...30)
        let gpt = Double.random(in: 1...15)
        let haiku = Double.random(in: 1...10)
        return (day, [("Opus", opus), ("Sonnet", sonnet), ("GPT-5", gpt), ("Haiku", haiku)])
    }
    let rows: [ProviderDailyModelRow] = dailyShares.flatMap { day, mix in
        mix.map { family, cost in
            ProviderDailyModelRow(
                day: day,
                model: family.lowercased() == "opus" ? "claude-opus-4-5" :
                       family.lowercased() == "sonnet" ? "claude-sonnet-4-5" :
                       family.lowercased() == "haiku" ? "claude-haiku-3-5" : "gpt-5",
                costUSD: cost,
                input: 1_000,
                output: 500,
                cacheRead: 0,
                cacheCreation: 0,
                reasoningOutput: 0,
                turns: 10
            )
        }
    }
    let colorMap = ModelDistributionDonut.colorMap(for: families)
    return ModelFamilyHistoryChart(
        rows: rows,
        orderedFamilies: families,
        colorMap: colorMap,
        hoveredFamily: .constant(nil)
    )
    .padding()
    .frame(width: 480)
}
