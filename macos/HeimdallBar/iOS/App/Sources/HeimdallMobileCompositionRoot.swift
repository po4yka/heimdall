import HeimdallServices

struct HeimdallMobileCompositionRoot {
    @MainActor
    func dashboardModel() -> MobileDashboardModel {
        MobileDashboardModel(store: CloudKitSnapshotSyncStore())
    }
}
