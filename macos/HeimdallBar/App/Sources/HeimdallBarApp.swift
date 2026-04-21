import HeimdallAppUI
import HeimdallPlatformMac
import SwiftUI

@main
struct HeimdallBarApp: App {
    @State private var model: AppModel

    init() {
        let environment = MacPlatformFactory.appEnvironment()
        self._model = State(initialValue: AppModel(environment: environment))
    }

    @MainActor
    var body: some Scene {
        HeimdallBarScenes(model: self.model)
    }
}
