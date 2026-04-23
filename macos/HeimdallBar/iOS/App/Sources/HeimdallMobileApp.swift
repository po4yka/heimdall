import CloudKit
import SwiftUI

final class MobileAppDelegate: NSObject, UIApplicationDelegate {
    var onAcceptedCloudShare: (@Sendable (URL) -> Void)?

    func application(
        _: UIApplication,
        userDidAcceptCloudKitShareWith metadata: CKShare.Metadata
    ) {
        guard let url = metadata.share.url else { return }
        self.onAcceptedCloudShare?(url)
    }
}

@main
struct HeimdallMobileApp: App {
    @UIApplicationDelegateAdaptor(MobileAppDelegate.self) private var appDelegate
    @State private var model: MobileDashboardModel

    @MainActor
    init() {
        self._model = State(initialValue: HeimdallMobileCompositionRoot().dashboardModel())
    }

    var body: some Scene {
        WindowGroup {
            HeimdallMobileRootView(model: self.model)
                .onAppear {
                    self.appDelegate.onAcceptedCloudShare = { url in
                        Task { @MainActor in
                            await self.model.acceptShareURL(url)
                        }
                    }
                }
        }
    }
}
