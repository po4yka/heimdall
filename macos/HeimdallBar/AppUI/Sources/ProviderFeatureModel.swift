import Foundation
import HeimdallDomain
import HeimdallServices
import Observation
import os.log

@MainActor
@Observable
public final class ProviderFeatureModel {
    public let provider: ProviderID

    private let sessionStore: AppSessionStore
    private let repository: ProviderRepository
    private let refreshCoordinator: RefreshCoordinator
    private let authCoordinator: AuthCoordinator
    private let credentialInspector: any ProviderCredentialInspecting

    public init(
        provider: ProviderID,
        sessionStore: AppSessionStore,
        repository: ProviderRepository,
        refreshCoordinator: RefreshCoordinator,
        authCoordinator: AuthCoordinator,
        credentialInspector: any ProviderCredentialInspecting
    ) {
        self.provider = provider
        self.sessionStore = sessionStore
        self.repository = repository
        self.refreshCoordinator = refreshCoordinator
        self.authCoordinator = authCoordinator
        self.credentialInspector = credentialInspector
    }

    public var config: ProviderConfig {
        self.sessionStore.config.providerConfig(for: self.provider)
    }

    public var showUsedValues: Bool {
        self.sessionStore.config.showUsedValues
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

    private static let logger = Logger(subsystem: "dev.heimdall.HeimdallBar", category: "ProviderFeatureModel")

    public var issue: AppIssue? {
        let candidate = self.repository.issue(for: self.provider) ?? self.repository.issue(for: nil)
        if let candidate, candidate.kind == .widgetPersistence {
            Self.logger.debug("Suppressing widgetPersistence issue from UI: \(candidate.message)")
            return nil
        }
        return candidate
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

    public var missingCredentialDetail: String? {
        // Prefer the authoritative diagnosticCode computed by the Rust helper,
        // which considers env vars + macOS Keychain + file + config. The file
        // probe is only a last-resort fallback for when the helper hasn't
        // reported yet — with Claude Code v2.x on macOS, credentials live in
        // Keychain by default, so a file-absent check is a false negative.
        if let code = self.authHealth?.diagnosticCode {
            switch code {
            case "authenticated-compatible",
                 "authenticated",
                 "authenticated-incompatible":
                return nil
            case "expired-refreshable":
                return "Token expired — will refresh on next poll"
            case "requires-relogin", "refresh-failed":
                return self.provider == .claude
                    ? "Sign in to Claude (run /login in Claude Code)"
                    : "Sign in to Codex"
            case "missing-credentials":
                return self.provider == .claude ? "Sign in to Claude" : "Sign in to Codex"
            case "keychain-locked":
                return "Keychain access denied — click Refresh and approve the prompt"
            case "keychain-unavailable":
                return "Keychain unavailable — unlock your login keychain and retry"
            case "managed-restricted":
                return self.authHealth?.managedRestriction
                    ?? "Blocked by managed policy"
            default:
                // Unknown diagnostic — show the raw code so we at least hint
                // at something rather than the misleading 'missing file' text.
                return self.authHealth?.failureReason ?? code
            }
        }

        // Fallback: helper hasn't responded yet AND no file on disk. This used
        // to always render "Missing Claude OAuth credentials file" even when
        // the user was logged in via Keychain — now we only surface it when
        // we truly have no signal.
        if self.credentialInspector.credentialPresence(for: self.provider) == .missing {
            return "Waiting for helper to report auth status"
        }
        return nil
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
