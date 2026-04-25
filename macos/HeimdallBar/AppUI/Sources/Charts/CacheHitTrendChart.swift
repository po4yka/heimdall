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

    struct TrendEntry: Identifiable, Hashable {
        let day: Date
        let rate: Double
        var id: Date { self.day }
    }

    var body: some View {
        let entries = Self.entries(from: self.daily)
        let movingAverage = Self.movingAverageEntries(from: entries)
        let averageRate = Self.averageRate(from: entries)
        VStack(alignment: .leading, spacing: 6) {
            ChartHeader(
                title: "Cache hit rate, 30 days",
                caption: "Higher is better. Dashed line = 7-day average; horizontal rule = 30-day average."
            )
            if entries.count < 2 {
                Text("Cache hit rate is not available yet.")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .padding(.vertical, 12)
            } else {
                self.chart(entries: entries, movingAverage: movingAverage, averageRate: averageRate)
                    .frame(height: 84)
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

    private func chart(entries: [Entry], movingAverage: [TrendEntry], averageRate: Double?) -> some View {
        let lastDay = entries.last?.day
        let selectedEntry = self.selectedDay.flatMap { Self.nearestEntry(to: $0, in: entries) }
        let selectedIndex = selectedEntry.flatMap { entries.firstIndex(of: $0) }
        let selectedAverage = selectedEntry.flatMap { entry in
            movingAverage.first(where: { $0.day == entry.day })
        }
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
            ForEach(movingAverage) { entry in
                LineMark(
                    x: .value("Day", entry.day),
                    y: .value("7-day average", entry.rate)
                )
                .foregroundStyle(ChartStyle.secondaryLineStroke)
                .lineStyle(StrokeStyle(lineWidth: ChartStyle.secondaryLineWidth, lineCap: .round, lineJoin: .round, dash: [4, 3]))
                .interpolationMethod(.monotone)
            }
            if let averageRate {
                RuleMark(y: .value("Average rate", averageRate))
                    .foregroundStyle(ChartStyle.referenceRuleStroke)
                    .lineStyle(StrokeStyle(lineWidth: ChartStyle.referenceRuleWidth, dash: [2, 3]))
            }
            if let lastDay = lastDay {
                RuleMark(x: .value("Last", lastDay))
                    .foregroundStyle(ChartStyle.todayRuleStroke)
                    .lineStyle(StrokeStyle(lineWidth: ChartStyle.todayRuleWidth, dash: [2, 2]))
            }
            if let selectedEntry, let selectedIndex {
                RuleMark(x: .value("Selected day", selectedEntry.day))
                    .foregroundStyle(Color.primary.opacity(0.34))
                    .lineStyle(StrokeStyle(lineWidth: 1))
                RuleMark(y: .value("Selected rate", selectedEntry.rate))
                    .foregroundStyle(Color.primary.opacity(0.16))
                    .lineStyle(StrokeStyle(lineWidth: 1, dash: [2, 3]))
                    .annotation(
                        position: ChartStyle.inspectorPlacement(index: selectedIndex, totalCount: entries.count).annotationPosition,
                        spacing: 6,
                        overflowResolution: .init(x: .fit(to: .chart), y: .fit(to: .chart))
                    ) {
                        ChartInspectorCard(
                            title: Self.axisFormatter.string(from: selectedEntry.day),
                            lines: Self.inspectorLines(
                                for: selectedEntry,
                                movingAverage: selectedAverage,
                                averageRate: averageRate
                            )
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
            ChartStyle.framedPlot(plot, verticalPadding: 4)
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

    nonisolated static func movingAverageEntries(from entries: [Entry], windowSize: Int = 7) -> [TrendEntry] {
        guard windowSize > 0 else { return [] }
        return entries.indices.map { index in
            let start = max(0, index - (windowSize - 1))
            let window = entries[start...index]
            let average = window.reduce(0.0) { $0 + $1.rate } / Double(window.count)
            return TrendEntry(day: entries[index].day, rate: average)
        }
    }

    nonisolated static func averageRate(from entries: [Entry]) -> Double? {
        guard !entries.isEmpty else { return nil }
        return entries.reduce(0.0) { $0 + $1.rate } / Double(entries.count)
    }

    nonisolated static func inspectorLines(
        for entry: Entry,
        movingAverage: TrendEntry?,
        averageRate: Double?
    ) -> [String] {
        var lines = [Self.rateLabel(entry.rate)]
        if let movingAverage {
            lines.append("7d avg \(Self.rateLabel(movingAverage.rate))")
        }
        if let averageRate {
            lines.append(Self.rateDeltaLabel(entry.rate - averageRate, suffix: "vs window avg"))
        }
        return lines
    }

    nonisolated private static func rateLabel(_ rate: Double) -> String {
        String(format: "%.1f%%", rate * 100)
    }

    nonisolated private static func rateDeltaLabel(_ delta: Double, suffix: String) -> String {
        let points = delta * 100
        if abs(points) < 0.05 {
            return "Flat \(suffix)"
        }
        return String(format: "%@%.1f pt %@", points >= 0 ? "+" : "-", abs(points), suffix)
    }

    private static let dayFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy-MM-dd"
        formatter.timeZone = TimeZone(secondsFromGMT: 0)
        formatter.locale = Locale(identifier: "en_US_POSIX")
        return formatter
    }()

    private static let axisFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "MMM d"
        return formatter
    }()
}

private func cacheHitTrendPreviewPoint(
    offset: Int,
    base: Date,
    calendar: Calendar,
    formatter: DateFormatter
) -> CostHistoryPoint {
    let date: Date = calendar.date(byAdding: .day, value: -offset, to: base) ?? base
    let progress: Double = Double(30 - offset) / 30.0
    // Rising cache hit rate: starts ~20%, ends ~75%, with small noise
    let cacheRead: Int = Int((progress * 0.75 + Double(offset % 3) * 0.02) * 10_000)
    let input: Int = 5_000
    let output: Int = 1_000
    let cacheCreation: Int = 2_000
    let extra: Int = 1_000
    let breakdown = TokenBreakdown(
        input: input,
        output: output,
        cacheRead: cacheRead,
        cacheCreation: cacheCreation
    )
    let totalTokens: Int = input + cacheRead + cacheCreation + extra
    let costUSD: Double = 1.5 + progress * 8.0
    return CostHistoryPoint(
        day: formatter.string(from: date),
        totalTokens: totalTokens,
        costUSD: costUSD,
        breakdown: breakdown
    )
}

#Preview("Cache hit rate — rising trend") {
    let formatter = DateFormatter()
    formatter.dateFormat = "yyyy-MM-dd"
    formatter.timeZone = TimeZone(secondsFromGMT: 0)
    formatter.locale = Locale(identifier: "en_US_POSIX")
    let base = Date()
    let calendar = Calendar.current
    let points: [CostHistoryPoint] = (0..<30).reversed().map { offset in
        cacheHitTrendPreviewPoint(
            offset: offset,
            base: base,
            calendar: calendar,
            formatter: formatter
        )
    }
    return CacheHitTrendChart(daily: points)
        .padding()
        .frame(width: 360)
}
