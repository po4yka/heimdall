import Foundation
import HeimdallDomain
import HeimdallServices
import Observation
import AppKit
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
    private let cloudSyncController: (any CloudSyncControlling)?

    private static let logger = Logger(subsystem: "dev.heimdall.HeimdallBar", category: "SettingsFeatureModel")

    public init(
        sessionStore: AppSessionStore,
        repository: ProviderRepository,
        settingsStore: any SettingsStore,
        refreshCoordinator: RefreshCoordinator,
        localNotificationCoordinator: any LocalNotificationCoordinating,
        cloudSyncController: (any CloudSyncControlling)? = nil
    ) {
        self.sessionStore = sessionStore
        self.repository = repository
        self.settingsStore = settingsStore
        self.refreshCoordinator = refreshCoordinator
        self.localNotificationCoordinator = localNotificationCoordinator
        self.cloudSyncController = cloudSyncController
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
        let stateSize: Int? = {
            guard let url = try? FileBackedCloudKitSyncEngineStateStore.defaultURL() else {
                return nil
            }
            let attrs = try? FileManager.default.attributesOfItem(atPath: url.path)
            return (attrs?[.size] as? NSNumber)?.intValue
        }()
        return CloudSyncDiagnostics(
            containerIdentifier: CloudKitSnapshotSyncStore.defaultContainerIdentifier,
            zoneName: self.cloudSyncState.zoneName ?? SnapshotCloudEngine.zoneName,
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
