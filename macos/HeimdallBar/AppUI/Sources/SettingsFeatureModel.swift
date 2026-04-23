import Foundation
import HeimdallDomain
import HeimdallServices
import Observation
import os.log

@MainActor
@Observable
public final class SettingsFeatureModel {
    public let sessionStore: AppSessionStore
    public var onSettingsSaved: (() -> Void)?
    public var draftConfig: HeimdallBarConfig

    private let repository: ProviderRepository
    private let settingsStore: any SettingsStore
    private let refreshCoordinator: RefreshCoordinator
    private let localNotificationCoordinator: any LocalNotificationCoordinating

    private static let logger = Logger(subsystem: "dev.heimdall.HeimdallBar", category: "SettingsFeatureModel")

    public init(
        sessionStore: AppSessionStore,
        repository: ProviderRepository,
        settingsStore: any SettingsStore,
        refreshCoordinator: RefreshCoordinator,
        localNotificationCoordinator: any LocalNotificationCoordinating
    ) {
        self.sessionStore = sessionStore
        self.repository = repository
        self.settingsStore = settingsStore
        self.refreshCoordinator = refreshCoordinator
        self.localNotificationCoordinator = localNotificationCoordinator
        self.onSettingsSaved = nil
        self.draftConfig = sessionStore.config
    }

    public var config: HeimdallBarConfig {
        self.sessionStore.config
    }

    public var issue: AppIssue? {
        let candidate = self.repository.issue(for: nil)
        if let candidate, candidate.kind == .widgetPersistence {
            Self.logger.debug("Suppressing widgetPersistence issue from Settings UI: \(candidate.message)")
            return nil
        }
        return candidate
    }

    public func save() async {
        let draftConfig = self.draftConfig
        let previousConfig = self.sessionStore.config
        do {
            try self.settingsStore.save(draftConfig)
            self.sessionStore.config = draftConfig
            self.draftConfig = draftConfig
            self.repository.clearIssue(kind: .settingsSave)
            if previousConfig.localNotificationsEnabled != draftConfig.localNotificationsEnabled {
                if let issue = await self.localNotificationCoordinator.handleConfigChange(
                    previous: previousConfig,
                    current: draftConfig
                ) {
                    self.repository.recordIssue(issue)
                } else {
                    self.repository.clearIssue(kind: .localNotifications)
                }
            }
            self.onSettingsSaved?()
        } catch {
            self.repository.setIssue(
                AppIssue(kind: .settingsSave, message: error.localizedDescription)
            )
        }
    }

    public func resetDraftFromLiveConfig() {
        self.draftConfig = self.sessionStore.config
    }

    public func refreshAll() async {
        await self.refreshCoordinator.refresh(force: true, provider: nil)
    }

    public func refreshBrowserImports() async {
        await self.refreshCoordinator.refreshBrowserImports()
    }
}
