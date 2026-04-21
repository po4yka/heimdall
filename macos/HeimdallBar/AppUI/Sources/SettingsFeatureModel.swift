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
    }

    public var config: HeimdallBarConfig {
        get { self.sessionStore.config }
        set { self.sessionStore.config = newValue }
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
}
