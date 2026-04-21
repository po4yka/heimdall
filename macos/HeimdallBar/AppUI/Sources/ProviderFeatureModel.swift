import Foundation
import HeimdallDomain
import HeimdallServices
import Observation

@MainActor
@Observable
public final class ProviderFeatureModel {
    public let provider: ProviderID

    private let sessionStore: AppSessionStore
    private let repository: ProviderRepository
    private let refreshCoordinator: RefreshCoordinator
    private let authCoordinator: AuthCoordinator

    public init(
        provider: ProviderID,
        sessionStore: AppSessionStore,
        repository: ProviderRepository,
        refreshCoordinator: RefreshCoordinator,
        authCoordinator: AuthCoordinator
    ) {
        self.provider = provider
        self.sessionStore = sessionStore
        self.repository = repository
        self.refreshCoordinator = refreshCoordinator
        self.authCoordinator = authCoordinator
    }

    public var config: ProviderConfig {
        self.sessionStore.config.providerConfig(for: self.provider)
    }

    public var snapshot: ProviderSnapshot? {
        self.repository.snapshot(for: self.provider)
    }

    public var importedSession: ImportedBrowserSession? {
        self.repository.importedSessions[self.provider]
    }

    public var importCandidates: [BrowserSessionImportCandidate] {
        self.repository.browserImportCandidates[self.provider] ?? []
    }

    public var issue: AppIssue? {
        self.repository.issue(for: self.provider) ?? self.repository.issue(for: nil)
    }

    public var isBusy: Bool {
        self.isRefreshing || self.isImportingSession
    }

    public var isRefreshing: Bool {
        self.repository.refreshActivity == .refreshingAll || self.repository.refreshActivity == .refreshingProvider(self.provider)
    }

    public var isImportingSession: Bool {
        self.repository.sessionImportActivity.provider == self.provider
    }

    public var presentation: ProviderPresentationState {
        self.repository.presentation(for: self.provider, sessionStore: self.sessionStore)
    }

    public var projection: ProviderMenuProjection {
        MenuProjectionBuilder.projection(
            from: self.presentation,
            config: self.sessionStore.config,
            isRefreshing: self.isRefreshing,
            lastGlobalError: self.issue?.message
        )
    }

    public var menuTitle: String {
        MenuProjectionBuilder.menuTitle(for: self.presentation, provider: self.provider, config: self.sessionStore.config)
    }

    public var authHealth: ProviderAuthHealth? {
        self.snapshot?.auth
    }

    public var authRecoveryActions: [AuthRecoveryAction] {
        self.authCoordinator.recoveryActions(for: self.provider, projection: self.projection)
    }

    public func isClaudeOAuthCredentialsMissing() -> Bool {
        let url = URL(fileURLWithPath: NSHomeDirectory())
            .appendingPathComponent(".claude", isDirectory: true)
            .appendingPathComponent(".credentials.json", isDirectory: false)
        return !FileManager.default.fileExists(atPath: url.path)
    }

    public func refresh() async {
        await self.refreshCoordinator.refresh(force: true, provider: self.provider)
    }

    public func importBrowserSession(candidate: BrowserSessionImportCandidate) async {
        await self.refreshCoordinator.importBrowserSession(provider: self.provider, candidate: candidate)
    }

    public func resetBrowserSession() async {
        await self.refreshCoordinator.resetBrowserSession(provider: self.provider)
    }

    public func runAuthRecoveryAction(_ action: AuthRecoveryAction) async {
        do {
            if let detail = action.detail, action.command == nil {
                self.repository.setIssue(
                    AppIssue(kind: .authRecovery, provider: self.provider, message: detail)
                )
                return
            }
            try self.authCoordinator.run(action, provider: self.provider)
            self.repository.setIssue(nil)
        } catch {
            self.repository.setIssue(
                AppIssue(
                    kind: .authRecovery,
                    provider: self.provider,
                    message: "Failed to start \(self.provider.title) auth recovery. Run `\(self.authCoordinator.defaultCommand(for: action, provider: self.provider))` manually."
                )
            )
        }
    }
}
