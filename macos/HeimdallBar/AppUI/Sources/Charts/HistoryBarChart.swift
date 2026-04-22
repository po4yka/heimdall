import Charts
import SwiftUI

/// 7-day (or N-day) spend bars using Swift Charts. Replaces the hand-rolled
/// `HistoryBarStrip` fractions mode. Y-domain is fixed 0...1 because the
/// caller already normalizes each day to the window peak, matching the
/// previous "share of peak" semantics.
struct HistoryBarChart: View {
    let fractions: [Double]
    var showsHeader: Bool = true

    struct Entry: Identifiable, Hashable {
        let index: Int
        let label: String
        let fraction: Double

        var id: Int { self.index }
        var isToday: Bool { false }
    }

    var body: some View {
        let entries = Self.entries(from: self.fractions)
        VStack(alignment: .leading, spacing: 6) {
            if self.showsHeader {
                ChartHeader(
                    title: "Usage history",
                    caption: "Daily spend, last 7 days."
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
        .accessibilityLabel("Usage history, last \(entries.count) days")
    }

    private func chart(entries: [Entry]) -> some View {
        Chart(entries) { entry in
            BarMark(
                x: .value("Day", entry.label),
                y: .value("Fraction", entry.fraction)
            )
            .foregroundStyle(self.isToday(entry, in: entries) ? ChartStyle.barTodayFill : ChartStyle.barFill)
            .cornerRadius(ChartStyle.barCornerRadius)
            .accessibilityLabel(entry.label)
            .accessibilityValue("\(Int((entry.fraction * 100).rounded())) percent of peak")
        }
        .chartYScale(domain: 0...1)
        .chartYAxis(.hidden)
        .chartXAxis {
            AxisMarks(values: entries.map(\.label)) { value in
                AxisValueLabel {
                    if let label = value.as(String.self) {
                        let today = entries.last?.label
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

    private func isToday(_ entry: Entry, in entries: [Entry]) -> Bool {
        entry.index == entries.count - 1
    }

    nonisolated static func entries(from fractions: [Double]) -> [Entry] {
        let labels = Self.dayLabels(count: fractions.count)
        return fractions.enumerated().map { offset, fraction in
            let isLast = offset == fractions.count - 1
            return Entry(
                index: offset,
                label: isLast ? "Today" : labels[offset],
                fraction: max(0, min(1, fraction))
            )
        }
    }

    nonisolated private static func dayLabels(count: Int) -> [String] {
        ChartDayLabels.lastNDays(count)
    }
}

#Preview("History — 7 days") {
    HistoryBarChart(fractions: [0.12, 0.34, 0.08, 0.66, 0.9, 0.4, 0.72])
        .padding()
        .frame(width: 320)
}

#Preview("History — no header") {
    HistoryBarChart(
        fractions: [0.05, 0.2, 0.45, 0.6, 0.3, 0.85, 1.0],
        showsHeader: false
    )
    .padding()
    .frame(width: 320)
}
