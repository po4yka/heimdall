import CloudKit
import SwiftUI

final class MobileAppDelegate: NSObject, UIApplicationDelegate {
    var onAcceptedCloudShare: (@Sendable (URL) -> Void)?
    var onRemoteNotification: (@Sendable () async -> RemotePushResult)?

    func application(
        _: UIApplication,
        userDidAcceptCloudKitShareWith metadata: CKShare.Metadata
    ) {
        guard let url = metadata.share.url else { return }
        self.onAcceptedCloudShare?(url)
    }

    func application(
        _: UIApplication,
        didReceiveRemoteNotification _: [AnyHashable: Any],
        fetchCompletionHandler completionHandler: @escaping (UIBackgroundFetchResult) -> Void
    ) {
        guard let handler = self.onRemoteNotification else {
            completionHandler(.noData)
            return
        }
        Task {
            let result = await handler()
            switch result {
            case .newData:
                completionHandler(.newData)
            case .failed:
                completionHandler(.failed)
            }
        }
    }
}

@main
struct HeimdallMobileApp: App {
    @UIApplicationDelegateAdaptor(MobileAppDelegate.self) private var appDelegate
    @State private var coordinator: MobileDashboardCoordinator

    @MainActor
    init() {
        self._coordinator = State(initialValue: HeimdallMobileCompositionRoot().dashboardCoordinator())
    }

    var body: some Scene {
        WindowGroup {
            HeimdallMobileRootView(coordinator: self.coordinator)
                .task {
                    self.coordinator.configure(appDelegate: self.appDelegate)
                }
        }
    }
}
