import HeimdallDomain
import HeimdallServices
import Observation

@MainActor
@Observable
public final class AppModel {
    public let sessionStore: AppSessionStore
    public let shell: AppShellModel
    public let overview: OverviewFeatureModel
    public let settings: SettingsFeatureModel

    private let refreshCoordinator: RefreshCoordinator
    private let providerRepository: ProviderRepository
    private let providerFeatures: [ProviderID: ProviderFeatureModel]
    private var hasStarted: Bool

    public init(environment: HeimdallAppEnvironment) {
        let sessionStore = AppSessionStore(config: environment.settingsStore.load())
        let providerRepository = ProviderRepository()
        let refreshCoordinator = RefreshCoordinator(
            sessionStore: sessionStore,
            repository: providerRepository,
            helperRuntime: environment.helperRuntime,
            adjunctLoader: environment.adjunctLoader,
            browserSessionManager: environment.browserSessionManager,
            widgetSnapshotCoordinator: WidgetSnapshotCoordinator(
                writer: environment.widgetSnapshotWriter,
                reloader: environment.widgetReloader
            ),
            liveProviderClientFactory: environment.liveProviderClientFactory
        )
        let authCoordinator = AuthCoordinator(runner: environment.authCommandRunner)

        self.sessionStore = sessionStore
        self.shell = AppShellModel(sessionStore: sessionStore)
        self.overview = OverviewFeatureModel(
            sessionStore: sessionStore,
            repository: providerRepository,
            refreshCoordinator: refreshCoordinator
        )
        self.settings = SettingsFeatureModel(
            sessionStore: sessionStore,
            repository: providerRepository,
            settingsStore: environment.settingsStore,
            refreshCoordinator: refreshCoordinator
        )
        self.refreshCoordinator = refreshCoordinator
        self.providerRepository = providerRepository
        self.providerFeatures = Dictionary(uniqueKeysWithValues: ProviderID.allCases.map { provider in
            (
                provider,
                ProviderFeatureModel(
                    provider: provider,
                    sessionStore: sessionStore,
                    repository: providerRepository,
                    refreshCoordinator: refreshCoordinator,
                    authCoordinator: authCoordinator,
                    credentialInspector: environment.credentialInspector
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
}
