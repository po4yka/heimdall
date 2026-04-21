import HeimdallServices

public enum MacPlatformFactory {
    public static func appEnvironment() -> HeimdallAppEnvironment {
        HeimdallAppEnvironment(
            settingsStore: ConfigStore.shared,
            helperRuntime: HeimdallHelperController(),
            adjunctProvider: DashboardAdjunctController(),
            widgetSnapshotWriter: AppGroupWidgetSnapshotStore(),
            widgetReloader: WidgetCenterReloader(),
            authCommandRunner: TerminalAuthCommandRunner(),
            liveProviderClientFactory: { port in
                HeimdallAPIClient(port: port)
            }
        )
    }

    public static func cliDependencies() -> HeimdallCLIDependencies {
        HeimdallCLIDependencies(
            settingsStore: ConfigStore.shared,
            adjunctProvider: DashboardAdjunctController(),
            authCommandRunner: TerminalAuthCommandRunner(),
            liveProviderClientFactory: { port in
                HeimdallAPIClient(port: port)
            }
        )
    }
}
