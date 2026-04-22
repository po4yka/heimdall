import SwiftUI

/// Shared chart tokens so the three chart views stay short and drift-free.
/// The Apple-Swiss house style is monochrome `Color.primary` opacity ladder
/// plus a single `Color.accentColor` for "today" emphasis; no gradients, no
/// shadows, no chart-grid chrome.
enum ChartStyle {
    static let cardCornerRadius: CGFloat = 8
    static let cardBackgroundOpacity: Double = 0.03
    static let headerTitleOpacity: Double = 0.72
    static let headerCaptionOpacity: Double = 0.68

    static let barFill: Color = Color.primary.opacity(0.55)
    static let barTodayFill: Color = Color.accentColor
    static let barTrackFill: Color = Color.primary.opacity(0.10)
    static let barCornerRadius: CGFloat = 1.5

    static let areaFill: Color = Color.primary.opacity(0.08)
    static let lineStroke: Color = Color.primary.opacity(0.78)
    static let lineWidth: CGFloat = 1.5
    static let todayRuleStroke: Color = Color.accentColor.opacity(0.55)
    static let todayRuleWidth: CGFloat = 2

    static let animation: Animation = .smooth(duration: 0.2)

    /// Ordered category tints for `TokenStackChart`. Stable index order so
    /// `foregroundStyle(by:)` maps categories to the same opacity on each
    /// render. Mirrors the opacity ladder defined by `TokenCategory.tint`.
    static let categoryScale: [Color] = TokenCategory.orderedForStack.map(\.tint)
}

struct ChartHeader: View {
    let title: String
    let caption: String?
    let trailing: AnyView?

    init(title: String, caption: String? = nil, trailing: AnyView? = nil) {
        self.title = title
        self.caption = caption
        self.trailing = trailing
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack(alignment: .firstTextBaseline) {
                Text(self.title)
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(Color.primary.opacity(ChartStyle.headerTitleOpacity))
                Spacer(minLength: 0)
                if let trailing = self.trailing {
                    trailing
                }
            }
            if let caption = self.caption {
                Text(caption)
                    .font(.caption2)
                    .foregroundStyle(Color.primary.opacity(ChartStyle.headerCaptionOpacity))
            }
        }
    }
}

/// Inline category chips used when a chart's card renders a header with a
/// legend. The stock Swift-Charts legend doesn't respect the opacity ladder
/// or sentence-case labels, so we render our own — same shape the previous
/// hand-rolled strip used.
struct TokenCategoryLegend: View {
    var body: some View {
        HStack(spacing: 6) {
            ForEach(Array(TokenCategory.orderedForStack.enumerated()), id: \.offset) { entry in
                let category = entry.element
                HStack(spacing: 3) {
                    Circle()
                        .fill(category.tint)
                        .frame(width: 6, height: 6)
                    Text(category.shortLabel)
                        .font(.system(size: 9))
                        .foregroundStyle(.secondary)
                }
            }
        }
    }
}

/// Day-of-week labels for the last N days ending today; rightmost is today.
enum ChartDayLabels {
    static func lastNDays(_ count: Int, today: Date = Date()) -> [String] {
        guard count > 0 else { return [] }
        let formatter = DateFormatter()
        formatter.dateFormat = "EEE"
        let calendar = Calendar.current
        return (0..<count).map { offset in
            let date = calendar.date(byAdding: .day, value: -(count - 1 - offset), to: today) ?? today
            return formatter.string(from: date)
        }
    }
}
