import Charts
import HeimdallDomain
import SwiftUI

/// 30-day running-sum cost curve ("budget pace").
/// Each entry accumulates all prior days' costs. The curve is monotonically
/// non-decreasing by construction. Days with unparseable date strings are
/// skipped; the running sum continues across gaps.
struct CumulativeSpendChart: View {
    let daily: [CostHistoryPoint]
    @State private var selectedDay: Date?

    struct Entry: Identifiable, Hashable {
        let day: Date
        let cumulativeCostUSD: Double
        var id: Date { self.day }
    }

    var body: some View {
        let entries = Self.entries(from: self.daily)
        VStack(alignment: .leading, spacing: 6) {
            ChartHeader(
                title: "Cumulative spend, 30 days",
                caption: "Running total. The line is always monotonic."
            )
            if entries.isEmpty {
                Text("No daily data yet.")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .padding(.vertical, 12)
            } else {
                self.chart(entries: entries)
                    .frame(height: 72)
            }
        }
        .padding(8)
        .menuCardBackground(
            opacity: ChartStyle.cardBackgroundOpacity,
            cornerRadius: ChartStyle.cardCornerRadius
        )
        .accessibilityElement(children: .combine)
        .accessibilityLabel("Cumulative spend, last \(entries.count) days")
    }

    private func chart(entries: [Entry]) -> some View {
        let selectedEntry = self.selectedDay.flatMap { Self.nearestEntry(to: $0, in: entries) }
        let selectedIndex = selectedEntry.flatMap { entries.firstIndex(of: $0) }
        return Chart {
            ForEach(entries) { entry in
                AreaMark(
                    x: .value("Day", entry.day),
                    y: .value("Cumulative cost", entry.cumulativeCostUSD)
                )
                .foregroundStyle(ChartStyle.areaFill)
                .interpolationMethod(.monotone)
            }
            ForEach(entries) { entry in
                LineMark(
                    x: .value("Day", entry.day),
                    y: .value("Cumulative cost", entry.cumulativeCostUSD)
                )
                .foregroundStyle(ChartStyle.lineStroke)
                .lineStyle(StrokeStyle(lineWidth: ChartStyle.lineWidth, lineCap: .round, lineJoin: .round))
                .interpolationMethod(.monotone)
            }
            if let selectedEntry, let selectedIndex {
                RuleMark(x: .value("Selected day", selectedEntry.day))
                    .foregroundStyle(Color.primary.opacity(0.3))
                    .lineStyle(StrokeStyle(lineWidth: 1))
                    .annotation(
                        position: ChartStyle.inspectorPlacement(index: selectedIndex, totalCount: entries.count).annotationPosition,
                        spacing: 6,
                        overflowResolution: .init(x: .fit(to: .chart), y: .fit(to: .chart))
                    ) {
                        ChartInspectorCard(
                            title: Self.axisFormatter.string(from: selectedEntry.day),
                            lines: [Self.currencyLabel(selectedEntry.cumulativeCostUSD)]
                        )
                    }

                PointMark(
                    x: .value("Selected day", selectedEntry.day),
                    y: .value("Cumulative cost", selectedEntry.cumulativeCostUSD)
                )
                .foregroundStyle(Color.accentColor)
                .symbolSize(30)
            }
        }
        .chartYScale(domain: .automatic(includesZero: true))
        .chartYAxis(.hidden)
        .chartXAxis {
            AxisMarks(values: .stride(by: .day, count: 7)) { value in
                AxisValueLabel {
                    if let date = value.as(Date.self) {
                        Text(Self.axisFormatter.string(from: date))
                            .font(.system(size: 9).monospacedDigit())
                            .foregroundStyle(.secondary)
                    }
                }
                AxisTick(stroke: StrokeStyle(lineWidth: 0.5))
                    .foregroundStyle(Color.primary.opacity(0.15))
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
                                let day = proxy.value(atX: x, as: Date.self),
                                let nearest = Self.nearestEntry(to: day, in: entries),
                                let snappedX = proxy.position(forX: nearest.day),
                                abs(snappedX - x) <= ChartStyle.snapThreshold(
                                    plotWidth: proxy.plotSize.width,
                                    itemCount: entries.count
                                )
                            else {
                                ChartStyle.updateHoverSelection(&self.selectedDay, to: nil)
                                return
                            }
                            ChartStyle.updateHoverSelection(&self.selectedDay, to: nearest.day)
                        case .ended:
                            ChartStyle.updateHoverSelection(&self.selectedDay, to: nil)
                        }
                    }
            }
        }
        .help(Self.tooltip(for: entries))
        .animation(ChartStyle.animation, value: entries)
        .animation(ChartStyle.hoverAnimation, value: self.selectedDay)
    }

    nonisolated static func entries(from daily: [CostHistoryPoint]) -> [Entry] {
        var running = 0.0
        return daily.compactMap { point in
            guard let date = Self.dayFormatter.date(from: point.day) else { return nil }
            running += point.costUSD
            return Entry(day: date, cumulativeCostUSD: running)
        }
    }

    nonisolated static func tooltip(for entries: [Entry]) -> String {
        entries.map { entry in
            "\(Self.axisFormatter.string(from: entry.day)): \(Self.currencyLabel(entry.cumulativeCostUSD))"
        }
        .joined(separator: "\n")
    }

    nonisolated static func nearestEntry(to day: Date, in entries: [Entry]) -> Entry? {
        entries.min { lhs, rhs in
            abs(lhs.day.timeIntervalSince(day)) < abs(rhs.day.timeIntervalSince(day))
        }
    }

    nonisolated private static func currencyLabel(_ usd: Double) -> String {
        String(format: "$%.2f", usd)
    }

    nonisolated(unsafe) private static let dayFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy-MM-dd"
        formatter.timeZone = TimeZone(secondsFromGMT: 0)
        formatter.locale = Locale(identifier: "en_US_POSIX")
        return formatter
    }()

    nonisolated(unsafe) private static let axisFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "MMM d"
        return formatter
    }()
}

#Preview("Cumulative spend — plateaus and spikes") {
    let formatter = DateFormatter()
    formatter.dateFormat = "yyyy-MM-dd"
    formatter.timeZone = TimeZone(secondsFromGMT: 0)
    formatter.locale = Locale(identifier: "en_US_POSIX")
    let base = Date()
    let calendar = Calendar.current
    // Alternating active days (cost > 0) and plateau days (cost = 0) to show
    // monotonicity clearly: curve steps up then holds flat.
    let dailyCosts: [Double] = [
        3.2, 0.0, 0.0, 5.8, 1.1, 0.0, 4.4,
        2.9, 0.0, 0.0, 6.1, 0.0, 3.3, 1.7,
        0.0, 4.8, 0.0, 0.0, 7.2, 2.5, 0.0,
        3.6, 1.4, 0.0, 5.5, 0.0, 2.1, 4.0,
        0.0, 8.3,
    ]
    let points: [CostHistoryPoint] = dailyCosts.enumerated().map { offset, cost in
        let date = calendar.date(byAdding: .day, value: -(29 - offset), to: base) ?? base
        return CostHistoryPoint(
            day: formatter.string(from: date),
            totalTokens: Int(cost * 1_000),
            costUSD: cost
        )
    }
    return CumulativeSpendChart(daily: points)
        .padding()
        .frame(width: 360)
}
