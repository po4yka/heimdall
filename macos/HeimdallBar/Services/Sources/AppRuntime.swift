import Foundation

@MainActor
public final class HeimdallAppRuntime {
    public let sessionStore: AppSessionStore
    public let providerRepository: ProviderRepository
    public let refreshCoordinator: RefreshCoordinator
    public let authCoordinator: AuthCoordinator
    public let settingsStore: any SettingsStore
    public let credentialInspector: any ProviderCredentialInspecting

    public init(
        sessionStore: AppSessionStore,
        providerRepository: ProviderRepository,
        refreshCoordinator: RefreshCoordinator,
        authCoordinator: AuthCoordinator,
        settingsStore: any SettingsStore,
        credentialInspector: any ProviderCredentialInspecting
    ) {
        self.sessionStore = sessionStore
        self.providerRepository = providerRepository
        self.refreshCoordinator = refreshCoordinator
        self.authCoordinator = authCoordinator
        self.settingsStore = settingsStore
        self.credentialInspector = credentialInspector
    }
}
