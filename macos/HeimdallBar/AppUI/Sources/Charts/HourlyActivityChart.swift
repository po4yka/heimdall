import Charts
import HeimdallDomain
import SwiftUI

/// 24-hour bar chart showing turn activity aggregated over 30 days.
/// One bar per hour of day; x-axis ticks every 6 hours.
struct HourlyActivityChart: View {
    let buckets: [ProviderHourlyBucket]

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
        Chart {
            ForEach(self.buckets) { b in
                BarMark(
                    x: .value("Hour", b.hour),
                    y: .value("Turns", b.turns)
                )
                .foregroundStyle(ChartStyle.barFill)
                .cornerRadius(ChartStyle.barCornerRadius)
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
        .animation(ChartStyle.animation, value: self.buckets.map(\.turns))
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
