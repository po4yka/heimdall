import Charts
import HeimdallDomain
import SwiftUI

/// Stacked-area chart showing per-provider daily cost over the trailing 30
/// days, with a total-spend outline and compact provider share cards.
struct ProviderComparisonChart: View {
    let items: [ProviderMenuProjection]
    @State private var selectedDay: Date?

    struct Entry: Identifiable, Hashable {
        let day: Date
        let providerTitle: String
        let costUSD: Double

        var id: String { "\(self.day.timeIntervalSince1970)-\(self.providerTitle)" }
    }

    struct TotalEntry: Identifiable, Hashable {
        let day: Date
        let costUSD: Double

        var id: Date { self.day }
    }

    struct ProviderSummary: Identifiable, Hashable {
        let title: String
        let costUSD: Double
        let share: Double

        var id: String { self.title }
    }

    private let columns = [
        GridItem(.flexible(minimum: 120), spacing: 8),
        GridItem(.flexible(minimum: 120), spacing: 8),
    ]

    var body: some View {
        let summaries = Self.providerSummaries(from: self.items)
        let entries = Self.entries(from: self.items)
        let totals = Self.totalEntries(from: entries)
        let providerTitles = summaries.map(\.title)
        let totalCostUSD = summaries.reduce(0.0) { $0 + $1.costUSD }
        let hasData = summaries.count >= 2 && !entries.isEmpty && !totals.isEmpty

        VStack(alignment: .leading, spacing: 8) {
            ChartHeader(
                title: "Provider split, 30 days",
                caption: "Stacked daily cost by provider with total-spend contour."
            )

            if hasData {
                ProviderComparisonSummaryStrip(
                    totalCostUSD: totalCostUSD,
                    averageDailyCostUSD: Self.averageDailyCost(totalCostUSD: totalCostUSD, activeDays: totals.count),
                    leadProvider: summaries.first
                )

                self.chart(entries: entries, totals: totals, providerTitles: providerTitles)
                    .frame(height: 104)
                    .help(Self.tooltip(entries: entries, totals: totals))

                LazyVGrid(columns: self.columns, spacing: 8) {
                    ForEach(Array(summaries.enumerated()), id: \.element.id) { index, summary in
                        ProviderSplitStatCard(
                            summary: summary,
                            tint: Self.providerScale(count: providerTitles.count)[index]
                        )
                    }
                }
            } else {
                Text("No provider activity yet.")
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
        .accessibilityLabel("Provider cost split, last 30 days")
    }

    private func chart(entries: [Entry], totals: [TotalEntry], providerTitles: [String]) -> some View {
        let selectedTotal = self.selectedDay.flatMap { Self.nearestTotalEntry(to: $0, in: totals) }
        let selectedIndex = selectedTotal.flatMap { totals.firstIndex(of: $0) }
        return Chart {
            ForEach(entries) { entry in
                AreaMark(
                    x: .value("Day", entry.day),
                    y: .value("Cost", entry.costUSD),
                    stacking: .standard
                )
                .foregroundStyle(by: .value("Provider", entry.providerTitle))
                .interpolationMethod(.monotone)
            }

            ForEach(totals) { entry in
                LineMark(
                    x: .value("Day", entry.day),
                    y: .value("Total cost", entry.costUSD)
                )
                .foregroundStyle(Color.primary.opacity(0.82))
                .lineStyle(StrokeStyle(lineWidth: 1.25, lineCap: .round, lineJoin: .round))
                .interpolationMethod(.monotone)
            }

            if let latest = totals.last {
                RuleMark(x: .value("Latest", latest.day))
                    .foregroundStyle(ChartStyle.todayRuleStroke)
                    .lineStyle(StrokeStyle(lineWidth: ChartStyle.todayRuleWidth, dash: [2, 2]))

                PointMark(
                    x: .value("Latest", latest.day),
                    y: .value("Total cost", latest.costUSD)
                )
                .foregroundStyle(Color.primary)
                .symbolSize(26)
            }
            if let selectedTotal, let selectedIndex {
                RuleMark(x: .value("Selected day", selectedTotal.day))
                    .foregroundStyle(Color.primary.opacity(0.3))
                    .lineStyle(StrokeStyle(lineWidth: 1))
                    .annotation(
                        position: ChartStyle.inspectorPlacement(index: selectedIndex, totalCount: totals.count).annotationPosition,
                        spacing: 6,
                        overflowResolution: .init(x: .fit(to: .chart), y: .fit(to: .chart))
                    ) {
                        ChartInspectorCard(
                            title: Self.axisFormatter.string(from: selectedTotal.day),
                            lines: Self.inspectorLines(for: selectedTotal, entries: entries)
                        )
                    }
            }
        }
        .chartForegroundStyleScale(
            domain: providerTitles,
            range: Self.providerScale(count: providerTitles.count)
        )
        .chartLegend(.hidden)
        .chartYAxis(.hidden)
        .chartXAxis {
            AxisMarks(values: .automatic(desiredCount: 4)) { value in
                AxisGridLine(stroke: StrokeStyle(lineWidth: 0.5))
                    .foregroundStyle(Color.primary.opacity(0.08))
                AxisValueLabel {
                    if let date = value.as(Date.self) {
                        Text(Self.axisFormatter.string(from: date))
                            .font(.system(size: 9).monospacedDigit())
                            .foregroundStyle(.secondary)
                    }
                }
            }
        }
        .chartYScale(domain: .automatic(includesZero: true))
        .chartPlotStyle { plot in
            plot
                .padding(.vertical, 4)
                .background(
                    RoundedRectangle(cornerRadius: 10, style: .continuous)
                        .fill(Color.primary.opacity(0.025))
                )
                .overlay(
                    RoundedRectangle(cornerRadius: 10, style: .continuous)
                        .stroke(Color.primary.opacity(0.08), lineWidth: 1)
                )
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
                                let nearest = Self.nearestTotalEntry(to: day, in: totals),
                                let snappedX = proxy.position(forX: nearest.day),
                                abs(snappedX - x) <= ChartStyle.snapThreshold(
                                    plotWidth: proxy.plotSize.width,
                                    itemCount: totals.count
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
        .animation(ChartStyle.animation, value: entries)
        .animation(ChartStyle.hoverAnimation, value: self.selectedDay)
    }

    // MARK: - Data transform

    nonisolated static func entries(from items: [ProviderMenuProjection]) -> [Entry] {
        var result: [Entry] = []
        for item in items {
            let parsed = item.dailyCosts.compactMap { point -> Entry? in
                guard let date = Self.dayFormatter.date(from: point.day) else { return nil }
                return Entry(day: date, providerTitle: item.title, costUSD: point.costUSD)
            }
            result.append(contentsOf: parsed)
        }
        return result.sorted { lhs, rhs in
            if lhs.day == rhs.day {
                return lhs.providerTitle < rhs.providerTitle
            }
            return lhs.day < rhs.day
        }
    }

    nonisolated static func totalEntries(from entries: [Entry]) -> [TotalEntry] {
        let grouped = Dictionary(grouping: entries, by: \.day)
        return grouped.keys.sorted().map { day in
            TotalEntry(
                day: day,
                costUSD: grouped[day, default: []].reduce(0.0) { $0 + $1.costUSD }
            )
        }
    }

    nonisolated static func providerSummaries(from items: [ProviderMenuProjection]) -> [ProviderSummary] {
        let raw: [(title: String, costUSD: Double)] = items.compactMap { item in
            let dailyFallback = item.dailyCosts.reduce(0.0) { $0 + $1.costUSD }
            let total = item.last30DaysCostUSD ?? dailyFallback
            guard total > 0 else { return nil }
            return (title: item.title, costUSD: total)
        }

        let totalCostUSD = raw.reduce(0.0) { $0 + $1.costUSD }
        guard totalCostUSD > 0 else { return [] }

        return raw
            .map {
                ProviderSummary(
                    title: $0.title,
                    costUSD: $0.costUSD,
                    share: $0.costUSD / totalCostUSD
                )
            }
            .sorted { lhs, rhs in
                if lhs.costUSD == rhs.costUSD {
                    return lhs.title < rhs.title
                }
                return lhs.costUSD > rhs.costUSD
            }
    }

    nonisolated static func averageDailyCost(totalCostUSD: Double, activeDays: Int) -> Double? {
        guard activeDays > 0, totalCostUSD > 0 else { return nil }
        return totalCostUSD / Double(activeDays)
    }

    nonisolated static func tooltip(entries: [Entry], totals: [TotalEntry]) -> String {
        let byDay = Dictionary(grouping: entries, by: \.day)
        return totals.map { total in
            let providers = byDay[total.day, default: []]
                .sorted { $0.costUSD > $1.costUSD }
                .map { "\($0.providerTitle) \(Self.currencyLabel($0.costUSD))" }
                .joined(separator: " · ")
            let prefix = "\(Self.axisFormatter.string(from: total.day)): \(Self.currencyLabel(total.costUSD))"
            return providers.isEmpty ? prefix : "\(prefix) · \(providers)"
        }
        .joined(separator: "\n")
    }

    nonisolated static func inspectorLines(for total: TotalEntry, entries: [Entry]) -> [String] {
        let contributors = entries
            .filter { $0.day == total.day }
            .sorted { $0.costUSD > $1.costUSD }
            .map { "\($0.providerTitle) \(Self.currencyLabel($0.costUSD))" }
        return [Self.currencyLabel(total.costUSD)] + contributors
    }

    nonisolated static func nearestTotalEntry(to day: Date, in totals: [TotalEntry]) -> TotalEntry? {
        totals.min { lhs, rhs in
            abs(lhs.day.timeIntervalSince(day)) < abs(rhs.day.timeIntervalSince(day))
        }
    }

    /// Monochrome opacity ladder: first provider uses `accentColor`, rest step
    /// down `Color.primary` at 0.72 / 0.45 / 0.24, cycling when count > 4.
    nonisolated static func providerScale(count: Int) -> [Color] {
        let ladder: [Color] = [
            Color.primary.opacity(0.72),
            Color.primary.opacity(0.45),
            Color.primary.opacity(0.24),
        ]
        guard count > 0 else { return [] }
        var result: [Color] = [Color.accentColor]
        for i in 1..<count {
            result.append(ladder[(i - 1) % ladder.count])
        }
        return result
    }

    nonisolated static func currencyLabel(_ value: Double) -> String {
        let formatter = NumberFormatter()
        formatter.numberStyle = .currency
        formatter.locale = Locale(identifier: "en_US")
        formatter.currencyCode = "USD"
        formatter.currencySymbol = "$"
        formatter.minimumFractionDigits = value >= 100 ? 0 : 2
        formatter.maximumFractionDigits = value >= 100 ? 0 : 2
        formatter.positiveFormat = formatter.minimumFractionDigits == 0 ? "¤#,##0" : "¤#,##0.00"
        formatter.negativeFormat = formatter.minimumFractionDigits == 0 ? "-¤#,##0" : "-¤#,##0.00"
        return formatter.string(from: NSNumber(value: value)) ?? String(format: "$%.2f", value)
    }

    nonisolated static func percentLabel(_ value: Double) -> String {
        String(format: "%.0f%%", max(0, min(1, value)) * 100)
    }

    nonisolated(unsafe) static let dayFormatter: DateFormatter = {
        let f = DateFormatter()
        f.dateFormat = "yyyy-MM-dd"
        f.timeZone = TimeZone(secondsFromGMT: 0)
        f.locale = Locale(identifier: "en_US_POSIX")
        return f
    }()

    nonisolated(unsafe) private static let axisFormatter: DateFormatter = {
        let f = DateFormatter()
        f.dateFormat = "MMM d"
        return f
    }()
}

private struct ProviderComparisonSummaryStrip: View {
    let totalCostUSD: Double
    let averageDailyCostUSD: Double?
    let leadProvider: ProviderComparisonChart.ProviderSummary?

    var body: some View {
        HStack(spacing: 8) {
            ProviderComparisonHeadlineCard(
                title: "30-day total",
                value: ProviderComparisonChart.currencyLabel(self.totalCostUSD),
                detail: self.averageDailyCostUSD.map {
                    "Avg/day \(ProviderComparisonChart.currencyLabel($0))"
                } ?? "No active days"
            )

            if let leadProvider {
                ProviderComparisonHeadlineCard(
                    title: "Leader",
                    value: leadProvider.title,
                    detail: "\(ProviderComparisonChart.percentLabel(leadProvider.share)) share · \(ProviderComparisonChart.currencyLabel(leadProvider.costUSD))"
                )
            }
        }
    }
}

private struct ProviderComparisonHeadlineCard: View {
    let title: String
    let value: String
    let detail: String

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(self.title)
                .font(.caption2.weight(.semibold))
                .foregroundStyle(.secondary)
                .textCase(.uppercase)
                .tracking(0.4)
            Text(self.value)
                .font(.callout.monospacedDigit().weight(.semibold))
            Text(self.detail)
                .font(.caption2)
                .foregroundStyle(Color.primary.opacity(0.66))
                .lineLimit(2)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, 10)
        .padding(.vertical, 8)
        .menuCardBackground(opacity: 0.03, cornerRadius: 10)
    }
}

private struct ProviderSplitStatCard: View {
    let summary: ProviderComparisonChart.ProviderSummary
    let tint: Color

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack(alignment: .center, spacing: 6) {
                Circle()
                    .fill(self.tint)
                    .frame(width: 8, height: 8)
                Text(self.summary.title)
                    .font(.caption.weight(.semibold))
                Spacer(minLength: 0)
            }

            Text(ProviderComparisonChart.percentLabel(self.summary.share))
                .font(.headline.monospacedDigit().weight(.semibold))

            Text(ProviderComparisonChart.currencyLabel(self.summary.costUSD))
                .font(.caption2)
                .foregroundStyle(Color.primary.opacity(0.66))
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, 10)
        .padding(.vertical, 8)
        .menuCardBackground(opacity: 0.03, cornerRadius: 10)
    }
}

