import Charts
import AppKit
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
    static let hoverAnimation: Animation = .easeOut(duration: 0.14)

    /// Ordered category tints for `TokenStackChart`. Stable index order so
    /// `foregroundStyle(by:)` maps categories to the same opacity on each
    /// render. Mirrors the opacity ladder defined by `TokenCategory.tint`.
    static let categoryScale: [Color] = TokenCategory.orderedForStack.map(\.tint)

    static func snapThreshold(plotWidth: CGFloat, itemCount: Int) -> CGFloat {
        guard plotWidth > 0, itemCount > 0 else { return 0 }
        let laneWidth = plotWidth / CGFloat(max(itemCount, 1))
        return min(28, max(12, laneWidth * 0.5))
    }

    static func inspectorPlacement(index: Int, totalCount: Int) -> ChartInspectorPlacement {
        guard totalCount > 2 else { return .top }
        let normalized = Double(index) / Double(max(totalCount - 1, 1))
        if normalized <= 0.22 {
            return .trailing
        }
        if normalized >= 0.78 {
            return .leading
        }
        return .top
    }

    @MainActor
    static func updateHoverSelection<T: Equatable>(_ selection: inout T?, to newValue: T?) {
        guard selection != newValue else { return }
        withAnimation(Self.hoverAnimation) {
            selection = newValue
        }
    }
}

enum ChartInspectorPlacement: Equatable {
    case leading
    case top
    case trailing

    var annotationPosition: AnnotationPosition {
        switch self {
        case .leading:
            .leading
        case .top:
            .top
        case .trailing:
            .trailing
        }
    }
}

struct ChartInspectorCard: View {
    let title: String
    let lines: [String]

    var body: some View {
        VStack(alignment: .leading, spacing: 3) {
            Text(self.title)
                .font(.system(size: 10, weight: .semibold).monospacedDigit())
                .foregroundStyle(.primary)
            ForEach(self.lines, id: \.self) { line in
                Text(line)
                    .font(.system(size: 9).monospacedDigit())
                    .foregroundStyle(Color.primary.opacity(0.78))
            }
        }
        .padding(.horizontal, 7)
        .padding(.vertical, 6)
        .background(
            RoundedRectangle(cornerRadius: 7, style: .continuous)
                .fill(Color(nsColor: .windowBackgroundColor))
        )
        .overlay(
            RoundedRectangle(cornerRadius: 7, style: .continuous)
                .stroke(Color.primary.opacity(0.12), lineWidth: 1)
        )
    }
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
