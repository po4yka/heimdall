import Foundation
import HeimdallDomain
import HeimdallServices
import Observation
import AppKit

public enum SettingsSaveStatus: Equatable, Sendable {
    case idle
    case saving
    case saved
    case error(String)
}

@MainActor
@Observable
public final class SettingsFeatureModel {
    public let sessionStore: AppSessionStore
    public var onSettingsSaved: (() -> Void)?
    public var draftConfig: HeimdallConfig
    public var saveStatus: SettingsSaveStatus = .idle

    private var saveStatusClearTask: Task<Void, Never>?

    private let repository: ProviderRepository
    private let settingsStore: any SettingsStore
    private let refreshCoordinator: RefreshCoordinator
    private let localNotificationCoordinator: any LocalNotificationCoordinating
    private let cloudSyncController: (any CloudSyncControlling)?
    private let cloudSyncDiagnosticsContext: CloudSyncDiagnosticsContext?

    public init(
        sessionStore: AppSessionStore,
        repository: ProviderRepository,
        settingsStore: any SettingsStore,
        refreshCoordinator: RefreshCoordinator,
        localNotificationCoordinator: any LocalNotificationCoordinating,
        cloudSyncController: (any CloudSyncControlling)? = nil,
        cloudSyncDiagnosticsContext: CloudSyncDiagnosticsContext? = nil
    ) {
        self.sessionStore = sessionStore
        self.repository = repository
        self.settingsStore = settingsStore
        self.refreshCoordinator = refreshCoordinator
        self.localNotificationCoordinator = localNotificationCoordinator
        self.cloudSyncController = cloudSyncController
        self.cloudSyncDiagnosticsContext = cloudSyncDiagnosticsContext
        self.onSettingsSaved = nil
        self.draftConfig = sessionStore.config
    }

    public var config: HeimdallConfig {
        self.sessionStore.config
    }

    public var issue: AppIssue? {
        let candidate = self.repository.issue(for: nil)
        if let candidate, candidate.kind == .widgetPersistence {
            return nil
        }
        return candidate
    }

    public var cloudSyncState: CloudSyncSpaceState {
        self.repository.cloudSyncState
    }

    public var cloudSyncAggregate: SyncedAggregateEnvelope? {
        self.repository.syncedAggregate
    }

    public var cloudSyncStatusLine: String {
        switch self.cloudSyncState.status {
        case .notConfigured:
            return "Cloud Sync is not configured on this Mac."
        case .ownerReady:
            return "This Mac owns the sync space."
        case .inviteReady:
            return "This Mac owns the sync space and has a share link ready."
        case .participantJoined:
            return "This Mac is joined to a shared sync space."
        case .iCloudUnavailable:
            return self.cloudSyncState.statusMessage ?? "Sign in to iCloud to enable Cloud Sync."
        case .sharingBlocked:
            return self.cloudSyncState.statusMessage ?? "CloudKit sharing is restricted on this device."
        }
    }

    public func save() async {
        let draftConfig = self.draftConfig
        let previousConfig = self.sessionStore.config
        self.saveStatusClearTask?.cancel()
        self.saveStatus = .saving
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
            self.saveStatus = .saved
            self.scheduleSaveStatusClear(after: .seconds(2.5))
        } catch {
            let message = error.localizedDescription
            self.repository.setIssue(
                AppIssue(kind: .settingsSave, message: message)
            )
            self.saveStatus = .error(message)
            self.scheduleSaveStatusClear(after: .seconds(6))
        }
    }

    private func scheduleSaveStatusClear(after duration: Duration) {
        self.saveStatusClearTask?.cancel()
        self.saveStatusClearTask = Task { @MainActor [weak self] in
            try? await Task.sleep(for: duration)
            guard let self, !Task.isCancelled else { return }
            self.saveStatus = .idle
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

    public func refreshCloudSyncState() async {
        guard let cloudSyncController else { return }
        do {
            let state = try await cloudSyncController.loadCloudSyncSpaceState()
            self.repository.setCloudSyncState(state)
            self.sessionStore.cloudSyncState = state
            self.repository.setSyncedAggregate(try await cloudSyncController.loadAggregateSnapshot())
            self.repository.clearIssue(kind: .snapshotSync)
        } catch {
            self.repository.recordIssue(AppIssue(kind: .snapshotSync, message: error.localizedDescription))
        }
    }

    public func prepareCloudShare() async {
        guard let cloudSyncController else { return }
        do {
            let state = try await cloudSyncController.prepareOwnerShare()
            self.repository.setCloudSyncState(state)
            self.sessionStore.cloudSyncState = state
            if let shareURL = state.shareURL {
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(shareURL, forType: .string)
            }
            self.repository.clearIssue(kind: .snapshotSync)
        } catch {
            self.repository.recordIssue(AppIssue(kind: .snapshotSync, message: error.localizedDescription))
        }
    }

    public var cloudSyncDiagnostics: CloudSyncDiagnostics {
        let stateSize = self.cloudSyncDiagnosticsContext?.stateFileURL.flatMap { url in
            let attrs = try? FileManager.default.attributesOfItem(atPath: url.path)
            return (attrs?[.size] as? NSNumber)?.intValue
        }
        return CloudSyncDiagnostics(
            containerIdentifier: self.cloudSyncDiagnosticsContext?.containerIdentifier ?? "Unavailable",
            zoneName: self.cloudSyncState.zoneName ?? self.cloudSyncDiagnosticsContext?.defaultZoneName ?? "_defaultZone",
            zoneOwner: self.cloudSyncState.zoneOwnerName ?? "_defaultOwner",
            installationID: self.sessionStore.installationID,
            engineStateFileBytes: stateSize,
            lastPublishedAt: self.cloudSyncState.lastPublishedAt,
            lastAcceptedAt: self.cloudSyncState.lastAcceptedAt,
            role: self.cloudSyncState.role,
            status: self.cloudSyncState.status
        )
    }
}

public struct CloudSyncDiagnostics: Sendable {
    public var containerIdentifier: String
    public var zoneName: String
    public var zoneOwner: String
    public var installationID: String
    public var engineStateFileBytes: Int?
    public var lastPublishedAt: String?
    public var lastAcceptedAt: String?
    public var role: CloudSyncRole
    public var status: CloudSyncStatus

    public var truncatedInstallationID: String {
        guard self.installationID.count > 12 else { return self.installationID }
        let prefix = self.installationID.prefix(8)
        let suffix = self.installationID.suffix(4)
        return "\(prefix)…\(suffix)"
    }
}
