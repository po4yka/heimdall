import Charts
import HeimdallDomain
import SwiftUI

/// 7 × 24 heatmap of turn activity. Rows = days of week (Sun–Sat),
/// columns = hours 0–23. Intensity is opacity of `Color.primary`.
struct ActivityHeatmap: View {
    let cells: [ProviderHeatmapCell]

    private static let dayLabels = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]
    private static let cellSize: CGFloat = 9
    private static let cellSpacing: CGFloat = 2

    var body: some View {
        let grid = Self.lookup(self.cells)
        let maxTurns = grid.flatMap { $0 }.max() ?? 0
        VStack(alignment: .leading, spacing: 6) {
            ChartHeader(
                title: "Activity heatmap · 30 days",
                caption: "Brighter cells = more turns."
            )
            if maxTurns == 0 {
                Text("No heatmap data yet.")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .padding(.vertical, 12)
            } else {
                self.heatmapGrid(grid: grid, maxTurns: maxTurns)
            }
        }
        .padding(8)
        .menuCardBackground(
            opacity: ChartStyle.cardBackgroundOpacity,
            cornerRadius: ChartStyle.cardCornerRadius
        )
        .accessibilityElement(children: .combine)
        .accessibilityLabel("Activity heatmap, 7 days by 24 hours")
    }

    @ViewBuilder
    private func heatmapGrid(grid: [[Int]], maxTurns: Int) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            // Day rows
            VStack(spacing: Self.cellSpacing) {
                ForEach(0..<7, id: \.self) { day in
                    HStack(spacing: 0) {
                        // Day label
                        Text(Self.dayLabels[day])
                            .font(.system(size: 8).monospacedDigit())
                            .foregroundStyle(.secondary)
                            .frame(width: 24, alignment: .trailing)
                            .padding(.trailing, 4)
                        // Hour cells
                        HStack(spacing: Self.cellSpacing) {
                            ForEach(0..<24, id: \.self) { hour in
                                let turns = grid[day][hour]
                                let intensity = turns == 0
                                    ? 0.04
                                    : max(0.04, min(1.0, Double(turns) / Double(maxTurns)))
                                RoundedRectangle(cornerRadius: 2, style: .continuous)
                                    .fill(Color.primary.opacity(intensity))
                                    .frame(width: Self.cellSize, height: Self.cellSize)
                            }
                        }
                    }
                }
            }
            // Hour axis labels
            HStack(spacing: 0) {
                Spacer().frame(width: 28) // align with cells
                HStack(spacing: 0) {
                    ForEach([0, 6, 12, 18], id: \.self) { tick in
                        // Each tick spans 6 cells
                        Text("\(tick)")
                            .font(.system(size: 8).monospacedDigit())
                            .foregroundStyle(.secondary)
                        if tick < 18 {
                            Spacer()
                        }
                    }
                }
                .frame(width: CGFloat(24) * (Self.cellSize + Self.cellSpacing) - Self.cellSpacing)
            }
        }
    }

    /// Build a 7×24 matrix (day × hour) of turn counts from sparse cells.
    nonisolated static func lookup(_ cells: [ProviderHeatmapCell]) -> [[Int]] {
        var grid = Array(repeating: Array(repeating: 0, count: 24), count: 7)
        for cell in cells {
            let day = max(0, min(6, cell.dayOfWeek))
            let hour = max(0, min(23, cell.hour))
            grid[day][hour] += cell.turns
        }
        return grid
    }
}

// MARK: - Preview

#Preview("Activity heatmap — weekday mornings/afternoons") {
    let sample: [ProviderHeatmapCell] = {
        var result: [ProviderHeatmapCell] = []
        // Weekdays (Mon=1 .. Fri=5): morning cluster 9–11, afternoon 14–17
        for day in 1...5 {
            for hour in 9...11 {
                result.append(ProviderHeatmapCell(dayOfWeek: day, hour: hour, turns: Int.random(in: 8...30)))
            }
            for hour in 14...17 {
                result.append(ProviderHeatmapCell(dayOfWeek: day, hour: hour, turns: Int.random(in: 15...50)))
            }
            // Light evening
            for hour in 19...21 {
                result.append(ProviderHeatmapCell(dayOfWeek: day, hour: hour, turns: Int.random(in: 2...10)))
            }
        }
        // Light weekend activity
        result.append(ProviderHeatmapCell(dayOfWeek: 0, hour: 10, turns: 5))
        result.append(ProviderHeatmapCell(dayOfWeek: 6, hour: 11, turns: 8))
        return result
    }()
    ActivityHeatmap(cells: sample)
        .padding()
        .frame(width: 320)
}
