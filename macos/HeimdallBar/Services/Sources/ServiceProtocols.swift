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
    case snapshotSync
    case widgetPersistence
    case localNotifications
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
    public var liveMonitorPreferences: LiveMonitorPreferences?

    public init(
        selectedProvider: ProviderID = .claude,
        selectedMergeTab: MergeMenuTab = .overview,
        liveMonitorPreferences: LiveMonitorPreferences? = nil
    ) {
        self.selectedProvider = selectedProvider
        self.selectedMergeTab = selectedMergeTab
        self.liveMonitorPreferences = liveMonitorPreferences
    }
}

public enum LiveMonitorDensity: String, Codable, CaseIterable, Sendable, Identifiable {
    case expanded
    case compact

    public var id: String { self.rawValue }

    public var title: String {
        switch self {
        case .expanded:
            return "Expanded"
        case .compact:
            return "Compact"
        }
    }
}

public enum LiveMonitorPanelID: String, Codable, CaseIterable, Sendable, Identifiable {
    case activeBlock = "active_block"
    case depletionForecast = "depletion_forecast"
    case quotaSuggestions = "quota_suggestions"
    case contextWindow = "context_window"
    case recentSession = "recent_session"
    case warnings = "warnings"

    public var id: String { self.rawValue }

    public var title: String {
        switch self {
        case .activeBlock:
            return "Active Block"
        case .depletionForecast:
            return "Depletion Forecast"
        case .quotaSuggestions:
            return "Suggested Quotas"
        case .contextWindow:
            return "Context Window"
        case .recentSession:
            return "Recent Session"
        case .warnings:
            return "Warnings"
        }
    }
}

public struct LiveMonitorPreferences: Codable, Sendable, Equatable {
    public var focus: LiveMonitorFocus
    public var density: LiveMonitorDensity
    public var hiddenPanels: [LiveMonitorPanelID]

    public static let `default` = LiveMonitorPreferences(
        focus: .all,
        density: .expanded,
        hiddenPanels: []
    )

    public init(
        focus: LiveMonitorFocus = .all,
        density: LiveMonitorDensity = .expanded,
        hiddenPanels: [LiveMonitorPanelID] = []
    ) {
        self.focus = focus
        self.density = density
        self.hiddenPanels = hiddenPanels
    }
}

public protocol AppSessionStatePersisting: Sendable {
    func loadAppSessionState() -> PersistedAppSessionState?
    func saveAppSessionState(_ state: PersistedAppSessionState)
}

public protocol CloudSyncStatePersisting: Sendable {
    func loadInstallationID() -> String?
    func saveInstallationID(_ installationID: String)
    func loadCloudSyncSpaceState() -> CloudSyncSpaceState?
    func saveCloudSyncSpaceState(_ state: CloudSyncSpaceState)
}

public struct CachedSyncedAggregateEnvelope: Codable, Sendable {
    public var aggregate: SyncedAggregateEnvelope
    public var cachedAt: String
    public var lastSuccessfulRefreshAt: String

    public init(
        aggregate: SyncedAggregateEnvelope,
        cachedAt: String,
        lastSuccessfulRefreshAt: String
    ) {
        self.aggregate = aggregate
        self.cachedAt = cachedAt
        self.lastSuccessfulRefreshAt = lastSuccessfulRefreshAt
    }
}

public protocol SyncedAggregateCaching: Sendable {
    func loadCachedAggregate() async throws -> CachedSyncedAggregateEnvelope?
    func saveCachedAggregate(_ cached: CachedSyncedAggregateEnvelope) async throws
}

public enum NotificationAuthorizationStatus: Sendable, Equatable {
    case notDetermined
    case denied
    case authorized
}

public struct LocalNotificationRequest: Sendable, Equatable {
    public var identifier: String
    public var title: String
    public var body: String

    public init(identifier: String, title: String, body: String) {
        self.identifier = identifier
        self.title = title
        self.body = body
    }
}

public struct PersistedLocalNotificationState: Codable, Sendable, Equatable {
    public var lastKnownActive: [String: Bool]
    public var lastFiredDayKeys: [String: String]

    public init(
        lastKnownActive: [String: Bool] = [:],
        lastFiredDayKeys: [String: String] = [:]
    ) {
        self.lastKnownActive = lastKnownActive
        self.lastFiredDayKeys = lastFiredDayKeys
    }
}

public protocol NotificationAuthorizationManaging: Sendable {
    func authorizationStatus() async -> NotificationAuthorizationStatus
    func requestAuthorization() async throws -> Bool
}

public protocol LocalNotificationScheduling: Sendable {
    func schedule(_ request: LocalNotificationRequest) async throws
}

public protocol LocalNotificationStatePersisting: Sendable {
    func loadState() -> PersistedLocalNotificationState
    func saveState(_ state: PersistedLocalNotificationState)
    func clearState()
}

public protocol LocalNotificationCoordinating: Sendable {
    func handleConfigChange(previous: HeimdallBarConfig, current: HeimdallBarConfig) async -> AppIssue?
    func process(envelope: ProviderSnapshotEnvelope, config: HeimdallBarConfig) async -> AppIssue?
}

