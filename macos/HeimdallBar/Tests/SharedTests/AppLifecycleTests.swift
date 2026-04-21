import Foundation
import Testing
@testable import HeimdallAppUI

struct AppLifecycleTests {
    @Test
    func shutdownCompletesBeforeTimeout() async {
        let completed = await AppDelegate.awaitShutdown(timeoutNanoseconds: 500_000_000) {
            try? await Task.sleep(nanoseconds: 50_000_000)
        }

        #expect(completed)
    }

    @Test
    func shutdownTimesOutWhenCleanupHangs() async {
        let completed = await AppDelegate.awaitShutdown(timeoutNanoseconds: 10_000_000) {
            try? await Task.sleep(nanoseconds: 100_000_000)
        }

        #expect(!completed)
    }

    @Test
    func reopeningMainWindowActivatesAppAndUsesStableSceneIdentifier() {
        var events: [String] = []

        WindowReopener.reopenMainWindow(
            openWindow: { windowID in events.append("open:\(windowID)") },
            activateApp: { events.append("activate") }
        )

        #expect(events == ["activate", "open:\(HeimdallBarSceneID.mainWindow)"])
    }
}
