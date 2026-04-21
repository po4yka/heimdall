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

    private static let logger = Logger(subsystem: "dev.heimdall.HeimdallBar", category: "SettingsFeatureModel")

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

    public func save() {
        let draftConfig = self.draftConfig
        do {
            try self.settingsStore.save(draftConfig)
            self.sessionStore.config = draftConfig
            self.draftConfig = draftConfig
            self.repository.setIssue(nil)
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
