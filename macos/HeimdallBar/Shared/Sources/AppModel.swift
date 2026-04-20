import Foundation
import Observation
#if canImport(WidgetKit)
import WidgetKit
#endif

@MainActor
@Observable
public final class AppModel {
    public var config: HeimdallBarConfig
    public var snapshots: [ProviderSnapshot]
    public var selectedProvider: ProviderID
    public var selectedMergeTab: MergeMenuTab
    public var adjunctSnapshots: [ProviderID: DashboardAdjunctSnapshot]
    public var importedSessions: [ProviderID: ImportedBrowserSession]
    public var browserImportCandidates: [ProviderID: [BrowserSessionImportCandidate]]
    public var lastError: String?
    public var isRefreshing: Bool
    public var refreshingProvider: ProviderID?
    public var lastRefreshCompletedAt: Date?
    public var isImportingSession: Bool

    private let configStore: ConfigStore
    private let helperController: HeimdallHelperController
    private let dashboardAdjunctController: DashboardAdjunctController
    private var hasStarted: Bool

    public init(
        configStore: ConfigStore = .shared,
        helperController: HeimdallHelperController = HeimdallHelperController(),
        dashboardAdjunctController: DashboardAdjunctController = DashboardAdjunctController()
    ) {
        self.configStore = configStore
        self.helperController = helperController
        self.dashboardAdjunctController = dashboardAdjunctController
        self.config = configStore.load()
        self.snapshots = []
        self.selectedProvider = .claude
        self.selectedMergeTab = .overview
        self.adjunctSnapshots = [:]
        self.importedSessions = [:]
        self.browserImportCandidates = [:]
        self.lastError = nil
        self.isRefreshing = false
        self.refreshingProvider = nil
        self.lastRefreshCompletedAt = nil
        self.isImportingSession = false
        self.hasStarted = false
    }

    public var visibleProviders: [ProviderID] {
        ProviderID.allCases.filter { self.config.providerConfig(for: $0).enabled }
    }

    public var visibleTabs: [MergeMenuTab] {
        MenuProjectionBuilder.availableTabs(config: self.config)
    }

    public func start() {
        guard !self.hasStarted else { return }
        self.hasStarted = true
        Task { @MainActor [weak self] in
            guard let self else { return }
            await self.refresh(force: false, provider: nil)
            self.startRefreshLoop()
        }
    }

    public func prepareForExit() async {
        await self.helperController.stopOwnedHelper()
    }

    public func refresh(force: Bool, provider: ProviderID? = nil) async {
        self.isRefreshing = true
        self.refreshingProvider = provider
        defer { self.isRefreshing = false }
        let helperReady = await self.helperController.ensureServerRunning(port: self.config.helperPort)
        guard helperReady else {
            self.lastError = "The local Heimdall server is still starting."
            self.lastRefreshCompletedAt = Date()
            self.refreshingProvider = nil
            return
        }
        let client = HeimdallAPIClient(port: self.config.helperPort)
        do {
            let envelope: ProviderSnapshotEnvelope
            if force {
                envelope = try await client.refresh(provider: provider)
            } else {
                envelope = try await client.fetchSnapshots()
            }
            self.apply(envelope.providers, replacing: provider == nil)
            await self.loadAdjuncts(for: provider.map { [$0] } ?? self.visibleProviders, forceRefresh: force)
            await self.loadImportedSessions(for: provider.map { [$0] } ?? ProviderID.allCases)
            self.syncSelections()
            self.lastError = nil
            self.lastRefreshCompletedAt = Date()
            do {
                let saveResult = try WidgetSnapshotStore.save(self.makeWidgetSnapshot())
                #if canImport(WidgetKit)
                if saveResult == .saved {
                    WidgetCenter.shared.reloadAllTimelines()
                }
                #endif
            } catch {
                self.lastError = error.localizedDescription
            }
        } catch {
            self.lastError = error.localizedDescription
            self.lastRefreshCompletedAt = Date()
        }
        self.refreshingProvider = nil
    }

    public func saveConfig() {
        do {
            try self.configStore.save(self.config)
            self.syncSelections()
            Task { @MainActor [weak self] in
                guard let self else { return }
                await self.loadAdjuncts(for: self.visibleProviders, forceRefresh: false)
                await self.loadImportedSessions(for: ProviderID.allCases)
            }
        } catch {
            self.lastError = error.localizedDescription
        }
    }

    public func importedSession(for provider: ProviderID) -> ImportedBrowserSession? {
        self.importedSessions[provider]
    }

    public func importCandidates(for provider: ProviderID) -> [BrowserSessionImportCandidate] {
        self.browserImportCandidates[provider] ?? []
    }

    public func refreshBrowserImports() async {
        await self.loadImportedSessions(for: ProviderID.allCases)
        await self.loadAdjuncts(for: self.visibleProviders, forceRefresh: false)
    }

