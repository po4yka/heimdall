import SwiftUI

@MainActor
@Observable
final class MobileDashboardCoordinator {
    let dashboard: MobileDashboardModel
    private var hasStarted = false

    init(dashboard: MobileDashboardModel) {
        self.dashboard = dashboard
    }

    var aliasEditorPresentation: Binding<Bool> {
        Binding(
            get: { self.dashboard.isAliasEditorPresented },
            set: { isPresented in
                if isPresented {
                    self.dashboard.presentAliasEditor()
                } else {
                    self.dashboard.dismissAliasEditor()
                }
            }
        )
    }

    func configure(appDelegate: MobileAppDelegate) {
        appDelegate.onAcceptedCloudShare = { [weak self] url in
            Task { @MainActor [weak self] in
                self?.handleOpenURL(url)
            }
        }
        appDelegate.onRemoteNotification = { [weak self] in
            guard let self else { return .failed }
            return await self.handleRemoteNotification()
        }
    }

    func start() async {
        guard !self.hasStarted else { return }
        self.hasStarted = true
        await self.dashboard.load()
    }

    func handleOpenURL(_ url: URL) {
        Task { @MainActor [weak self] in
            await self?.dashboard.acceptShareURL(url)
        }
    }

    func handleScenePhaseChange(_ phase: ScenePhase) {
        guard phase == .active else { return }
        Task { @MainActor [weak self] in
            await self?.dashboard.refresh(reason: .foreground)
        }
    }

    func handleRemoteNotification() async -> RemotePushResult {
        await self.dashboard.handleRemotePush()
    }
}
