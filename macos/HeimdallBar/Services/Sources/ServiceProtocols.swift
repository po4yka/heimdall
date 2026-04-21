import Foundation
import HeimdallDomain

public protocol SettingsStore: Sendable {
    func load() -> HeimdallBarConfig
    func save(_ config: HeimdallBarConfig) throws
    func validate() throws
}

public protocol LiveProviderClient: Sendable {
    func fetchSnapshots() async throws -> ProviderSnapshotEnvelope
    func refresh(provider: ProviderID?) async throws -> ProviderSnapshotEnvelope
    func fetchCostSummary(provider: ProviderID) async throws -> CostSummaryEnvelope
}

public protocol SyncProviderClient: Sendable {
    func fetchSyncedSnapshots() async throws -> ProviderSnapshotEnvelope
}

public protocol HelperRuntime: Sendable {
    func ensureServerRunning(port: Int) async -> Bool
    func stopOwnedHelper() async
}

public protocol AdjunctProvider: Sendable {
    func loadAdjunct(
        provider: ProviderID,
        config: ProviderConfig,
        snapshot: ProviderSnapshot?,
        forceRefresh: Bool,
        allowLiveNavigation: Bool
    ) async -> DashboardAdjunctSnapshot?
    func importedSession(provider: ProviderID) async -> ImportedBrowserSession?
    func discoverImportCandidates(provider: ProviderID) async -> [BrowserSessionImportCandidate]
    func importBrowserSession(
        provider: ProviderID,
        candidate: BrowserSessionImportCandidate
    ) async throws -> ImportedBrowserSession
    func resetImportedSession(provider: ProviderID) async throws
}

public protocol WidgetSnapshotWriter: Sendable {
    func save(_ snapshot: WidgetSnapshot) throws -> WidgetSnapshotSaveResult
    func load() -> WidgetSnapshotLoadResult
}

public protocol WidgetReloading: Sendable {
    func reloadAllTimelines()
}

public protocol AuthCommandRunning: Sendable {
    func runAuthCommand(
        provider: ProviderID,
        title: String,
        command: String
    ) throws
}

public struct HeimdallAppEnvironment: Sendable {
    public var settingsStore: any SettingsStore
    public var helperRuntime: any HelperRuntime
    public var adjunctProvider: any AdjunctProvider
    public var widgetSnapshotWriter: any WidgetSnapshotWriter
    public var widgetReloader: any WidgetReloading
    public var authCommandRunner: any AuthCommandRunning
    public var liveProviderClientFactory: @Sendable (Int) -> any LiveProviderClient

    public init(
        settingsStore: any SettingsStore,
        helperRuntime: any HelperRuntime,
        adjunctProvider: any AdjunctProvider,
        widgetSnapshotWriter: any WidgetSnapshotWriter,
        widgetReloader: any WidgetReloading,
        authCommandRunner: any AuthCommandRunning,
        liveProviderClientFactory: @escaping @Sendable (Int) -> any LiveProviderClient
    ) {
        self.settingsStore = settingsStore
        self.helperRuntime = helperRuntime
        self.adjunctProvider = adjunctProvider
        self.widgetSnapshotWriter = widgetSnapshotWriter
        self.widgetReloader = widgetReloader
        self.authCommandRunner = authCommandRunner
        self.liveProviderClientFactory = liveProviderClientFactory
    }
}

public struct HeimdallCLIDependencies: Sendable {
    public var settingsStore: any SettingsStore
    public var adjunctProvider: any AdjunctProvider
    public var authCommandRunner: any AuthCommandRunning
    public var liveProviderClientFactory: @Sendable (Int) -> any LiveProviderClient

    public init(
        settingsStore: any SettingsStore,
        adjunctProvider: any AdjunctProvider,
        authCommandRunner: any AuthCommandRunning,
        liveProviderClientFactory: @escaping @Sendable (Int) -> any LiveProviderClient
    ) {
        self.settingsStore = settingsStore
        self.adjunctProvider = adjunctProvider
        self.authCommandRunner = authCommandRunner
        self.liveProviderClientFactory = liveProviderClientFactory
    }
}

public struct NoopWidgetReloader: WidgetReloading {
    public init() {}

    public func reloadAllTimelines() {}
}