    public func importBrowserSession(
        provider: ProviderID,
        candidate: BrowserSessionImportCandidate
    ) async {
        self.isImportingSession = true
        defer { self.isImportingSession = false }

        do {
            let session = try await self.dashboardAdjunctController.importBrowserSession(provider: provider, candidate: candidate)
            self.importedSessions[provider] = session
            await self.loadAdjuncts(for: [provider], forceRefresh: true)
            self.lastError = nil
        } catch {
            self.lastError = error.localizedDescription
        }
    }

    public func resetBrowserSession(provider: ProviderID) async {
        self.isImportingSession = true
        defer { self.isImportingSession = false }

        do {
            try await self.dashboardAdjunctController.resetImportedSession(provider: provider)
            self.importedSessions.removeValue(forKey: provider)
            await self.loadAdjuncts(for: [provider], forceRefresh: true)
            self.lastError = nil
        } catch {
            self.lastError = error.localizedDescription
        }
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
            self.lastError = "No auth recovery action is available for \(provider.title)."
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
        let actions = self.projection(for: provider).authRecoveryActions
        if !actions.isEmpty {
            return actions
        }
        return self.defaultAuthRecoveryActions(for: provider)
    }

    public func primaryAuthAction(for provider: ProviderID) -> AuthRecoveryAction? {
        self.authRecoveryActions(for: provider).first
    }

    public func runAuthRecoveryAction(_ action: AuthRecoveryAction, for provider: ProviderID) async {
        do {
            guard let launch = Self.recoveryLaunch(for: action.actionID, provider: provider) else {
                if let detail = action.detail {
                    self.lastError = detail
                } else {
                    self.lastError = "Unsupported \(provider.title) auth recovery action."
                }
                return
            }
            if let detail = action.detail, action.command == nil, launch.command.isEmpty {
                self.lastError = detail
                return
            }
            try Self.launchAuthCommand(
                provider: provider,
                title: launch.title,
                command: launch.command
            )
            self.lastError = nil
        } catch {
            self.lastError = "Failed to start \(provider.title) auth recovery. Run `\(Self.recoveryLaunch(for: action.actionID, provider: provider)?.command ?? Self.defaultCommand(for: action, provider: provider))` manually."
        }
    }

    public func snapshot(for provider: ProviderID) -> ProviderSnapshot? {
        self.snapshots.first(where: { $0.providerID == provider })
    }

    public func presentation(for provider: ProviderID) -> ProviderPresentationState {
        SourceResolver.presentation(
            for: provider,
            config: self.config.providerConfig(for: provider),
            snapshot: self.snapshot(for: provider),
            adjunct: self.adjunctSnapshots[provider]
        )
    }

    public func menuTitle(for provider: ProviderID?) -> String {
        let presentation = provider.map(self.presentation(for:))
            ?? self.visibleProviders.map(self.presentation(for:)).first
        return MenuProjectionBuilder.menuTitle(for: presentation, provider: provider, config: self.config)
    }

