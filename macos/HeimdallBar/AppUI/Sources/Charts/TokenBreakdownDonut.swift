import Charts
import HeimdallDomain
import SwiftUI

/// Donut chart for a single period's token mix. Companion to `TokenBreakdownRow`:
/// the row gives the proportional rail, the donut gives a sector view useful when
/// comparing two periods side-by-side or when the total is large enough that the
/// ring conveys magnitude better than a thin bar.
struct TokenBreakdownDonut: View {
    let title: String
    let breakdown: TokenBreakdown
    var diameter: CGFloat = 116

    struct Entry: Identifiable, Hashable {
        let category: TokenCategory
        let tokens: Int
        var id: String { self.category.label }
    }

    var body: some View {
        let entries = Self.entries(from: self.breakdown)
        VStack(alignment: .leading, spacing: 6) {
            ChartHeader(
                title: "Token mix",
                caption: self.title,
                trailing: AnyView(self.headerSummary)
            )
            ViewThatFits(in: .horizontal) {
                HStack(alignment: .center, spacing: 12) {
                    self.donutView(entries: entries)
                    self.detailsColumn(entries: entries)
                }
                VStack(alignment: .leading, spacing: 10) {
                    self.donutView(entries: entries)
                    self.detailsColumn(entries: entries)
                }
            }
        }
        .padding(8)
        .menuCardBackground(
            opacity: ChartStyle.cardBackgroundOpacity,
            cornerRadius: ChartStyle.cardCornerRadius
        )
        .accessibilityElement(children: .combine)
        .accessibilityLabel("Token mix donut for \(self.title)")
    }

    private var headerSummary: some View {
        VStack(alignment: .trailing, spacing: 1) {
            Text(Self.compactTokenCount(self.breakdown.total))
                .font(.caption.monospacedDigit().weight(.semibold))
                .foregroundStyle(Color.primary)
            Text("total")
                .font(.system(size: 9, weight: .medium))
                .foregroundStyle(.secondary)
                .textCase(.uppercase)
                .tracking(0.35)
        }
    }

