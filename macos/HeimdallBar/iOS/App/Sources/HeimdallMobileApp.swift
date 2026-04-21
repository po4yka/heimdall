import SwiftUI

@main
struct HeimdallMobileApp: App {
    @State private var model: MobileDashboardModel

    @MainActor
    init() {
        self._model = State(initialValue: HeimdallMobileCompositionRoot().dashboardModel())
    }

    var body: some Scene {
        WindowGroup {
            HeimdallMobileRootView(model: self.model)
        }
    }
}
