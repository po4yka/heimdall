import Charts
import HeimdallDomain
import SwiftUI

/// 30-day daily cache hit rate line + area chart.
/// Rate = cacheRead / (cacheRead + cacheCreation + input). Days where
/// `breakdown` is nil or the denominator is zero are skipped. Fewer than 2
/// computable points renders an empty-state caption.
struct CacheHitTrendChart: View {
    let daily: [CostHistoryPoint]
    @State private var selectedDay: Date?

    struct Entry: Identifiable, Hashable {
        let day: Date
        let rate: Double
        var id: Date { self.day }
    }

    var body: some View {
        let entries = Self.entries(from: self.daily)
        VStack(alignment: .leading, spacing: 6) {
            ChartHeader(
                title: "Cache hit rate, 30 days",
                caption: "Higher is better. Ratio uses cache reads vs. input + cache writes."
            )
            if entries.count < 2 {
                Text("Cache hit rate is not available yet.")
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
        .accessibilityLabel("Cache hit rate, last \(entries.count) days")
    }

    private func chart(entries: [Entry]) -> some View {
        let lastDay = entries.last?.day
        let selectedEntry = self.selectedDay.flatMap { Self.nearestEntry(to: $0, in: entries) }
        let selectedIndex = selectedEntry.flatMap { entries.firstIndex(of: $0) }
        return Chart {
            ForEach(entries) { entry in
                AreaMark(
                    x: .value("Day", entry.day),
                    y: .value("Rate", entry.rate)
                )
                .foregroundStyle(ChartStyle.areaFill)
                .interpolationMethod(.monotone)
            }
            ForEach(entries) { entry in
                LineMark(
                    x: .value("Day", entry.day),
                    y: .value("Rate", entry.rate)
                )
                .foregroundStyle(ChartStyle.lineStroke)
                .lineStyle(StrokeStyle(lineWidth: ChartStyle.lineWidth, lineCap: .round, lineJoin: .round))
                .interpolationMethod(.monotone)
            }
            if let lastDay = lastDay {
                RuleMark(x: .value("Last", lastDay))
                    .foregroundStyle(ChartStyle.todayRuleStroke)
                    .lineStyle(StrokeStyle(lineWidth: ChartStyle.todayRuleWidth, dash: [2, 2]))
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
                            lines: [String(format: "%.1f%%", selectedEntry.rate * 100)]
                        )
                    }

                PointMark(
                    x: .value("Selected day", selectedEntry.day),
                    y: .value("Rate", selectedEntry.rate)
                )
                .foregroundStyle(Color.accentColor)
                .symbolSize(30)
            }
        }
        .chartYScale(domain: 0...1)
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
        daily.compactMap { point in
            guard
                let breakdown = point.breakdown,
                let date = Self.dayFormatter.date(from: point.day)
            else { return nil }
            let denominator = breakdown.cacheRead + breakdown.cacheCreation + breakdown.input
            guard denominator > 0 else { return nil }
            let rate = Double(breakdown.cacheRead) / Double(denominator)
            return Entry(day: date, rate: rate)
        }
    }

    nonisolated static func tooltip(for entries: [Entry]) -> String {
        entries.map { entry in
            "\(Self.axisFormatter.string(from: entry.day)): \(String(format: "%.1f%%", entry.rate * 100))"
        }
        .joined(separator: "\n")
    }

    nonisolated static func nearestEntry(to day: Date, in entries: [Entry]) -> Entry? {
        entries.min { lhs, rhs in
            abs(lhs.day.timeIntervalSince(day)) < abs(rhs.day.timeIntervalSince(day))
        }
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

#Preview("Cache hit rate — rising trend") {
    let formatter = DateFormatter()
    formatter.dateFormat = "yyyy-MM-dd"
    formatter.timeZone = TimeZone(secondsFromGMT: 0)
    formatter.locale = Locale(identifier: "en_US_POSIX")
    let base = Date()
    let calendar = Calendar.current
    let points: [CostHistoryPoint] = (0..<30).reversed().map { offset in
        let date = calendar.date(byAdding: .day, value: -offset, to: base) ?? base
        let progress = Double(30 - offset) / 30.0
        // Rising cache hit rate: starts ~20%, ends ~75%, with small noise
        let cacheRead = Int((progress * 0.75 + Double(offset % 3) * 0.02) * 10_000)
        let input = 5_000
        let cacheCreation = 2_000
        let breakdown = TokenBreakdown(
            input: input,
            output: 1_000,
            cacheRead: cacheRead,
            cacheCreation: cacheCreation
        )
        return CostHistoryPoint(
            day: formatter.string(from: date),
            totalTokens: input + cacheRead + cacheCreation + 1_000,
            costUSD: 1.5 + progress * 8.0,
            breakdown: breakdown
        )
    }
    return CacheHitTrendChart(daily: points)
        .padding()
        .frame(width: 360)
}