    @ViewBuilder
    private func donutView(entries: [Entry]) -> some View {
        if entries.isEmpty {
            ZStack {
                Circle()
                    .stroke(Color.primary.opacity(0.08), lineWidth: 8)
                    .frame(width: self.diameter, height: self.diameter)
                Text("No tokens\nrecorded yet.")
                    .font(.system(size: 9))
                    .foregroundStyle(.secondary)
                    .multilineTextAlignment(.center)
                    .frame(width: self.diameter * 0.6)
            }
            .frame(width: self.diameter, height: self.diameter)
        } else {
            ZStack {
                Chart(entries) { entry in
                    SectorMark(
                        angle: .value("Tokens", entry.tokens),
                        innerRadius: .ratio(0.68),
                        outerRadius: .ratio(0.98)
                    )
                    .foregroundStyle(by: .value("Category", entry.category.label))
                    .accessibilityLabel(entry.category.label)
                    .accessibilityValue("\(Self.compactTokenCount(entry.tokens)) tokens")
                }
                .chartForegroundStyleScale(
                    domain: TokenCategory.orderedForStack.map(\.label),
                    range: ChartStyle.categoryScale
                )
                .chartLegend(.hidden)
                .frame(width: self.diameter, height: self.diameter)
                .help(Self.tooltip(for: entries, total: self.breakdown.total))
                .animation(ChartStyle.animation, value: entries)

                VStack(spacing: 2) {
                    Text(Self.compactTokenCount(self.breakdown.total))
                        .font(.title3.monospacedDigit().weight(.semibold))
                        .minimumScaleFactor(0.6)
                        .lineLimit(1)
                    Text(self.title)
                        .font(.system(size: 10, weight: .medium))
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
                .frame(width: self.diameter * 0.52)
            }
            .frame(width: self.diameter, height: self.diameter)
        }
    }

    private func detailsColumn(entries: [Entry]) -> some View {
        VStack(alignment: .leading, spacing: 7) {
            if entries.isEmpty {
                Text("No category data available yet.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            } else {
                ForEach(entries) { entry in
                    TokenBreakdownLegendRow(
                        category: entry.category,
                        tokens: entry.tokens,
                        share: Self.share(for: entry.tokens, total: self.breakdown.total)
                    )
                }
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    nonisolated static func entries(from breakdown: TokenBreakdown) -> [Entry] {
        TokenCategory.orderedForStack.compactMap { category in
            let tokens = category.value(for: breakdown)
            guard tokens > 0 else { return nil }
            return Entry(category: category, tokens: tokens)
        }
    }

    nonisolated static func share(for tokens: Int, total: Int) -> Double {
        guard total > 0 else { return 0 }
        return Double(tokens) / Double(total)
    }

    nonisolated static func percentLabel(for share: Double) -> String {
        guard share > 0 else { return "0%" }
        let percentage = share * 100
        if percentage < 1 {
            return "<1%"
        }
        if percentage >= 10 {
            return "\(Int(percentage.rounded()))%"
        }
        return String(format: "%.1f%%", percentage)
    }

    nonisolated static func tooltip(for entries: [Entry], total: Int) -> String {
        entries.map { entry in
            let share = Self.share(for: entry.tokens, total: total)
            return "\(entry.category.label): \(Self.percentLabel(for: share)) · \(Self.compactTokenCount(entry.tokens)) tokens"
        }
        .joined(separator: "\n")
    }

    nonisolated static func compactTokenCount(_ count: Int) -> String {
        let value = Double(count)
        if value >= 1_000_000_000 {
            return String(format: "%.1fB", value / 1_000_000_000)
        }
        if value >= 1_000_000 {
            return String(format: "%.1fM", value / 1_000_000)
        }
        if value >= 1_000 {
            return String(format: "%.1fK", value / 1_000)
        }
        return "\(count)"
    }
}

private struct TokenBreakdownLegendRow: View {
    let category: TokenCategory
    let tokens: Int
    let share: Double

    var body: some View {
        HStack(alignment: .firstTextBaseline, spacing: 8) {
            HStack(spacing: 6) {
                RoundedRectangle(cornerRadius: 2, style: .continuous)
                    .fill(self.category.tint)
                    .frame(width: 10, height: 10)
                Text(self.category.label)
                    .font(.caption)
                    .foregroundStyle(Color.primary.opacity(0.82))
                    .lineLimit(1)
            }
            Spacer(minLength: 8)
            Text(TokenBreakdownDonut.percentLabel(for: self.share))
                .font(.caption.monospacedDigit().weight(.semibold))
                .foregroundStyle(Color.primary.opacity(0.88))
                .frame(minWidth: 34, alignment: .trailing)
            Text(TokenBreakdownDonut.compactTokenCount(self.tokens))
                .font(.caption.monospacedDigit())
                .foregroundStyle(.secondary)
                .frame(minWidth: 48, alignment: .trailing)
        }
        .accessibilityElement(children: .combine)
        .accessibilityLabel("\(self.category.label), \(TokenBreakdownDonut.percentLabel(for: self.share)), \(TokenBreakdownDonut.compactTokenCount(self.tokens)) tokens")
        .help("\(self.category.label): \(TokenBreakdownDonut.percentLabel(for: self.share)) · \(TokenBreakdownDonut.compactTokenCount(self.tokens)) tokens")
    }
}

#Preview("Today — mixed breakdown") {
    TokenBreakdownDonut(
        title: "Today",
        breakdown: TokenBreakdown(
            input: 12_400,
            output: 8_750,
            cacheRead: 42_300,
            cacheCreation: 3_100,
            reasoningOutput: 1_850
        )
    )
    .padding()
    .frame(width: 320)
}

#Preview("30 days — large breakdown") {
    TokenBreakdownDonut(
        title: "30 days",
        breakdown: TokenBreakdown(
            input: 4_820_000,
            output: 3_150_000,
            cacheRead: 18_400_000,
            cacheCreation: 1_270_000,
            reasoningOutput: 640_000
        ),
        diameter: 96
    )
    .padding()
    .frame(width: 320)
}
