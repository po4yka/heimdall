import WidgetKit
import HeimdallServices

struct HeimdallMobileCompositionRoot {
    @MainActor
    func dashboardModel() -> MobileDashboardModel {
        MobileDashboardModel(
            store: CloudKitSnapshotSyncStore(),
            cache: FileBackedSyncedAggregateCache(),
            preferencesStore: UserDefaultsMobileDashboardPreferencesStore(),
            widgetSnapshotCoordinator: WidgetSnapshotCoordinator(
                writer: AppGroupWidgetSnapshotStore(),
                reloader: MobileWidgetCenterReloader()
            )
        )
    }
}

private struct MobileWidgetCenterReloader: WidgetReloading {
    func reloadAllTimelines() {
        WidgetCenter.shared.reloadAllTimelines()
    }
}
