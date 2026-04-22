import Charts
import HeimdallDomain
import SwiftUI

/// 24-hour bar chart showing turn activity aggregated over 30 days.
/// One bar per hour of day; x-axis ticks every 6 hours.
struct HourlyActivityChart: View {
    let buckets: [ProviderHourlyBucket]
    @State private var selectedHour: Int?

    var body: some View {
        let nonEmpty = !self.buckets.isEmpty && self.buckets.contains { $0.turns > 0 }
        VStack(alignment: .leading, spacing: 6) {
            ChartHeader(
                title: "Activity by hour · 30 days",
                caption: "Turns per hour of day (local time)."
            )
            if nonEmpty {
                self.chart
                    .frame(height: 48)
            } else {
                Text("No hourly data yet.")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .padding(.vertical, 12)
            }
        }
        .padding(8)
        .menuCardBackground(
            opacity: ChartStyle.cardBackgroundOpacity,
            cornerRadius: ChartStyle.cardCornerRadius
        )
        .accessibilityElement(children: .combine)
        .accessibilityLabel("Activity by hour, 30-day aggregate")
    }

    @ViewBuilder
    private var chart: some View {
        let selectedBucket = self.selectedHour.flatMap { Self.nearestBucket(to: $0, in: self.buckets) }
        Chart {
            ForEach(self.buckets) { b in
                BarMark(
                    x: .value("Hour", b.hour),
                    y: .value("Turns", b.turns)
                )
                .foregroundStyle(selectedBucket?.hour == b.hour ? Color.accentColor : ChartStyle.barFill)
                .cornerRadius(ChartStyle.barCornerRadius)
            }
            if let selectedBucket {
                RuleMark(x: .value("Hour", selectedBucket.hour))
                    .foregroundStyle(Color.primary.opacity(0.3))
                    .lineStyle(StrokeStyle(lineWidth: 1))
                    .annotation(
                        position: ChartStyle.inspectorPlacement(index: selectedBucket.hour, totalCount: self.buckets.count).annotationPosition,
                        spacing: 6,
                        overflowResolution: .init(x: .fit(to: .chart), y: .fit(to: .chart))
                    ) {
                        ChartInspectorCard(
                            title: Self.hourLabel(selectedBucket.hour),
                            lines: [
                                "\(selectedBucket.turns) turns",
                                "\(Self.compactTokenCount(selectedBucket.tokens)) tokens",
                                Self.currencyLabel(selectedBucket.costUSD),
                            ]
                        )
                    }
            }
        }
        .chartYAxis(.hidden)
        .chartXScale(domain: 0...23)
        .chartXAxis {
            AxisMarks(values: .stride(by: 6)) { value in
                AxisValueLabel {
                    if let h = value.as(Int.self) {
                        Text("\(h):00")
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
                                let rawHour = proxy.value(atX: x, as: Int.self)
                            else {
                                ChartStyle.updateHoverSelection(&self.selectedHour, to: nil)
                                return
                            }
                            let hour = min(max(rawHour, 0), 23)
                            guard
                                let snappedX = proxy.position(forX: hour),
                                abs(snappedX - x) <= ChartStyle.snapThreshold(
                                    plotWidth: proxy.plotSize.width,
                                    itemCount: self.buckets.count
                                )
                            else {
                                ChartStyle.updateHoverSelection(&self.selectedHour, to: nil)
                                return
                            }
                            ChartStyle.updateHoverSelection(&self.selectedHour, to: hour)
                        case .ended:
                            ChartStyle.updateHoverSelection(&self.selectedHour, to: nil)
                        }
                    }
            }
        }
        .help(Self.tooltip(for: self.buckets))
        .animation(ChartStyle.animation, value: self.buckets.map(\.turns))
        .animation(ChartStyle.hoverAnimation, value: self.selectedHour)
    }

    nonisolated static func tooltip(for buckets: [ProviderHourlyBucket]) -> String {
        buckets.map { bucket in
            "\(Self.hourLabel(bucket.hour)): \(bucket.turns) turns · \(Self.compactTokenCount(bucket.tokens)) tokens · \(Self.currencyLabel(bucket.costUSD))"
        }
        .joined(separator: "\n")
    }

    nonisolated static func nearestBucket(to hour: Int, in buckets: [ProviderHourlyBucket]) -> ProviderHourlyBucket? {
        buckets.min { lhs, rhs in
            abs(lhs.hour - hour) < abs(rhs.hour - hour)
        }
    }

    nonisolated private static func hourLabel(_ hour: Int) -> String {
        String(format: "%02d:00", hour)
    }

    nonisolated private static func compactTokenCount(_ count: Int) -> String {
        let value = Double(count)
        if value >= 1_000_000 {
            return String(format: "%.1fM", value / 1_000_000)
        }
        if value >= 1_000 {
            return String(format: "%.1fK", value / 1_000)
        }
        return "\(count)"
    }

    nonisolated private static func currencyLabel(_ usd: Double) -> String {
        String(format: "$%.2f", usd)
    }
}

// MARK: - Preview

#Preview("Hourly activity — peak afternoon") {
    let buckets: [ProviderHourlyBucket] = (0..<24).map { hour in
        let turns: Int
        switch hour {
        case 0..<6:   turns = Int.random(in: 0...3)
        case 6..<9:   turns = Int.random(in: 5...15)
        case 9..<12:  turns = Int.random(in: 20...45)
        case 12..<14: turns = Int.random(in: 10...25)
        case 14..<18: turns = Int.random(in: 35...80)
        case 18..<21: turns = Int.random(in: 15...35)
        default:       turns = Int.random(in: 2...8)
        }
        return ProviderHourlyBucket(hour: hour, turns: turns, costUSD: Double(turns) * 0.012, tokens: turns * 800)
    }
    HourlyActivityChart(buckets: buckets)
        .padding()
        .frame(width: 320)
}
