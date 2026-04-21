import Foundation
import HeimdallDomain
import HeimdallServices
import Observation

@MainActor
@Observable
public final class AppModel {
    public let sessionStore: AppSessionStore
    public let providerRepository: ProviderRepository

    private let settingsStore: any SettingsStore
    private let refreshCoordinator: RefreshCoordinator
    private let authCoordinator: AuthCoordinator
    private var hasStarted: Bool

    public init(environment: HeimdallAppEnvironment) {
        let sessionStore = AppSessionStore(config: environment.settingsStore.load())
        let providerRepository = ProviderRepository()
        self.sessionStore = sessionStore
        self.providerRepository = providerRepository
        self.settingsStore = environment.settingsStore
        self.refreshCoordinator = RefreshCoordinator(
            sessionStore: sessionStore,
            repository: providerRepository,
            helperRuntime: environment.helperRuntime,
            adjunctProvider: environment.adjunctProvider,
            widgetSnapshotCoordinator: WidgetSnapshotCoordinator(
                writer: environment.widgetSnapshotWriter,
                reloader: environment.widgetReloader
            ),
            liveProviderClientFactory: environment.liveProviderClientFactory
        )
        self.authCoordinator = AuthCoordinator(runner: environment.authCommandRunner)
        self.hasStarted = false
    }

    public var config: HeimdallBarConfig {
        get { self.sessionStore.config }
        set { self.sessionStore.config = newValue }
    }

    public var snapshots: [ProviderSnapshot] {
        self.providerRepository.snapshots
    }

    public var selectedProvider: ProviderID {
        get { self.sessionStore.selectedProvider }
        set { self.sessionStore.selectedProvider = newValue }
    }

    public var selectedMergeTab: MergeMenuTab {
        get { self.sessionStore.selectedMergeTab }
        set { self.sessionStore.selectedMergeTab = newValue }
    }

    public var adjunctSnapshots: [ProviderID: DashboardAdjunctSnapshot] {
        self.providerRepository.adjunctSnapshots
    }

    public var importedSessions: [ProviderID: ImportedBrowserSession] {
        self.providerRepository.importedSessions
    }

    public var browserImportCandidates: [ProviderID: [BrowserSessionImportCandidate]] {
        self.providerRepository.browserImportCandidates
    }

    public var lastError: String? {
        self.providerRepository.lastError
    }

    public var isRefreshing: Bool {
        self.providerRepository.isRefreshing
    }

    public var refreshingProvider: ProviderID? {
        self.providerRepository.refreshingProvider
    }

    public var lastRefreshCompletedAt: Date? {
        self.providerRepository.lastRefreshCompletedAt
    }

    public var isImportingSession: Bool {
        self.providerRepository.isImportingSession
    }

    public var visibleProviders: [ProviderID] {
        self.sessionStore.visibleProviders
    }

    public var visibleTabs: [MergeMenuTab] {
        self.sessionStore.visibleTabs
    }

    public func start() {
        guard !self.hasStarted else { return }
        self.hasStarted = true
        self.refreshCoordinator.start()
    }

    public func prepareForExit() async {
        await self.refreshCoordinator.stop()
    }

    public func refresh(force: Bool, provider: ProviderID? = nil) async {
        await self.refreshCoordinator.refresh(force: force, provider: provider)
    }

    public func saveConfig() {
        do {
            try self.settingsStore.save(self.sessionStore.config)
            self.providerRepository.syncSelections(sessionStore: self.sessionStore)
            Task { @MainActor [weak self] in
                guard let self else { return }
                await self.refreshCoordinator.refreshBrowserImports()
            }
        } catch {
            self.providerRepository.lastError = error.localizedDescription
        }
    }

    public func importedSession(for provider: ProviderID) -> ImportedBrowserSession? {
        self.providerRepository.importedSessions[provider]
    }

    public func importCandidates(for provider: ProviderID) -> [BrowserSessionImportCandidate] {
        self.providerRepository.browserImportCandidates[provider] ?? []
    }

    public func refreshBrowserImports() async {
        await self.refreshCoordinator.refreshBrowserImports()
    }

    public func importBrowserSession(
        provider: ProviderID,
        candidate: BrowserSessionImportCandidate
    ) async {
        await self.refreshCoordinator.importBrowserSession(provider: provider, candidate: candidate)
    }