    public func projection(for provider: ProviderID) -> ProviderMenuProjection {
        MenuProjectionBuilder.projection(
            from: self.presentation(for: provider),
            config: self.config,
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
        let snapshotsByProvider = Dictionary(uniqueKeysWithValues: self.snapshots.compactMap { snapshot in
            snapshot.providerID.map { ($0, snapshot) }
        })
        let adjunctsByProvider = self.adjunctSnapshots

        return WidgetSnapshotBuilder.snapshot(
            providers: self.visibleProviders,
            snapshots: snapshotsByProvider,
            adjuncts: adjunctsByProvider,
            config: self.config,
            generatedAt: ISO8601DateFormatter().string(from: Date())
        )
    }

    private func startRefreshLoop() {
        Task { @MainActor [weak self] in
            while let self {
                let refreshIntervalSeconds = self.config.refreshIntervalSeconds
                try? await Task.sleep(for: .seconds(refreshIntervalSeconds))
                await self.refresh(force: false, provider: nil)
            }
        }
    }

    private func apply(_ incoming: [ProviderSnapshot], replacing: Bool) {
        if replacing {
            self.snapshots = incoming
            return
        }

        var merged = Dictionary(uniqueKeysWithValues: self.snapshots.compactMap { snapshot in
            snapshot.providerID.map { ($0, snapshot) }
        })
        for snapshot in incoming {
            if let provider = snapshot.providerID {
                merged[provider] = snapshot
            }
        }
        self.snapshots = ProviderID.allCases.compactMap { merged[$0] }
    }

    private func loadAdjuncts(for providers: [ProviderID], forceRefresh: Bool) async {
        for provider in providers {
            let providerConfig = self.config.providerConfig(for: provider)
            let adjunct = await self.dashboardAdjunctController.loadAdjunct(
                provider: provider,
                config: providerConfig,
                snapshot: self.snapshot(for: provider),
                forceRefresh: forceRefresh
            )
            self.adjunctSnapshots[provider] = adjunct
        }
    }

    private func loadImportedSessions(for providers: [ProviderID]) async {
        for provider in providers {
            self.importedSessions[provider] = await self.dashboardAdjunctController.importedSession(provider: provider)
            self.browserImportCandidates[provider] = await self.dashboardAdjunctController.discoverImportCandidates(provider: provider)
        }
    }

    private func syncSelections() {
        if !self.visibleProviders.contains(self.selectedProvider) {
            self.selectedProvider = self.visibleProviders.first ?? .claude
        }
        if !self.visibleTabs.contains(self.selectedMergeTab) {
            self.selectedMergeTab = self.visibleTabs.first ?? .overview
        }
    }

    private static var claudeCredentialsURL: URL {
        URL(fileURLWithPath: NSHomeDirectory())
            .appendingPathComponent(".claude", isDirectory: true)
            .appendingPathComponent(".credentials.json", isDirectory: false)
    }

    private func defaultAuthRecoveryActions(for provider: ProviderID) -> [AuthRecoveryAction] {
        switch provider {
        case .claude:
            return [
                AuthRecoveryAction(
                    label: "Run Claude Login",
                    actionID: "claude-login",
                    command: "claude login",
                    detail: "Run Claude login to restore desktop subscription OAuth."
                ),
                AuthRecoveryAction(
                    label: "Run Claude Doctor",
                    actionID: "claude-doctor",
                    command: "claude doctor",
                    detail: "Use Claude doctor to diagnose credential, keychain, and environment problems."
                ),
            ]
        case .codex:
            return [
                AuthRecoveryAction(
                    label: "Run Codex Login",
                    actionID: "codex-login",
                    command: "codex login",
                    detail: "Run Codex login to restore ChatGPT-backed auth."
                ),
                AuthRecoveryAction(
                    label: "Run Device Login",
                    actionID: "codex-login-device",
                    command: "codex login --device-auth",
                    detail: "Use device auth when localhost callback login is blocked or headless."
                ),
            ]
        }
    }

    private static func defaultCommand(for action: AuthRecoveryAction, provider: ProviderID) -> String {
        if let launch = self.recoveryLaunch(for: action.actionID, provider: provider) {
            return launch.command
        }
        switch (provider, action.actionID) {
        case (.claude, "claude-doctor"):
            return "claude doctor"
        case (.claude, _):
            return "claude login"
        case (.codex, "codex-login-device"):
            return "codex login --device-auth"
        case (.codex, _):
            return "codex login"
        }
    }

    private struct RecoveryLaunch {
        let title: String
        let command: String
    }

    private static func recoveryLaunch(for actionID: String, provider: ProviderID) -> RecoveryLaunch? {
        switch (provider, actionID) {
        case (.claude, "claude-run"):
            return RecoveryLaunch(title: "Run Claude", command: "claude")
        case (.claude, "claude-login"):
            return RecoveryLaunch(title: "Run Claude Login", command: "claude login")
        case (.claude, "claude-doctor"):
            return RecoveryLaunch(title: "Run Claude Doctor", command: "claude doctor")
        case (.codex, "codex-login"):
            return RecoveryLaunch(title: "Run Codex Login", command: "codex login")
        case (.codex, "codex-login-device"):
            return RecoveryLaunch(title: "Run Device Login", command: "codex login --device-auth")
        default:
            return nil
        }
    }

    private static func launchAuthCommand(
        provider: ProviderID,
        title: String,
        command: String
    ) throws {
        let scriptURL = FileManager.default.temporaryDirectory
            .appendingPathComponent("heimdallbar-\(provider.rawValue)-auth.command", isDirectory: false)
        let script = """
        #!/bin/zsh
        export PATH="$HOME/.local/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:$PATH"
        clear
        echo "HeimdallBar \(provider.title) Auth Recovery"
        echo
        if ! command -v \(provider == .claude ? "claude" : "codex") >/dev/null 2>&1; then
          echo "\(provider.title) CLI was not found in PATH."
          echo "Run '\(command)' manually in a shell where the \(provider == .claude ? "claude" : "codex") command exists."
          echo
          read -k '?Press any key to close...'
          exit 1
        fi
        echo "\(title)"
        echo
        echo "Running: \(command)"
        echo
        \(command)
        echo
        if [ "\(provider.rawValue)" = "claude" ]; then
          if [ -f "$HOME/.claude/.credentials.json" ]; then
            echo "Claude OAuth credentials were saved to ~/.claude/.credentials.json."
          else
            echo "Claude OAuth credentials file is still missing."
          fi
        else
          if [ -f "${CODEX_HOME:-$HOME/.codex}/auth.json" ]; then
            echo "Codex auth file is present at ${CODEX_HOME:-$HOME/.codex}/auth.json."
          else
            echo "Codex auth file is still missing."
          fi
        fi
        echo "Return to HeimdallBar and refresh \(provider.title)."
        echo
        read -k '?Press any key to close...'
        """
        try script.write(to: scriptURL, atomically: true, encoding: .utf8)
        try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: scriptURL.path())
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/open")
        process.arguments = ["-a", "Terminal", scriptURL.path()]
        try process.run()
    }
}
