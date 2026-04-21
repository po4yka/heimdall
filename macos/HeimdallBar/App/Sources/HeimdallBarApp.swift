import HeimdallAppUI
import HeimdallPlatformMac
import SwiftUI

@main
struct HeimdallBarApp: App {
    @State private var model: AppModel

    @MainActor
    init() {
        let compositionRoot = MacPlatformCompositionRoot()
        let runtime = compositionRoot.appRuntime()
        let model = AppModel(runtime: runtime)
        model.start()
        self._model = State(initialValue: model)
    }

    @MainActor
    var body: some Scene {
        HeimdallBarScenes(model: self.model)
    }
}
