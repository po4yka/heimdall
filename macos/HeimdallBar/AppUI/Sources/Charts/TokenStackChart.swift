import Charts
import HeimdallDomain
import SwiftUI

/// 7-day (or N-day) stacked bars, one stack per day, colored by token
/// category. Replaces the hand-rolled `StackedDayBar`. Layout matches
/// `HistoryBarChart` so the two can swap places based on whether the
/// underlying snapshot has category breakdowns available.
struct TokenStackChart: View {
    let breakdowns: [TokenBreakdown]
    var showsHeader: Bool = true

    struct Entry: Identifiable, Hashable {
        let dayIndex: Int
        let dayLabel: String
        let category: TokenCategory
        let tokens: Int

        var id: String { "\(self.dayIndex)-\(self.category.label)" }
    }

    var body: some View {
        let entries = Self.entries(from: self.breakdowns)
        VStack(alignment: .leading, spacing: 6) {
            if self.showsHeader {
                ChartHeader(
                    title: "Usage history",
                    caption: "Daily spend by category, last 7 days."
                )
            }
            self.chart(entries: entries)
                .frame(height: 48)
        }
        .padding(8)
        .menuCardBackground(
            opacity: ChartStyle.cardBackgroundOpacity,
            cornerRadius: ChartStyle.cardCornerRadius
        )
        .accessibilityElement(children: .combine)
        .accessibilityLabel("Usage history by category, last \(self.breakdowns.count) days")
    }

    private func chart(entries: [Entry]) -> some View {
        let labels = Self.dayLabels(count: self.breakdowns.count)
        return Chart(entries) { entry in
            BarMark(
                x: .value("Day", entry.dayLabel),
                y: .value("Tokens", entry.tokens)
            )
            .foregroundStyle(by: .value("Category", entry.category.label))
            .accessibilityLabel(entry.dayLabel)
            .accessibilityValue("\(entry.category.label): \(entry.tokens) tokens")
        }
        .chartForegroundStyleScale(
            domain: TokenCategory.orderedForStack.map(\.label),
            range: ChartStyle.categoryScale
        )
        .chartLegend(.hidden)
        .chartYAxis(.hidden)
        .chartXAxis {
            AxisMarks(values: labels) { value in
                AxisValueLabel {
                    if let label = value.as(String.self) {
                        let today = labels.last
                        Text(label)
                            .font(.system(size: 9, weight: label == today ? .semibold : .regular).monospacedDigit())
                            .foregroundStyle(label == today ? .primary : .secondary)
                    }
                }
            }
        }
        .chartPlotStyle { plot in
            plot.background(Color.clear)
        }
        .animation(ChartStyle.animation, value: entries)
    }

    nonisolated static func entries(from breakdowns: [TokenBreakdown]) -> [Entry] {
        let labels = Self.dayLabels(count: breakdowns.count)
        var result: [Entry] = []
        result.reserveCapacity(breakdowns.count * TokenCategory.orderedForStack.count)
        for (offset, breakdown) in breakdowns.enumerated() {
            let isLast = offset == breakdowns.count - 1
            let label = isLast ? "Today" : labels[offset]
            for category in TokenCategory.orderedForStack {
                let tokens = category.value(for: breakdown)
                if tokens > 0 {
                    result.append(
                        Entry(
                            dayIndex: offset,
                            dayLabel: label,
                            category: category,
                            tokens: tokens
                        )
                    )
                }
            }
        }
        return result
    }

    nonisolated private static func dayLabels(count: Int) -> [String] {
        let raw = ChartDayLabels.lastNDays(count)
        guard !raw.isEmpty else { return raw }
        var labels = raw
        labels[labels.count - 1] = "Today"
        return labels
    }
}

#Preview("Stack — 7 days") {
    let sample = [
        TokenBreakdown(input: 1_200, output: 800, cacheRead: 4_500, cacheCreation: 300, reasoningOutput: 0),
        TokenBreakdown(input: 900, output: 2_100, cacheRead: 8_200, cacheCreation: 450, reasoningOutput: 120),
        TokenBreakdown(input: 0, output: 0, cacheRead: 0, cacheCreation: 0, reasoningOutput: 0),
        TokenBreakdown(input: 1_500, output: 1_800, cacheRead: 12_000, cacheCreation: 500, reasoningOutput: 50),
        TokenBreakdown(input: 2_200, output: 3_500, cacheRead: 18_000, cacheCreation: 900, reasoningOutput: 200),
        TokenBreakdown(input: 1_800, output: 2_200, cacheRead: 10_000, cacheCreation: 600, reasoningOutput: 150),
        TokenBreakdown(input: 2_500, output: 4_000, cacheRead: 20_000, cacheCreation: 1_100, reasoningOutput: 300),
    ]
    return TokenStackChart(breakdowns: sample)
        .padding()
        .frame(width: 320)
}