public protocol LiveProviderClient: Sendable {
    func fetchSnapshots() async throws -> ProviderSnapshotEnvelope
    func fetchStartupSnapshots() async throws -> ProviderSnapshotEnvelope
    func refresh(provider: ProviderID?) async throws -> ProviderSnapshotEnvelope
    func fetchCostSummary(provider: ProviderID) async throws -> CostSummaryEnvelope
}

public extension LiveProviderClient {
    func fetchStartupSnapshots() async throws -> ProviderSnapshotEnvelope {
        try await self.fetchSnapshots()
    }
}

public protocol LiveMonitorClient: Sendable {
    func fetchLiveMonitor() async throws -> LiveMonitorEnvelope
    func liveMonitorEvents() -> AsyncThrowingStream<String, Error>
}

public protocol SyncProviderClient: Sendable {
    func fetchSyncedSnapshots() async throws -> ProviderSnapshotEnvelope
}

public protocol MobileSnapshotClient: Sendable {
    func fetchMobileSnapshot() async throws -> MobileSnapshotEnvelope
}

public protocol CloudSyncControlling: Sendable {
    func loadAggregateSnapshot() async throws -> SyncedAggregateEnvelope?
    func loadCloudSyncSpaceState() async throws -> CloudSyncSpaceState
    func prepareOwnerShare() async throws -> CloudSyncSpaceState
    func acceptShareURL(_ url: URL) async throws -> CloudSyncSpaceState
}

public protocol SnapshotSyncStore: CloudSyncControlling {
    func loadLiveAggregateSnapshot() async throws -> SyncedAggregateEnvelope?
    func loadLegacySnapshot() async throws -> MobileSnapshotEnvelope?
    func saveLatestSnapshot(_ snapshot: MobileSnapshotEnvelope) async throws -> SyncedAggregateEnvelope
}

public extension SnapshotSyncStore {
    func loadLatestSnapshot() async throws -> MobileSnapshotEnvelope? {
        if let aggregate = try await self.loadAggregateSnapshot() {
            return aggregate.mobileSnapshotCompatibility
        }
        return try await self.loadLegacySnapshot()
    }
}

public protocol SnapshotSyncing: Sendable {
    func syncLatestSnapshot() async throws -> SyncedAggregateEnvelope
    func loadCloudSyncSpaceState() async throws -> CloudSyncSpaceState
}

public protocol ProviderDataSource: Sendable {
    func fetchSnapshots(
        config: HeimdallBarConfig,
        refresh: Bool,
        provider: ProviderID?
    ) async throws -> ProviderSnapshotEnvelope
    func fetchCostSummary(
        config: HeimdallBarConfig,
        provider: ProviderID
    ) async throws -> CostSummaryEnvelope
}

public protocol StartupOptimizedProviderDataSource: ProviderDataSource {
    func fetchStartupSnapshots(config: HeimdallBarConfig) async throws -> ProviderSnapshotEnvelope
}

public extension StartupOptimizedProviderDataSource {
    func fetchStartupSnapshots(config: HeimdallBarConfig) async throws -> ProviderSnapshotEnvelope {
        try await self.fetchSnapshots(config: config, refresh: false, provider: nil)
    }
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
    public var providerDataSource: any ProviderDataSource

    public init(
        settingsStore: any SettingsStore,
        helperRuntime: any HelperRuntime,
        adjunctLoader: any DashboardAdjunctLoading,
        browserSessionManager: any BrowserSessionManaging,
        credentialInspector: any ProviderCredentialInspecting,
        widgetSnapshotWriter: any WidgetSnapshotWriter,
        widgetReloader: any WidgetReloading,
        authCommandRunner: any AuthCommandRunning,
        providerDataSource: any ProviderDataSource
    ) {
        self.settingsStore = settingsStore
        self.helperRuntime = helperRuntime
        self.adjunctLoader = adjunctLoader
        self.browserSessionManager = browserSessionManager
        self.credentialInspector = credentialInspector
        self.widgetSnapshotWriter = widgetSnapshotWriter
        self.widgetReloader = widgetReloader
        self.authCommandRunner = authCommandRunner
        self.providerDataSource = providerDataSource
    }
}

public struct HeimdallCLIDependencies: Sendable {
    public var settingsStore: any SettingsStore
    public var adjunctLoader: any DashboardAdjunctLoading
    public var authCommandRunner: any AuthCommandRunning
    public var providerDataSource: any ProviderDataSource

    public init(
        settingsStore: any SettingsStore,
        adjunctLoader: any DashboardAdjunctLoading,
        authCommandRunner: any AuthCommandRunning,
        providerDataSource: any ProviderDataSource
    ) {
        self.settingsStore = settingsStore
        self.adjunctLoader = adjunctLoader
        self.authCommandRunner = authCommandRunner
        self.providerDataSource = providerDataSource
    }
}

public struct NoopWidgetReloader: WidgetReloading {
    public init() {}

    public func reloadAllTimelines() {}
}
