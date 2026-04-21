import AppKit
import HeimdallDomain

enum MenuBarMeterRenderer {
    static func image(for projection: ProviderMenuProjection) -> NSImage {
        self.image(
            topFraction: fraction(from: projection.laneDetails.first?.remainingPercent),
            bottomFraction: fraction(from: projection.laneDetails.dropFirst().first?.remainingPercent),
            state: projection.visualState,
            merged: false
        )
    }

    static func mergedImage(from items: [ProviderMenuProjection], isRefreshing: Bool) -> NSImage {
        let sorted = items.sorted { $0.provider.rawValue < $1.provider.rawValue }
        let top = sorted.first
        let bottom = sorted.dropFirst().first ?? sorted.first
        let state = mergedState(items: items, isRefreshing: isRefreshing)
        return self.image(
            topFraction: fraction(from: top?.laneDetails.first?.remainingPercent),
            bottomFraction: fraction(from: bottom?.laneDetails.first?.remainingPercent),
            state: state,
            merged: true
        )
    }

    private static func image(
        topFraction: CGFloat,
        bottomFraction: CGFloat,
        state: ProviderVisualState,
        merged: Bool
    ) -> NSImage {
        let size = NSSize(width: 18, height: 18)
        let image = NSImage(size: size)
        image.lockFocus()

        let metrics = merged
            ? (topY: CGFloat(10), topHeight: CGFloat(4), bottomY: CGFloat(4), bottomHeight: CGFloat(4))
            : (topY: CGFloat(11), topHeight: CGFloat(3), bottomY: CGFloat(5), bottomHeight: CGFloat(5))
        let alpha = baseAlpha(for: state)
        let backgroundAlpha: CGFloat = merged ? 0.16 : 0.14

        NSColor.labelColor.withAlphaComponent(backgroundAlpha * alpha).setFill()
        NSBezierPath(roundedRect: NSRect(x: 2, y: metrics.topY, width: 14, height: metrics.topHeight), xRadius: 1.5, yRadius: 1.5).fill()
        NSBezierPath(roundedRect: NSRect(x: 2, y: metrics.bottomY, width: 14, height: metrics.bottomHeight), xRadius: 1.5, yRadius: 1.5).fill()

        NSColor.labelColor.withAlphaComponent(alpha).setFill()
        NSBezierPath(roundedRect: NSRect(x: 2, y: metrics.topY, width: max(1.5, 14 * topFraction), height: metrics.topHeight), xRadius: 1.5, yRadius: 1.5).fill()
        NSBezierPath(roundedRect: NSRect(x: 2, y: metrics.bottomY, width: max(1.5, 14 * bottomFraction), height: metrics.bottomHeight), xRadius: 1.5, yRadius: 1.5).fill()

        drawStateOverlay(state: state)

        image.unlockFocus()
        image.isTemplate = true
        return image
    }

    private static func fraction(from remainingPercent: Int?) -> CGFloat {
        CGFloat(max(0, min(100, remainingPercent ?? 0))) / 100
    }

    private static func baseAlpha(for state: ProviderVisualState) -> CGFloat {
        switch state {
        case .healthy:
            return 1
        case .refreshing:
            return 0.95
        case .stale:
            return 0.45
        case .degraded:
            return 0.85
        case .incident:
            return 1
        case .error:
            return 0.55
        }
    }

    private static func mergedState(items: [ProviderMenuProjection], isRefreshing: Bool) -> ProviderVisualState {
        if items.contains(where: { $0.visualState == .error }) {
            return .error
        }
        if items.contains(where: { $0.visualState == .incident }) {
            return .incident
        }
        if items.contains(where: { $0.visualState == .degraded }) {
            return .degraded
        }
        if items.contains(where: { $0.visualState == .stale }) {
            return .stale
        }
        if isRefreshing || items.contains(where: \.isRefreshing) {
            return .refreshing
        }
        return .healthy
    }

    private static func drawStateOverlay(state: ProviderVisualState) {
        switch state {
        case .healthy:
            break
        case .refreshing:
            NSColor.labelColor.withAlphaComponent(0.95).setFill()
            NSBezierPath(ovalIn: NSRect(x: 13, y: 13, width: 3, height: 3)).fill()
        case .stale:
            NSColor.labelColor.withAlphaComponent(0.55).setStroke()
            let path = NSBezierPath()
            path.lineWidth = 1
            path.move(to: NSPoint(x: 13, y: 15))
            path.line(to: NSPoint(x: 16, y: 15))
            path.stroke()
        case .degraded:
            NSColor.labelColor.withAlphaComponent(0.95).setStroke()
            let path = NSBezierPath(ovalIn: NSRect(x: 12.5, y: 12.5, width: 4, height: 4))
            path.lineWidth = 1
            path.stroke()
        case .incident:
            NSColor.labelColor.withAlphaComponent(1).setFill()
            NSBezierPath(ovalIn: NSRect(x: 12.5, y: 12.5, width: 4, height: 4)).fill()
        case .error:
            NSColor.labelColor.withAlphaComponent(0.95).setStroke()
            let path = NSBezierPath()
            path.lineWidth = 1
            path.move(to: NSPoint(x: 12.75, y: 12.75))
            path.line(to: NSPoint(x: 16.25, y: 16.25))
            path.move(to: NSPoint(x: 16.25, y: 12.75))
            path.line(to: NSPoint(x: 12.75, y: 16.25))
            path.stroke()
        }
    }
}
