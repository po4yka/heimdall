import Charts
import HeimdallDomain
import SwiftUI

/// 7 × 24 heatmap of turn activity. Rows = days of week (Sun–Sat),
/// columns = hours 0–23. Intensity is opacity of `Color.primary`.
struct ActivityHeatmap: View {
    let cells: [ProviderHeatmapCell]

    nonisolated private static let dayLabels = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]
    private static let hourTicks = [0, 6, 12, 18, 23]
    private static let dayLabelWidth: CGFloat = 24
    private static let dayLabelGap: CGFloat = 8
    private static let cellSpacing: CGFloat = 2
    private static let tooltipWidth: CGFloat = 150
    private static let tooltipHeight: CGFloat = 64
    nonisolated private static let dimMultiplier: Double = 0.55

    @State private var hoveredCell: HoveredCell?
    @State private var cellFrames: [CellKey: CGRect] = [:]
    @State private var gridWidth: CGFloat = 0

    struct Summary: Equatable {
        let totalTurns: Int
        let activeCells: Int
        let peakTurns: Int
        let peakDay: Int
        let peakHour: Int
    }

    struct IntensityScale: Equatable {
        struct Level: Equatable, Identifiable {
            let threshold: Int
            let opacity: Double

            var id: Int { self.threshold * 100 + Int(self.opacity * 100) }
        }

        let levels: [Level]

        func opacity(for turns: Int) -> Double {
            guard turns > 0 else { return 0.04 }
            return self.levels.last(where: { turns >= $0.threshold })?.opacity ?? 0.14
        }
    }

    fileprivate struct CellKey: Hashable {
        let day: Int
        let hour: Int
    }

    private struct HoveredCell: Equatable {
        let day: Int
        let hour: Int
        let turns: Int
    }

    private struct CellFramePreferenceKey: PreferenceKey {
        static let defaultValue: [CellKey: CGRect] = [:]
        static func reduce(value: inout [CellKey: CGRect], nextValue: () -> [CellKey: CGRect]) {
            value.merge(nextValue(), uniquingKeysWith: { _, new in new })
        }
    }

    private struct GridWidthPreferenceKey: PreferenceKey {
        static let defaultValue: CGFloat = 0
        static func reduce(value: inout CGFloat, nextValue: () -> CGFloat) {
            value = max(value, nextValue())
        }
    }

    var body: some View {
        let grid = Self.lookup(self.cells)
        let maxTurns = grid.flatMap { $0 }.max() ?? 0
        let summary = Self.summary(from: grid)
        let scale = Self.intensityScale(for: grid)
        let rank = Self.rankLookup(grid: grid)
        VStack(alignment: .leading, spacing: 6) {
            ChartHeader(
                title: "Activity heatmap · 30 days",
                caption: "Stepped opacity scale keeps quieter hours visible.",
                trailing: summary.map { summary in
                    AnyView(
                        Text("\(summary.totalTurns) turns")
                            .font(.system(size: 10, weight: .semibold).monospacedDigit())
                            .foregroundStyle(Color.primary.opacity(0.62))
                    )
                }
            )
            if maxTurns == 0 {
                Text("No heatmap data yet.")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .padding(.vertical, 12)
            } else {
                self.heatmapGrid(grid: grid, summary: summary, scale: scale, rank: rank)
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
    private func heatmapGrid(
        grid: [[Int]],
        summary: Summary?,
        scale: IntensityScale,
        rank: [CellKey: Int]
    ) -> some View {
        let activeCells = summary?.activeCells ?? 0
        let totalTurns = summary?.totalTurns ?? 0
        VStack(alignment: .leading, spacing: 2) {
            if let summary {
                self.summaryRow(summary)
                    .padding(.bottom, 4)
            }
            self.legendRow(scale)
                .padding(.bottom, 5)
            self.gridStack(grid: grid, summary: summary, scale: scale)
                .background(
                    GeometryReader { geo in
                        Color.clear.preference(
                            key: GridWidthPreferenceKey.self,
                            value: geo.size.width
                        )
                    }
                )
                .coordinateSpace(name: "heatmap")
                .overlay(alignment: .topLeading) {
                    self.tooltipOverlay(totalTurns: totalTurns, activeCells: activeCells, rank: rank)
                }
                .onPreferenceChange(CellFramePreferenceKey.self) { newValue in
                    self.cellFrames = newValue
                }
                .onPreferenceChange(GridWidthPreferenceKey.self) { newValue in
                    self.gridWidth = newValue
                }
        }
    }

    @ViewBuilder
    private func gridStack(grid: [[Int]], summary: Summary?, scale: IntensityScale) -> some View {
        VStack(spacing: Self.cellSpacing) {
            ForEach(0..<7, id: \.self) { day in
                HStack(alignment: .center, spacing: Self.dayLabelGap) {
                    Text(Self.dayLabels[day])
                        .font(
                            .system(
                                size: 8,
                                weight: self.hoveredCell?.day == day ? .semibold : .medium
                            )
                            .monospacedDigit()
                        )
                        .foregroundStyle(self.hoveredCell?.day == day ? Color.primary : Color.secondary)
                        .frame(width: Self.dayLabelWidth, alignment: .leading)
                    HStack(spacing: Self.cellSpacing) {
                        ForEach(0..<24, id: \.self) { hour in
                            self.cellView(
                                day: day,
                                hour: hour,
                                grid: grid,
                                summary: summary,
                                scale: scale
                            )
                        }
                    }
                    .frame(maxWidth: .infinity)
                }
            }
            HStack(alignment: .center, spacing: Self.dayLabelGap) {
                Color.clear.frame(width: Self.dayLabelWidth, height: 8)
                HStack(spacing: Self.cellSpacing) {
                    ForEach(0..<24, id: \.self) { hour in
                        Text(Self.hourTicks.contains(hour) ? Self.hourLabel(hour) : "")
                            .font(
                                .system(
                                    size: 8,
                                    weight: self.hoveredCell?.hour == hour ? .semibold : .regular
                                )
                                .monospacedDigit()
                            )
                            .foregroundStyle(self.hoveredCell?.hour == hour ? Color.primary : Color.secondary)
                            .frame(maxWidth: .infinity, alignment: Self.tickAlignment(for: hour))
                    }
                }
                .frame(maxWidth: .infinity)
            }
            .padding(.top, 4)
        }
    }

    @ViewBuilder
    private func cellView(
        day: Int,
        hour: Int,
        grid: [[Int]],
        summary: Summary?,
        scale: IntensityScale
    ) -> some View {
        let turns = grid[day][hour]
        let isPeak = summary?.peakDay == day && summary?.peakHour == hour
        let hoverActive = self.hoveredCell != nil
        let isThisHovered = self.hoveredCell?.day == day && self.hoveredCell?.hour == hour
        let dim = hoverActive && !isThisHovered
        let opacity = Self.cellOpacity(turns: turns, scale: scale, dim: dim)
        let key = CellKey(day: day, hour: hour)
        RoundedRectangle(cornerRadius: 2, style: .continuous)
            .fill(Color.primary.opacity(opacity))
            .overlay {
                if isPeak {
                    RoundedRectangle(cornerRadius: 2, style: .continuous)
                        .stroke(Color.accentColor.opacity(0.85), lineWidth: 1)
                }
            }
            .aspectRatio(1, contentMode: .fit)
            .frame(maxWidth: .infinity)
            .background(
                GeometryReader { geometry in
                    Color.clear.preference(
                        key: CellFramePreferenceKey.self,
                        value: [key: geometry.frame(in: .named("heatmap"))]
                    )
                }
            )
            .contentShape(Rectangle())
            .onHover { hovering in
                if hovering {
                    let next = HoveredCell(day: day, hour: hour, turns: turns)
                    withAnimation(ChartStyle.hoverAnimation) {
                        self.hoveredCell = next
                    }
                } else if self.hoveredCell?.day == day && self.hoveredCell?.hour == hour {
                    withAnimation(ChartStyle.hoverAnimation) {
                        self.hoveredCell = nil
                    }
                }
            }
            .accessibilityLabel("\(Self.dayLabels[day]) \(Self.hourLabel(hour))")
            .accessibilityValue("\(turns) turns")
            .accessibilityAddTraits(.isButton)
    }

    @ViewBuilder
    private func tooltipOverlay(
        totalTurns: Int,
        activeCells: Int,
        rank: [CellKey: Int]
    ) -> some View {
        if let hovered = self.hoveredCell,
           let frame = self.cellFrames[CellKey(day: hovered.day, hour: hovered.hour)] {
            let above = hovered.day >= 2
            let layoutWidth = max(self.gridWidth, Self.tooltipWidth)
            let preferredX = frame.midX - Self.tooltipWidth / 2
            let clampedX = max(0, min(layoutWidth - Self.tooltipWidth, preferredX))
            let y = above
                ? frame.minY - Self.tooltipHeight - 6
                : frame.maxY + 6
            self.tooltipCard(
                for: hovered,
                totalTurns: totalTurns,
                activeCells: activeCells,
                rank: rank
            )
            .frame(width: Self.tooltipWidth, alignment: .leading)
            .offset(x: clampedX, y: y)
            .allowsHitTesting(false)
            .transition(.opacity.combined(with: .scale(scale: 0.96)))
            .zIndex(1)
        }
    }

    private func tooltipCard(
        for hovered: HoveredCell,
        totalTurns: Int,
        activeCells: Int,
        rank: [CellKey: Int]
    ) -> some View {
        let key = CellKey(day: hovered.day, hour: hovered.hour)
        let percent = totalTurns > 0
            ? Double(hovered.turns) / Double(totalTurns) * 100
            : 0
        var lines: [String] = []
        if hovered.turns > 0 {
            lines.append("\(hovered.turns) turns")
            lines.append(String(format: "%.1f%% of total", percent))
            if let rankIndex = rank[key], activeCells > 0 {
                lines.append("Cell #\(rankIndex) of \(activeCells) active")
            }
        } else {
            lines.append("No activity")
        }
        let title = "\(Self.dayLabels[hovered.day]) \(Self.hourLabel(hovered.hour)):00"
        return ChartInspectorCard(title: title, lines: lines)
    }

    private func legendRow(_ scale: IntensityScale) -> some View {
        HStack(spacing: 8) {
            Text("Scale")
                .font(.system(size: 8, weight: .bold))
                .tracking(0.5)
                .foregroundStyle(Color.primary.opacity(0.48))
            ForEach(scale.levels) { level in
                HStack(spacing: 4) {
                    RoundedRectangle(cornerRadius: 2, style: .continuous)
                        .fill(Color.primary.opacity(level.opacity))
                        .frame(width: 8, height: 8)
                    Text("\(Self.turnLabel(level.threshold))+")
                        .font(.system(size: 8).monospacedDigit())
                        .foregroundStyle(.secondary)
                }
            }
            Spacer(minLength: 0)
        }
    }

    private func summaryRow(_ summary: Summary) -> some View {
        HStack(spacing: 6) {
            self.summaryMetric(
                label: "Peak",
                value: "\(Self.dayLabels[summary.peakDay]) \(Self.hourLabel(summary.peakHour))",
                detail: "\(summary.peakTurns) turns"
            )
            self.summaryMetric(
                label: "Active",
                value: "\(summary.activeCells) cells",
                detail: "with activity"
            )
            self.summaryMetric(
                label: "Cadence",
                value: summary.activeCells > 0
                    ? String(format: "%.1f", Double(summary.totalTurns) / Double(summary.activeCells))
                    : "0.0",
                detail: "avg turns/cell"
            )
        }
    }

    private func summaryMetric(label: String, value: String, detail: String) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(label.uppercased())
                .font(.system(size: 8, weight: .bold))
                .tracking(0.5)
                .foregroundStyle(Color.primary.opacity(0.48))
            Text(value)
                .font(.system(size: 11, weight: .semibold).monospacedDigit())
                .foregroundStyle(.primary)
                .lineLimit(1)
                .minimumScaleFactor(0.8)
            Text(detail)
                .font(.system(size: 8))
                .foregroundStyle(Color.primary.opacity(0.55))
                .lineLimit(1)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, 8)
        .padding(.vertical, 7)
        .background(
            RoundedRectangle(cornerRadius: 7, style: .continuous)
                .fill(Color.primary.opacity(0.04))
        )
    }

    nonisolated static func summary(from grid: [[Int]]) -> Summary? {
        guard !grid.isEmpty else { return nil }

        var totalTurns = 0
        var activeCells = 0
        var peakTurns = 0
        var peakDay = 0
        var peakHour = 0

        for day in 0..<min(7, grid.count) {
            for hour in 0..<min(24, grid[day].count) {
                let turns = grid[day][hour]
                totalTurns += turns
                if turns > 0 {
                    activeCells += 1
                }
                if turns > peakTurns {
                    peakTurns = turns
                    peakDay = day
                    peakHour = hour
                }
            }
        }

        guard totalTurns > 0 else { return nil }
        return Summary(
            totalTurns: totalTurns,
            activeCells: activeCells,
            peakTurns: peakTurns,
            peakDay: peakDay,
            peakHour: peakHour
        )
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

    nonisolated static func intensityScale(for grid: [[Int]]) -> IntensityScale {
        let active = grid
            .flatMap { $0 }
            .filter { $0 > 0 }
            .sorted()

        guard !active.isEmpty else { return IntensityScale(levels: []) }

        let candidates = [
            1,
            Self.quantile(active, fraction: 0.35),
            Self.quantile(active, fraction: 0.6),
            Self.quantile(active, fraction: 0.85),
            active.last ?? 1,
        ]

        let thresholds = candidates.reduce(into: [Int]()) { result, value in
            let clamped = max(1, value)
            if result.last != clamped {
                result.append(clamped)
            }
        }

        let opacityRamp: [Double] = [0.14, 0.24, 0.38, 0.58, 0.82]
        let levels = thresholds.enumerated().map { index, threshold in
            IntensityScale.Level(
                threshold: threshold,
                opacity: opacityRamp[min(index, opacityRamp.count - 1)]
            )
        }
        return IntensityScale(levels: levels)
    }

    nonisolated fileprivate static func rankLookup(grid: [[Int]]) -> [CellKey: Int] {
        var entries: [(key: CellKey, turns: Int)] = []
        for day in 0..<min(7, grid.count) {
            for hour in 0..<min(24, grid[day].count) {
                let turns = grid[day][hour]
                if turns > 0 {
                    entries.append((CellKey(day: day, hour: hour), turns))
                }
            }
        }
        entries.sort { lhs, rhs in
            if lhs.turns != rhs.turns { return lhs.turns > rhs.turns }
            if lhs.key.day != rhs.key.day { return lhs.key.day < rhs.key.day }
            return lhs.key.hour < rhs.key.hour
        }
        var lookup: [CellKey: Int] = [:]
        for (index, entry) in entries.enumerated() {
            lookup[entry.key] = index + 1
        }
        return lookup
    }

    nonisolated private static func cellOpacity(turns: Int, scale: IntensityScale, dim: Bool) -> Double {
        let base = scale.opacity(for: turns)
        return dim ? base * Self.dimMultiplier : base
    }

    nonisolated private static func hourLabel(_ hour: Int) -> String {
        String(format: "%02d", hour)
    }

    nonisolated private static func turnLabel(_ turns: Int) -> String {
        if turns >= 1_000 {
            return String(format: "%.1fK", Double(turns) / 1_000)
        }
        return "\(turns)"
    }

    private static func tickAlignment(for hour: Int) -> Alignment {
        if hour == 0 {
            return .leading
        }
        if hour == 23 {
            return .trailing
        }
        return .center
    }

    nonisolated private static func quantile(_ values: [Int], fraction: Double) -> Int {
        guard !values.isEmpty else { return 0 }
        let clamped = max(0, min(1, fraction))
        let index = Int((Double(values.count - 1) * clamped).rounded())
        return values[index]
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
