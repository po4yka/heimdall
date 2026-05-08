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
    public let today: TodayFeatureModel
    public let activity: ActivityFeatureModel
    public let agents: AgentsFeatureModel
    public let costModels: CostModelsFeatureModel
    public let sessions: SessionsFeatureModel
    public let projects: ProjectsFeatureModel
    public let skills: SkillsFeatureModel
    public let instructions: InstructionsFeatureModel
    public let backup: BackupFeatureModel
    public let settings: SettingsFeatureModel
    public let filters: DashboardFiltersModel
    public let savedViews: SavedViewsModel

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
        self.filters = DashboardFiltersModel()
        self.savedViews = SavedViewsModel()
        self.today = TodayFeatureModel(helperPort: sessionStore.config.helperPort)
        let overview = OverviewFeatureModel(
            sessionStore: sessionStore,
            repository: providerRepository,
            refreshCoordinator: refreshCoordinator
        )
        self.overview = overview
        self.activity = ActivityFeatureModel(overview: overview)
        self.agents = AgentsFeatureModel(overview: overview)
        self.costModels = CostModelsFeatureModel(overview: overview)
        self.sessions = SessionsFeatureModel(overview: overview)
        self.projects = ProjectsFeatureModel(overview: overview)
        self.skills = SkillsFeatureModel(helperPort: sessionStore.config.helperPort)
        self.instructions = InstructionsFeatureModel(helperPort: sessionStore.config.helperPort)
        self.backup = BackupFeatureModel(helperPort: sessionStore.config.helperPort)
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
            cloudSyncController: cloudSyncController,
            cloudSyncDiagnosticsContext: runtime.cloudSyncDiagnosticsContext
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

    public var config: HeimdallConfig {
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
