import Charts
import HeimdallDomain
import SwiftUI

struct WindowTodayView: View {
    @Bindable var model: TodayFeatureModel

    var body: some View {
        VStack(alignment: .leading, spacing: 24) {
            WindowHeader(
                title: "Today",
                subtitle: self.model.response.map { "Usage for \($0.day)" } ?? "Loading…",
                issue: WindowHeaderIssuePresentation.make(message: self.model.errorMessage),
                onRetry: { Task { await self.model.load() } },
                isRetrying: self.model.isLoading
            ) {
                TodayDateNav(model: self.model)
            }

            if let response = self.model.response {
                TodayKpisRow(totals: response.totals)

                TodayHourTimeline(hours: response.hours)

                TodayDaysHoursSection(
                    cells7: response.daysHours7,
                    cells30: response.daysHours30,
                    anchorDay: response.day
                )

                TodayWeekdayPatternSection(cells: response.weekdayHour90)
            } else if !self.model.isLoading {
                ContentUnavailableView(
                    "No data",
                    systemImage: "sun.max",
                    description: Text("No usage recorded for this day.")
                )
                .frame(maxWidth: .infinity)
                .padding(.vertical, 48)
            }
        }
        .task { await self.model.load() }
    }
}

// MARK: - Date navigation

private struct TodayDateNav: View {
    @Bindable var model: TodayFeatureModel

    private var canGoForward: Bool {
        guard let day = self.model.pinnedDate else { return false }
        let today = Self.todayString()
        return day < today
    }

    var body: some View {
        HStack(spacing: 6) {
            Button {
                self.model.selectDate(self.adjacentDay(offset: -1))
            } label: {
                Image(systemName: "chevron.left")
            }
            .buttonStyle(SecondaryDashboardButtonStyle())
            .help("Previous day")

            Button("Today") { self.model.pinToday() }
                .buttonStyle(SecondaryDashboardButtonStyle())
                .disabled(self.model.pinnedDate == nil)

            Button {
                if self.canGoForward {
                    self.model.selectDate(self.adjacentDay(offset: 1))
                }
            } label: {
                Image(systemName: "chevron.right")
            }
            .buttonStyle(SecondaryDashboardButtonStyle())
            .disabled(!self.canGoForward)
            .help("Next day")
        }
    }

    private func adjacentDay(offset: Int) -> String {
        let base = self.model.pinnedDate ?? Self.todayString()
        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy-MM-dd"
        guard let date = formatter.date(from: base) else { return base }
        let adjusted = Calendar.current.date(byAdding: .day, value: offset, to: date) ?? date
        return formatter.string(from: adjusted)
    }

    private static func todayString() -> String {
        let f = DateFormatter()
        f.dateFormat = "yyyy-MM-dd"
        return f.string(from: Date())
    }
}

// MARK: - KPI row

private struct TodayKpisRow: View {
    let totals: TodayTotals

    private var peakHourLabel: String {
        guard let h = self.totals.peakHour else { return "—" }
        let suffix = h < 12 ? "AM" : "PM"
        let display = h == 0 ? 12 : (h > 12 ? h - 12 : h)
        return "\(display)\(suffix)"
    }

    var body: some View {
        LazyVGrid(
            columns: [
                GridItem(.flexible()), GridItem(.flexible()),
                GridItem(.flexible()), GridItem(.flexible())
            ],
            spacing: 12
        ) {
            WindowOverviewKpiTile(label: "Turns", value: "\(self.totals.turns)")
            WindowOverviewKpiTile(label: "Tokens", value: Self.compactInt(self.totals.totalTokens))
            WindowOverviewKpiTile(label: "Cost", value: FormatHelpers.formatUSD(self.totals.costUSD))
            WindowOverviewKpiTile(label: "Peak hour", value: self.peakHourLabel)
        }
    }

    private static func compactInt(_ value: Int) -> String {
        let d = Double(value)
        if value >= 1_000_000_000 { return String(format: "%.1fB", d / 1_000_000_000) }
        if value >= 1_000_000     { return String(format: "%.1fM", d / 1_000_000) }
        if value >= 1_000         { return String(format: "%.1fK", d / 1_000) }
        return "\(value)"
    }
}

// MARK: - Hour timeline

private struct TodayHourTimeline: View {
    let hours: [TodayHourRow]

