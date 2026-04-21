import Foundation
import HeimdallDomain

public protocol SettingsStore: Sendable {
    func load() -> HeimdallBarConfig
    func save(_ config: HeimdallBarConfig) throws
    func validate() throws
}

public enum RefreshActivity: Sendable, Equatable {
    case idle
    case refreshingAll
    case refreshingProvider(ProviderID)

    public var provider: ProviderID? {
        switch self {
        case .idle, .refreshingAll:
            return nil
        case .refreshingProvider(let provider):
            return provider
        }
    }

    public var isRefreshing: Bool {
        switch self {
        case .idle:
            return false
        case .refreshingAll, .refreshingProvider:
            return true
        }
    }
}

public enum SessionImportActivity: Sendable, Equatable {
    case idle
    case importing(ProviderID)
    case resetting(ProviderID)

    public var provider: ProviderID? {
        switch self {
        case .idle:
            return nil
        case .importing(let provider), .resetting(let provider):
            return provider
        }
    }

    public var isActive: Bool {
        self != .idle
    }
}

public enum AppIssueKind: String, Sendable, Equatable {
    case helperStartup
    case refresh
    case settingsSave
    case browserImport
    case authRecovery
    case widgetPersistence
}

public struct AppIssue: Error, Sendable, Equatable, Identifiable, LocalizedError {
    public var kind: AppIssueKind
    public var provider: ProviderID?
    public var message: String
    public var occurredAt: Date

    public init(
        kind: AppIssueKind,
        provider: ProviderID? = nil,
        message: String,
        occurredAt: Date = Date()
    ) {
        self.kind = kind
        self.provider = provider
        self.message = message
        self.occurredAt = occurredAt
    }

    public var id: String {
        "\(self.kind.rawValue):\(self.provider?.rawValue ?? "global"):\(self.occurredAt.timeIntervalSince1970)"
    }

    public var errorDescription: String? {
        self.message
    }
}

public struct RefreshOperationState: Sendable, Equatable {
    public var activity: RefreshActivity
    public var lastCompletedAt: Date?
    public var lastIssue: AppIssue?

    public init(
        activity: RefreshActivity = .idle,
        lastCompletedAt: Date? = nil,
        lastIssue: AppIssue? = nil
    ) {
        self.activity = activity
        self.lastCompletedAt = lastCompletedAt
        self.lastIssue = lastIssue
    }

    public var isRefreshing: Bool {
        self.activity.isRefreshing
    }

    public var provider: ProviderID? {
        self.activity.provider
    }
}

public struct SessionImportOperationState: Sendable, Equatable {
    public var activity: SessionImportActivity
    public var lastIssue: AppIssue?

    public init(
        activity: SessionImportActivity = .idle,
        lastIssue: AppIssue? = nil
    ) {
        self.activity = activity
        self.lastIssue = lastIssue
    }

    public var isActive: Bool {
        self.activity.isActive
    }

    public var provider: ProviderID? {
        self.activity.provider
    }
}

public struct PersistedAppSessionState: Codable, Sendable, Equatable {
    public var selectedProvider: ProviderID
    public var selectedMergeTab: MergeMenuTab

    public init(
        selectedProvider: ProviderID = .claude,
        selectedMergeTab: MergeMenuTab = .overview
    ) {
        self.selectedProvider = selectedProvider
        self.selectedMergeTab = selectedMergeTab
    }
}

public protocol AppSessionStatePersisting: Sendable {
    func loadAppSessionState() -> PersistedAppSessionState?
    func saveAppSessionState(_ state: PersistedAppSessionState)
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

public protocol DashboardAdjunctLoading: Sendable {
    func loadAdjunct(
        provider: ProviderID,
        config: ProviderConfig,
        snapshot: ProviderSnapshot?,
        forceRefresh: Bool,
        allowLiveNavigation: Bool
    ) async -> DashboardAdjunctSnapshot?
}

public protocol BrowserSessionManaging: Sendable {
    func importedSession(provider: ProviderID) async -> ImportedBrowserSession?
    func discoverImportCandidates(provider: ProviderID) async -> [BrowserSessionImportCandidate]
    func importBrowserSession(
        provider: ProviderID,
        candidate: BrowserSessionImportCandidate
    ) async throws -> ImportedBrowserSession
    func resetImportedSession(provider: ProviderID) async throws
}

public enum ProviderCredentialPresence: Sendable, Equatable {
    case present
    case missing
    case unknown
}

public protocol ProviderCredentialInspecting: Sendable {
    func credentialPresence(for provider: ProviderID) -> ProviderCredentialPresence
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
    public var adjunctLoader: any DashboardAdjunctLoading
    public var browserSessionManager: any BrowserSessionManaging
    public var credentialInspector: any ProviderCredentialInspecting
    public var widgetSnapshotWriter: any WidgetSnapshotWriter
    public var widgetReloader: any WidgetReloading
    public var authCommandRunner: any AuthCommandRunning
    public var liveProviderClientFactory: @Sendable (Int) -> any LiveProviderClient

    public init(
        settingsStore: any SettingsStore,
        helperRuntime: any HelperRuntime,
        adjunctLoader: any DashboardAdjunctLoading,
        browserSessionManager: any BrowserSessionManaging,
        credentialInspector: any ProviderCredentialInspecting,
        widgetSnapshotWriter: any WidgetSnapshotWriter,
        widgetReloader: any WidgetReloading,
        authCommandRunner: any AuthCommandRunning,
        liveProviderClientFactory: @escaping @Sendable (Int) -> any LiveProviderClient
    ) {
        self.settingsStore = settingsStore
        self.helperRuntime = helperRuntime
        self.adjunctLoader = adjunctLoader
        self.browserSessionManager = browserSessionManager
        self.credentialInspector = credentialInspector
        self.widgetSnapshotWriter = widgetSnapshotWriter
        self.widgetReloader = widgetReloader
        self.authCommandRunner = authCommandRunner
        self.liveProviderClientFactory = liveProviderClientFactory
    }
}

public struct HeimdallCLIDependencies: Sendable {
    public var settingsStore: any SettingsStore
    public var adjunctLoader: any DashboardAdjunctLoading
    public var authCommandRunner: any AuthCommandRunning
    public var liveProviderClientFactory: @Sendable (Int) -> any LiveProviderClient

    public init(
        settingsStore: any SettingsStore,
        adjunctLoader: any DashboardAdjunctLoading,
        authCommandRunner: any AuthCommandRunning,
        liveProviderClientFactory: @escaping @Sendable (Int) -> any LiveProviderClient
    ) {
        self.settingsStore = settingsStore
        self.adjunctLoader = adjunctLoader
        self.authCommandRunner = authCommandRunner
        self.liveProviderClientFactory = liveProviderClientFactory
    }
}

public struct NoopWidgetReloader: WidgetReloading {
    public init() {}

    public func reloadAllTimelines() {}
}
