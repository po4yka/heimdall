import HeimdallServices
import WidgetKit

public struct WidgetCenterReloader: WidgetReloading {
    public init() {}

    public func reloadAllTimelines() {
        WidgetCenter.shared.reloadAllTimelines()
    }
}