    public func resetBrowserSession(provider: ProviderID) async {
        await self.refreshCoordinator.resetBrowserSession(provider: provider)
    }

    public func oauthQuickFixButtonTitle(for provider: ProviderID) -> String {
        self.primaryAuthAction(for: provider)?.label ?? "Fix Auth"
    }

    public func oauthQuickFixHint(for provider: ProviderID) -> String? {
        self.authDetail(for: provider)
    }

    public func isClaudeOAuthCredentialsMissing() -> Bool {
        !FileManager.default.fileExists(atPath: Self.claudeCredentialsURL.path())
    }

    public func runOAuthQuickFix(for provider: ProviderID) async {
        guard let action = self.primaryAuthAction(for: provider) else {
            self.providerRepository.lastError = "No auth recovery action is available for \(provider.title)."
            return
        }
        await self.runAuthRecoveryAction(action, for: provider)
    }

    public func authHealth(for provider: ProviderID) -> ProviderAuthHealth? {
        self.snapshot(for: provider)?.auth
    }

    public func authHeadline(for provider: ProviderID) -> String? {
        self.projection(for: provider).authHeadline
    }

    public func authDetail(for provider: ProviderID) -> String? {
        self.projection(for: provider).authDetail
    }

    public func authRecoveryActions(for provider: ProviderID) -> [AuthRecoveryAction] {
        self.authCoordinator.recoveryActions(for: provider, projection: self.projection(for: provider))
    }

    public func primaryAuthAction(for provider: ProviderID) -> AuthRecoveryAction? {
        self.authCoordinator.primaryAction(for: provider, projection: self.projection(for: provider))
    }

    public func runAuthRecoveryAction(_ action: AuthRecoveryAction, for provider: ProviderID) async {
        do {
            if let detail = action.detail, action.command == nil {
                self.providerRepository.lastError = detail
                return
            }
            try self.authCoordinator.run(action, provider: provider)
            self.providerRepository.lastError = nil
        } catch {
            self.providerRepository.lastError = "Failed to start \(provider.title) auth recovery. Run `\(self.authCoordinator.defaultCommand(for: action, provider: provider))` manually."
        }
    }

    public func snapshot(for provider: ProviderID) -> ProviderSnapshot? {
        self.providerRepository.snapshot(for: provider)
    }

    public func presentation(for provider: ProviderID) -> ProviderPresentationState {
        self.providerRepository.presentation(for: provider, sessionStore: self.sessionStore)
    }

    public func menuTitle(for provider: ProviderID?) -> String {
        let presentation = provider.map(self.presentation(for:))
            ?? self.visibleProviders.map(self.presentation(for:)).first
        return MenuProjectionBuilder.menuTitle(for: presentation, provider: provider, config: self.sessionStore.config)
    }

    public func projection(for provider: ProviderID) -> ProviderMenuProjection {
        MenuProjectionBuilder.projection(
            from: self.presentation(for: provider),
            config: self.sessionStore.config,
            isRefreshing: self.isRefreshing && (self.refreshingProvider == nil || self.refreshingProvider == provider),
            lastGlobalError: self.lastError
        )
    }

    public func overviewProjection() -> OverviewMenuProjection {
        MenuProjectionBuilder.overview(
            from: self.visibleProviders.map(self.projection(for:)),
            isRefreshing: self.isRefreshing,
            lastGlobalError: self.lastError
        )
    }

    public func refreshActionLabel(for tab: MergeMenuTab) -> String {
        if let provider = tab.providerID {
            return "Refresh \(provider.title)"
        }
        return "Refresh All"
    }

    public func makeWidgetSnapshot() -> WidgetSnapshot {
        WidgetSnapshotBuilder.snapshot(
            providers: self.visibleProviders,
            snapshots: self.providerRepository.snapshotsByProvider,
            adjuncts: self.providerRepository.adjunctSnapshots,
            config: self.sessionStore.config,
            generatedAt: ISO8601DateFormatter().string(from: Date())
        )
    }

    private static var claudeCredentialsURL: URL {
        URL(fileURLWithPath: NSHomeDirectory())
            .appendingPathComponent(".claude", isDirectory: true)
            .appendingPathComponent(".credentials.json", isDirectory: false)
    }
}
