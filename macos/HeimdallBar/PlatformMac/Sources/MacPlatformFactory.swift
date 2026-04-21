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
    private let liveProviderClientFactory: @Sendable (Int) -> any LiveProviderClient

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
        self.liveProviderClientFactory = liveProviderClientFactory
    }

    public func appEnvironment() -> HeimdallAppEnvironment {
        HeimdallAppEnvironment(
            settingsStore: self.settingsStore,
            helperRuntime: self.helperRuntime,
            adjunctLoader: self.adjunctLoader,
            browserSessionManager: self.browserSessionManager,
            credentialInspector: self.credentialInspector,
            widgetSnapshotWriter: self.widgetSnapshotWriter,
            widgetReloader: self.widgetReloader,
            authCommandRunner: self.authCommandRunner,
            liveProviderClientFactory: self.liveProviderClientFactory
        )
    }

    public func cliDependencies() -> HeimdallCLIDependencies {
        HeimdallCLIDependencies(
            settingsStore: self.settingsStore,
            adjunctLoader: self.adjunctLoader,
            authCommandRunner: self.authCommandRunner,
            liveProviderClientFactory: self.liveProviderClientFactory
        )
    }
}
