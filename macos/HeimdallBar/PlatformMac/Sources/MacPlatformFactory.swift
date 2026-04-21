import HeimdallServices

public struct MacPlatformCompositionRoot: Sendable {
    private let settingsStore: any SettingsStore
    private let helperRuntime: any HelperRuntime
    private let adjunctLoader: any DashboardAdjunctLoading
    private let browserSessionManager: any BrowserSessionManaging
    private let credentialInspector: any ProviderCredentialInspecting
    private let widgetSnapshotWriter: any WidgetSnapshotWriter
    private let widgetReloader: any WidgetReloading
    private let authCommandRunner: any AuthCommandRunning
    private let providerDataSource: any ProviderDataSource

    public init(
        settingsStore: any SettingsStore = ConfigStore.shared,
        helperRuntime: any HelperRuntime = HeimdallHelperController(),
        browserSessionManager: any BrowserSessionManaging = BrowserSessionController(),
        credentialInspector: any ProviderCredentialInspecting = ProviderCredentialInspector(),
        widgetSnapshotWriter: any WidgetSnapshotWriter = AppGroupWidgetSnapshotStore(),
        widgetReloader: any WidgetReloading = WidgetCenterReloader(),
        authCommandRunner: any AuthCommandRunning = TerminalAuthCommandRunner(),
        liveProviderClientFactory: @escaping @Sendable (Int) -> any LiveProviderClient = { port in
            HeimdallAPIClient(port: port)
        }
    ) {
        self.settingsStore = settingsStore
        self.helperRuntime = helperRuntime
        self.browserSessionManager = browserSessionManager
        self.adjunctLoader = DashboardAdjunctController(sessionManager: browserSessionManager)
        self.credentialInspector = credentialInspector
        self.widgetSnapshotWriter = widgetSnapshotWriter
        self.widgetReloader = widgetReloader
        self.authCommandRunner = authCommandRunner
        self.providerDataSource = LocalProviderDataSource(clientFactory: liveProviderClientFactory)
    }

    @MainActor
    public func appRuntime() -> HeimdallAppRuntime {
        let sessionStore = AppSessionStore(config: self.settingsStore.load())
        let providerRepository = ProviderRepository()
        let liveProviderClient = HeimdallAPIClient(port: sessionStore.config.helperPort)
        let refreshCoordinator = RefreshCoordinator(
            sessionStore: sessionStore,
            repository: providerRepository,
            helperRuntime: self.helperRuntime,
            adjunctLoader: self.adjunctLoader,
            browserSessionManager: self.browserSessionManager,
            widgetSnapshotCoordinator: WidgetSnapshotCoordinator(
                writer: self.widgetSnapshotWriter,
                reloader: self.widgetReloader
            ),
            providerDataSource: self.providerDataSource,
            snapshotSyncer: SnapshotSyncCoordinator(
                client: liveProviderClient,
                store: CloudKitSnapshotSyncStore()
            )
        )
        let authCoordinator = AuthCoordinator(runner: self.authCommandRunner)

        return HeimdallAppRuntime(
            sessionStore: sessionStore,
            providerRepository: providerRepository,
            refreshCoordinator: refreshCoordinator,
            authCoordinator: authCoordinator,
            settingsStore: self.settingsStore,
            credentialInspector: self.credentialInspector
        )
    }

    public func cliDependencies() -> HeimdallCLIDependencies {
        HeimdallCLIDependencies(
            settingsStore: self.settingsStore,
            adjunctLoader: self.adjunctLoader,
            authCommandRunner: self.authCommandRunner,
            providerDataSource: self.providerDataSource
        )
    }
}
