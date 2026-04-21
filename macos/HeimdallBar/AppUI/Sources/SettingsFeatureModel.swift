import Foundation
import HeimdallDomain
import HeimdallServices
import Observation

@MainActor
@Observable
public final class SettingsFeatureModel {
    public let sessionStore: AppSessionStore
    public var onSettingsSaved: (() -> Void)?

    private let repository: ProviderRepository
    private let settingsStore: any SettingsStore
    private let refreshCoordinator: RefreshCoordinator

    public init(
        sessionStore: AppSessionStore,
        repository: ProviderRepository,
        settingsStore: any SettingsStore,
        refreshCoordinator: RefreshCoordinator
    ) {
        self.sessionStore = sessionStore
        self.repository = repository
        self.settingsStore = settingsStore
        self.refreshCoordinator = refreshCoordinator
        self.onSettingsSaved = nil
    }

    public var config: HeimdallBarConfig {
        get { self.sessionStore.config }
        set { self.sessionStore.config = newValue }
    }

    public var issue: AppIssue? {
        self.repository.issue(for: nil)
    }

    public func save() {
        do {
            try self.settingsStore.save(self.sessionStore.config)
            self.repository.setIssue(nil)
            self.onSettingsSaved?()
        } catch {
            self.repository.setIssue(
                AppIssue(kind: .settingsSave, message: error.localizedDescription)
            )
        }
    }

    public func refreshAll() async {
        await self.refreshCoordinator.refresh(force: true, provider: nil)
    }

    public func refreshBrowserImports() async {
        await self.refreshCoordinator.refreshBrowserImports()
    }

    public func isClaudeOAuthCredentialsMissing() -> Bool {
        let url = URL(fileURLWithPath: NSHomeDirectory())
            .appendingPathComponent(".claude", isDirectory: true)
            .appendingPathComponent(".credentials.json", isDirectory: false)
        return !FileManager.default.fileExists(atPath: url.path)
    }
}