#Preview("Provider split — 30 days") {
    let items = [
        ProviderComparisonChart.previewProjection(title: "Claude", providerID: .claude, scale: 1.0),
        ProviderComparisonChart.previewProjection(title: "Codex", providerID: .codex, scale: 0.38),
    ]
    ProviderComparisonChart(items: items)
        .padding()
        .frame(width: 360)
}

private extension ProviderComparisonChart {
    static func previewProjection(title: String, providerID: ProviderID, scale: Double) -> ProviderMenuProjection {
        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy-MM-dd"
        formatter.timeZone = TimeZone(secondsFromGMT: 0)
        formatter.locale = Locale(identifier: "en_US_POSIX")
        let base = Date()
        let calendar = Calendar.current
        let points: [CostHistoryPoint] = (0..<30).reversed().map { offset in
            let date = calendar.date(byAdding: .day, value: -offset, to: base) ?? base
            let ramp = Double(30 - offset) / 30.0
            let plateau = min(ramp * 1.4, 1.0)
            let wave = Double((offset + Int(scale * 10)) % 5) * 0.45
            let cost = scale * (1.1 + plateau * 10.0 + wave)
            return CostHistoryPoint(day: formatter.string(from: date), totalTokens: 0, costUSD: cost)
        }
        return ProviderMenuProjection(
            provider: providerID,
            title: title,
            sourceLabel: "",
            sourceExplanationLabel: nil,
            authHeadline: nil,
            authDetail: nil,
            authDiagnosticCode: nil,
            authSummaryLabel: nil,
            authRecoveryActions: [],
            warningLabels: [],
            visualState: .healthy,
            stateLabel: "",
            statusLabel: nil,
            identityLabel: nil,
            lastRefreshLabel: "",
            refreshStatusLabel: "",
            costLabel: "",
            todayCostUSD: nil,
            last30DaysCostUSD: points.reduce(0.0) { $0 + $1.costUSD },
            laneDetails: [],
            creditsLabel: nil,
            incidentLabel: nil,
            stale: false,
            isShowingCachedData: false,
            isRefreshing: false,
            error: nil,
            globalIssueLabel: nil,
            historyFractions: [],
            claudeFactors: [],
            adjunct: nil,
            dailyCosts: points
        )
    }
}