    private var peakCost: Double {
        self.hours.map { $0.costUSD }.max() ?? 0
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            WindowSectionHeader(
                title: "Hour by hour",
                subtitle: "Cost distribution across 24 hours"
            )

            Chart(self.hours, id: \.hour) { row in
                BarMark(
                    x: .value("Hour", row.hour),
                    y: .value("Cost", row.costUSD)
                )
                .foregroundStyle(Color.primary.opacity(self.barOpacity(for: row.costUSD)))
                .cornerRadius(2)
            }
            .chartXAxis {
                AxisMarks(values: [0, 3, 6, 9, 12, 15, 18, 21]) { value in
                    AxisValueLabel {
                        if let h = value.as(Int.self) {
                            Text(Self.hourLabel(h))
                                .font(.caption2)
                        }
                    }
                    AxisTick()
                }
            }
            .chartYAxis {
                AxisMarks(position: .leading) { value in
                    AxisValueLabel {
                        if let v = value.as(Double.self) {
                            Text(FormatHelpers.formatUSD(v))
                                .font(.caption2)
                        }
                    }
                    AxisGridLine()
                }
            }
            .chartYScale(domain: 0...(max(self.peakCost * 1.1, 0.01)))
            .frame(height: 140)
            .padding(18)
            .menuCardBackground(opacity: 0.04, cornerRadius: 16)
        }
    }

    private func barOpacity(for cost: Double) -> Double {
        guard self.peakCost > 0 else { return 0.08 }
        let fraction = cost / self.peakCost
        return max(0.08, fraction * 0.85)
    }

    private static func hourLabel(_ h: Int) -> String {
        if h == 0  { return "12AM" }
        if h == 12 { return "12PM" }
        return h < 12 ? "\(h)AM" : "\(h - 12)PM"
    }
}

// MARK: - Days × Hours heatmaps

private struct TodayDaysHoursSection: View {
    let cells7: [DayHourCell]
    let cells30: [DayHourCell]
    let anchorDay: String

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            WindowSectionHeader(
                title: "Days × hours",
                subtitle: "Activity density by day and hour"
            )

            VStack(alignment: .leading, spacing: 12) {
                if !self.cells7.isEmpty {
                    TodayDaysHoursGrid(cells: self.cells7, label: "Last 7 days")
                }
                if !self.cells30.isEmpty {
                    TodayDaysHoursGrid(cells: self.cells30, label: "Last 30 days")
                }
            }
            .padding(18)
            .menuCardBackground(opacity: 0.04, cornerRadius: 16)
        }
    }
}

private struct TodayDaysHoursGrid: View {
    let cells: [DayHourCell]
    let label: String

    private var days: [String] {
        Array(Set(self.cells.map { $0.day })).sorted()
    }

    private var cellsByDayHour: [String: [Int: Int]] {
        var result: [String: [Int: Int]] = [:]
        for cell in self.cells {
            result[cell.day, default: [:]][cell.hour] = cell.turns
        }
        return result
    }

    private var maxTurns: Int {
        self.cells.map { $0.turns }.max() ?? 1
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(self.label)
                .font(.caption.weight(.semibold))
                .foregroundStyle(.secondary)

            GeometryReader { geo in
                let dayWidth = max(2, (geo.size.width - CGFloat(max(1, self.days.count) - 1)) / CGFloat(max(1, self.days.count)))

                VStack(spacing: 1) {
                    ForEach(0..<24, id: \.self) { hour in
                        HStack(spacing: 1) {
                            ForEach(self.days, id: \.self) { day in
                                let turns = self.cellsByDayHour[day]?[hour] ?? 0
                                let opacity = turns == 0 ? 0.04 : (0.15 + 0.70 * Double(turns) / Double(max(1, self.maxTurns)))
                                Rectangle()
                                    .fill(Color.primary.opacity(opacity))
                                    .frame(width: dayWidth, height: 5)
                            }
                        }
                    }
                }
            }
            .frame(height: 24 * 5 + 23)
        }
    }
}

// MARK: - Weekday × Hour pattern

private struct TodayWeekdayPatternSection: View {
    let cells: [WeekdayHourCell]

    private var heatmapCells: [ProviderHeatmapCell] {
        self.cells.map { ProviderHeatmapCell(dayOfWeek: $0.dow, hour: $0.hour, turns: $0.turns) }
    }

    var body: some View {
        if !self.cells.isEmpty {
            VStack(alignment: .leading, spacing: 14) {
                WindowSectionHeader(
                    title: "Weekly pattern",
                    subtitle: "Activity by day of week and hour over 90 days"
                )

                ActivityHeatmap(cells: self.heatmapCells)
                    .padding(18)
                    .menuCardBackground(opacity: 0.04, cornerRadius: 16)
            }
        }
    }
}
