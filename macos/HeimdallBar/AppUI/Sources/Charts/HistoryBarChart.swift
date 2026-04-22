import Charts
import SwiftUI

/// 7-day (or N-day) spend bars using Swift Charts. Replaces the hand-rolled
/// `HistoryBarStrip` fractions mode. Y-domain is fixed 0...1 because the
/// caller already normalizes each day to the window peak, matching the
/// previous "share of peak" semantics.
struct HistoryBarChart: View {
    let fractions: [Double]
    var showsHeader: Bool = true
    /// When true, the chart renders without its own card background or
    /// padding. Use from a parent view that already provides a card —
    /// otherwise you end up with a card-in-a-card.
    var inset: Bool = false
    @State private var selectedIndex: Int?

    struct Entry: Identifiable, Hashable {
        let index: Int
        let label: String
        let fraction: Double

        var id: Int { self.index }
    }

    /// Minimum on-screen bar height so zero-spend days still render a
    /// visible stub and the axis doesn't look broken. The real fraction
    /// is still reported to VoiceOver via `accessibilityValue`.
    private static let minimumVisibleFraction: Double = 0.04

    var body: some View {
        let entries = Self.entries(from: self.fractions)
        let content = VStack(alignment: .leading, spacing: 6) {
            if self.showsHeader {
                ChartHeader(
                    title: "Usage history",
                    caption: "Daily spend, last 7 days."
                )
            }
            self.chart(entries: entries)
                .frame(height: 48)
        }
        .accessibilityElement(children: .combine)
        .accessibilityLabel("Usage history, last \(entries.count) days")

        if self.inset {
            content
        } else {
            content
                .padding(8)
                .menuCardBackground(
                    opacity: ChartStyle.cardBackgroundOpacity,
                    cornerRadius: ChartStyle.cardCornerRadius
                )
        }
    }

    private func chart(entries: [Entry]) -> some View {
        let selectedEntry = self.selectedIndex.flatMap { index in
            entries.first(where: { $0.index == index })
        }
        return Chart {
            ForEach(entries) { entry in
                BarMark(
                    x: .value("Day", entry.index),
                    y: .value("Fraction", max(Self.minimumVisibleFraction, entry.fraction))
                )
                .foregroundStyle(self.barTint(for: entry, in: entries))
                .cornerRadius(ChartStyle.barCornerRadius)
                .accessibilityLabel(entry.label)
                .accessibilityValue("\(Int((entry.fraction * 100).rounded())) percent of peak")
            }
            if let selectedEntry {
                RuleMark(x: .value("Day", selectedEntry.index))
                    .foregroundStyle(Color.primary.opacity(0.3))
                    .lineStyle(StrokeStyle(lineWidth: 1))
                    .annotation(
                        position: ChartStyle.inspectorPlacement(index: selectedEntry.index, totalCount: entries.count).annotationPosition,
                        spacing: 6,
                        overflowResolution: .init(x: .fit(to: .chart), y: .fit(to: .chart))
                    ) {
                        ChartInspectorCard(
                            title: selectedEntry.label,
                            lines: ["\(Int((selectedEntry.fraction * 100).rounded()))% of peak"]
                        )
                    }
            }
        }
        .chartYScale(domain: 0...1)
        .chartYAxis(.hidden)
        .chartXAxis {
            AxisMarks(values: entries.map(\.index)) { value in
                AxisValueLabel {
                    if let index = value.as(Int.self),
                       let entry = entries.first(where: { $0.index == index }) {
                        let label = entry.label
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
        .chartOverlay { proxy in
            GeometryReader { geometry in
                Rectangle()
                    .fill(Color.clear)
                    .contentShape(Rectangle())
                    .onContinuousHover { phase in
                        let plotFrame = geometry[proxy.plotFrame!]
                        switch phase {
                        case .active(let location):
                            let x = location.x - plotFrame.origin.x
                            guard
                                x >= 0,
                                x <= proxy.plotSize.width,
                                let rawIndex = proxy.value(atX: x, as: Int.self)
                            else {
                                ChartStyle.updateHoverSelection(&self.selectedIndex, to: nil)
                                return
                            }
                            let index = min(max(rawIndex, 0), max(entries.count - 1, 0))
                            guard
                                let snappedX = proxy.position(forX: index),
                                abs(snappedX - x) <= ChartStyle.snapThreshold(
                                    plotWidth: proxy.plotSize.width,
                                    itemCount: entries.count
                                )
                            else {
                                ChartStyle.updateHoverSelection(&self.selectedIndex, to: nil)
                                return
                            }
                            ChartStyle.updateHoverSelection(&self.selectedIndex, to: index)
                        case .ended:
                            ChartStyle.updateHoverSelection(&self.selectedIndex, to: nil)
                        }
                    }
            }
        }
        .help(Self.tooltip(for: entries))
        .animation(ChartStyle.animation, value: entries)
        .animation(ChartStyle.hoverAnimation, value: self.selectedIndex)
    }

    private func isToday(_ entry: Entry, in entries: [Entry]) -> Bool {
        entry.index == entries.count - 1
    }

    private func barTint(for entry: Entry, in entries: [Entry]) -> Color {
        if self.selectedIndex == entry.index {
            return Color.accentColor
        }
        return self.isToday(entry, in: entries) ? ChartStyle.barTodayFill : ChartStyle.barFill
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

    nonisolated static func tooltip(for entries: [Entry]) -> String {
        entries.map { entry in
            "\(entry.label): \(Int((entry.fraction * 100).rounded()))% of peak"
        }
        .joined(separator: "\n")
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
