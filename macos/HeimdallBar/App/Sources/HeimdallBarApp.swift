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
        self._model = State(initialValue: AppModel(runtime: runtime))
    }

    @MainActor
    var body: some Scene {
        HeimdallBarScenes(model: self.model)
    }
}
