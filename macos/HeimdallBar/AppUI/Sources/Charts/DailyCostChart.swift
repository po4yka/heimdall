import Charts
import HeimdallDomain
import SwiftUI

/// 30-day daily cost line + area chart. New visualization unlocked by Swift
/// Charts: the menu previously only had the normalized 7-day strip; the full
/// `CostHistoryPoint` series has always been in the snapshot but was not
/// rendered anywhere. Today's point is marked by an accent rule.
struct DailyCostChart: View {
    let daily: [CostHistoryPoint]
    @State private var selectedDay: Date?

    struct Entry: Identifiable, Hashable {
        let day: Date
        let costUSD: Double
        var id: Date { self.day }
    }

    struct TrendEntry: Identifiable, Hashable {
        let day: Date
        let costUSD: Double
        var id: Date { self.day }
    }

    var body: some View {
        let entries = Self.entries(from: self.daily)
        let movingAverage = Self.movingAverageEntries(from: entries)
        let windowAverage = Self.averageCost(from: entries)
        VStack(alignment: .leading, spacing: 6) {
            ChartHeader(
                title: "Daily cost",
                caption: "Last \(entries.count) days. Dashed line = 7-day average; horizontal rule = window average."
            )
            if entries.isEmpty {
                Text("No daily data yet.")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .padding(.vertical, 12)
            } else {
                self.chart(entries: entries, movingAverage: movingAverage, windowAverage: windowAverage)
                    .frame(height: 84)
            }
        }
        .padding(8)
        .menuCardBackground(
            opacity: ChartStyle.cardBackgroundOpacity,
            cornerRadius: ChartStyle.cardCornerRadius
        )
        .accessibilityElement(children: .combine)
        .accessibilityLabel("Daily cost, last \(entries.count) days")
    }

    private func chart(entries: [Entry], movingAverage: [TrendEntry], windowAverage: Double?) -> some View {
        let today = entries.last?.day
        let selectedEntry = self.selectedDay.flatMap { Self.nearestEntry(to: $0, in: entries) }
        let selectedIndex = selectedEntry.flatMap { entries.firstIndex(of: $0) }
        let selectedAverage = selectedEntry.flatMap { entry in
            movingAverage.first(where: { $0.day == entry.day })
        }
        return Chart {
            ForEach(entries) { entry in
                AreaMark(
                    x: .value("Day", entry.day),
                    y: .value("Cost", entry.costUSD)
                )
                .foregroundStyle(ChartStyle.areaFill)
                .interpolationMethod(.monotone)
            }
            ForEach(entries) { entry in
                LineMark(
                    x: .value("Day", entry.day),
                    y: .value("Cost", entry.costUSD)
                )
                .foregroundStyle(ChartStyle.lineStroke)
                .lineStyle(StrokeStyle(lineWidth: ChartStyle.lineWidth, lineCap: .round, lineJoin: .round))
                .interpolationMethod(.monotone)
            }
            ForEach(movingAverage) { entry in
                LineMark(
                    x: .value("Day", entry.day),
                    y: .value("7-day average", entry.costUSD)
                )
                .foregroundStyle(ChartStyle.secondaryLineStroke)
                .lineStyle(StrokeStyle(lineWidth: ChartStyle.secondaryLineWidth, lineCap: .round, lineJoin: .round, dash: [4, 3]))
                .interpolationMethod(.monotone)
            }
            if let windowAverage {
                RuleMark(y: .value("Window average", windowAverage))
                    .foregroundStyle(ChartStyle.referenceRuleStroke)
                    .lineStyle(StrokeStyle(lineWidth: ChartStyle.referenceRuleWidth, dash: [2, 3]))
            }
            if let today = today {
                RuleMark(x: .value("Today", today))
                    .foregroundStyle(ChartStyle.todayRuleStroke)
                    .lineStyle(StrokeStyle(lineWidth: ChartStyle.todayRuleWidth, dash: [2, 2]))
            }
            if let selectedEntry, let selectedIndex {
                RuleMark(x: .value("Selected day", selectedEntry.day))
                    .foregroundStyle(Color.primary.opacity(0.34))
                    .lineStyle(StrokeStyle(lineWidth: 1))
                RuleMark(y: .value("Selected cost", selectedEntry.costUSD))
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
                                windowAverage: windowAverage
                            )
                        )
                    }

                PointMark(
                    x: .value("Selected day", selectedEntry.day),
                    y: .value("Cost", selectedEntry.costUSD)
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
            guard let date = Self.dayFormatter.date(from: point.day) else {
                return nil
            }
            return Entry(day: date, costUSD: point.costUSD)
        }
    }

    nonisolated static func tooltip(for entries: [Entry]) -> String {
        entries.map { entry in
            "\(Self.axisFormatter.string(from: entry.day)): \(Self.currencyLabel(entry.costUSD))"
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
            let average = window.reduce(0.0) { $0 + $1.costUSD } / Double(window.count)
            return TrendEntry(day: entries[index].day, costUSD: average)
        }
    }

    nonisolated static func averageCost(from entries: [Entry]) -> Double? {
        guard !entries.isEmpty else { return nil }
        return entries.reduce(0.0) { $0 + $1.costUSD } / Double(entries.count)
    }

    nonisolated static func inspectorLines(
        for entry: Entry,
        movingAverage: TrendEntry?,
        windowAverage: Double?
    ) -> [String] {
        var lines = [Self.currencyLabel(entry.costUSD)]
        if let movingAverage {
            lines.append("7d avg \(Self.currencyLabel(movingAverage.costUSD))")
        }
        if let windowAverage {
            lines.append(Self.currencyDeltaLabel(entry.costUSD - windowAverage, suffix: "vs window avg"))
        }
        return lines
    }

    nonisolated private static func currencyLabel(_ usd: Double) -> String {
        String(format: "$%.2f", usd)
    }

    nonisolated private static func currencyDeltaLabel(_ delta: Double, suffix: String) -> String {
        if abs(delta) < 0.005 {
            return "Flat \(suffix)"
        }
        return "\(delta >= 0 ? "+" : "-")\(Self.currencyLabel(abs(delta))) \(suffix)"
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

#Preview("Daily cost — 30 days") {
    let formatter = DateFormatter()
    formatter.dateFormat = "yyyy-MM-dd"
    formatter.timeZone = TimeZone(secondsFromGMT: 0)
    formatter.locale = Locale(identifier: "en_US_POSIX")
    let base = Date()
    let calendar = Calendar.current
    let points: [CostHistoryPoint] = (0..<30).reversed().map { offset in
        let date = calendar.date(byAdding: .day, value: -offset, to: base) ?? base
        let amp = Double(30 - offset) / 30.0
        let cost = 2.0 + amp * 18.0 + Double(offset % 4) * 1.5
        return CostHistoryPoint(day: formatter.string(from: date), totalTokens: 0, costUSD: cost)
    }
    return DailyCostChart(daily: points)
        .padding()
        .frame(width: 360)
}
