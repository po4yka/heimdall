import Charts
import SwiftUI

/// Tiny inline sparkline for 7-day spend trajectory. Designed to sit inside an
/// existing HStack — no card, no header, no padding of its own.
struct SpendSparkline: View {
    let fractions: [Double]
    var width: CGFloat = 48
    var height: CGFloat = 14

    struct Entry: Identifiable, Hashable {
        let index: Int
        let fraction: Double
        var id: Int { self.index }
    }

    var body: some View {
        if self.fractions.count < 2 {
            EmptyView()
        } else {
            let entries = Self.entries(from: self.fractions)
            let lastIndex = entries[entries.count - 1].index
            let lastFraction = entries[entries.count - 1].fraction
            Chart {
                ForEach(entries) { entry in
                    AreaMark(
                        x: .value("Day", entry.index),
                        y: .value("Fraction", entry.fraction)
                    )
                    .foregroundStyle(ChartStyle.areaFill)
                    .interpolationMethod(.monotone)
                }
                ForEach(entries) { entry in
                    LineMark(
                        x: .value("Day", entry.index),
                        y: .value("Fraction", entry.fraction)
                    )
                    .foregroundStyle(ChartStyle.lineStroke)
                    .lineStyle(StrokeStyle(lineWidth: 1.0, lineCap: .round, lineJoin: .round))
                    .interpolationMethod(.monotone)
                }
                PointMark(
                    x: .value("Day", lastIndex),
                    y: .value("Fraction", lastFraction)
                )
                .foregroundStyle(Color.accentColor.opacity(0.95))
                .symbolSize(14)
            }
            .chartXAxis(.hidden)
            .chartYAxis(.hidden)
            .chartLegend(.hidden)
            .chartYScale(domain: 0...1)
            .chartPlotStyle { plot in
                plot.background(Color.clear)
            }
            .frame(width: self.width, height: self.height)
            .help(Self.tooltip(for: entries))
            .accessibilityLabel("Spend sparkline, last \(self.fractions.count) days")
        }
    }

    nonisolated static func entries(from fractions: [Double]) -> [Entry] {
        fractions.enumerated().map { offset, fraction in
            Entry(index: offset, fraction: max(0, min(1, fraction)))
        }
    }

    nonisolated static func tooltip(for entries: [Entry]) -> String {
        let labels = ChartDayLabels.lastNDays(entries.count)
        return entries.enumerated().map { offset, entry in
            let label = offset == entries.count - 1 ? "Today" : labels[offset]
            return "\(label): \(Int((entry.fraction * 100).rounded()))% of peak"
        }
        .joined(separator: "\n")
    }
}

#Preview("Sparkline — standalone") {
    SpendSparkline(fractions: [0.2, 0.5, 0.3, 0.8, 0.6, 1.0, 0.9])
        .padding(4)
        .frame(width: 56, height: 22)
        .background(Color.primary.opacity(0.06))
        .cornerRadius(4)
        .padding()
}

#Preview("Sparkline — inline HStack") {
    HStack(spacing: 4) {
        Text("Today: $12.40")
            .font(.caption)
        SpendSparkline(fractions: [0.2, 0.5, 0.3, 0.8, 0.6, 1.0, 0.9])
        Text("↓")
            .font(.caption)
            .foregroundStyle(.secondary)
    }
    .padding()
}
