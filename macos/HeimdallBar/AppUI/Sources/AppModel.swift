import Foundation
import HeimdallDomain
import HeimdallServices
import Observation

@MainActor
@Observable
public final class AppModel {
    public let sessionStore: AppSessionStore
    public let shell: AppShellModel
    public let overview: OverviewFeatureModel
    public let liveMonitor: LiveMonitorFeatureModel
    public let settings: SettingsFeatureModel

    private let refreshCoordinator: RefreshCoordinator
    private let providerRepository: ProviderRepository
    private let providerFeatures: [ProviderID: ProviderFeatureModel]
    private let cloudSyncController: (any CloudSyncControlling)?
    private var hasStarted: Bool

    public init(runtime: HeimdallAppRuntime) {
        let sessionStore = runtime.sessionStore
        let providerRepository = runtime.providerRepository
        let refreshCoordinator = runtime.refreshCoordinator
        let authCoordinator = runtime.authCoordinator
        let settingsStore = runtime.settingsStore
        let credentialInspector = runtime.credentialInspector
        let localNotificationCoordinator = runtime.localNotificationCoordinator
        let cloudSyncController = runtime.cloudSyncController

        self.sessionStore = sessionStore
        self.shell = AppShellModel(sessionStore: sessionStore)
        self.overview = OverviewFeatureModel(
            sessionStore: sessionStore,
            repository: providerRepository,
            refreshCoordinator: refreshCoordinator
        )
        self.liveMonitor = LiveMonitorFeatureModel(
            sessionStore: sessionStore,
            clientFactory: runtime.liveMonitorClientFactory
        )
        self.settings = SettingsFeatureModel(
            sessionStore: sessionStore,
            repository: providerRepository,
            settingsStore: settingsStore,
            refreshCoordinator: refreshCoordinator,
            localNotificationCoordinator: localNotificationCoordinator,
            cloudSyncController: cloudSyncController
        )
        self.refreshCoordinator = refreshCoordinator
        self.providerRepository = providerRepository
        self.cloudSyncController = cloudSyncController
        self.providerFeatures = Dictionary(uniqueKeysWithValues: ProviderID.allCases.map { provider in
            (
                provider,
                ProviderFeatureModel(
                    provider: provider,
                    sessionStore: sessionStore,
                    repository: providerRepository,
                    refreshCoordinator: refreshCoordinator,
                    authCoordinator: authCoordinator,
                    credentialInspector: credentialInspector
                )
            )
        })
        self.hasStarted = false
        self.settings.onSettingsSaved = { [weak shell = self.shell] in
            shell?.syncSelections()
        }
        self.shell.syncSelections()
    }

    public var config: HeimdallBarConfig {
        self.sessionStore.config
    }

    public var visibleProviders: [ProviderID] {
        self.sessionStore.visibleProviders
    }

    public func providerModel(for provider: ProviderID) -> ProviderFeatureModel {
        self.providerFeatures[provider]!
    }

    public func start() {
        guard !self.hasStarted else { return }
        self.hasStarted = true
        self.refreshCoordinator.start()
        Task { [weak self] in
            await self?.settings.refreshCloudSyncState()
        }
    }

    public func prepareForExit() async {
        await self.refreshCoordinator.stop()
    }

    public func syncShellSelections() {
        self.shell.syncSelections()
    }

    public var globalIssue: AppIssue? {
        self.providerRepository.issue(for: nil)
    }

    public func handleIncomingCloudShare(url: URL) async {
        guard let cloudSyncController else { return }
        do {
            let state = try await cloudSyncController.acceptShareURL(url)
            self.providerRepository.setCloudSyncState(state)
            self.sessionStore.cloudSyncState = state
            self.providerRepository.setSyncedAggregate(try await cloudSyncController.loadAggregateSnapshot())
            self.providerRepository.clearIssue(kind: .snapshotSync)
        } catch {
            self.providerRepository.recordIssue(AppIssue(kind: .snapshotSync, message: error.localizedDescription))
        }
    }
}
