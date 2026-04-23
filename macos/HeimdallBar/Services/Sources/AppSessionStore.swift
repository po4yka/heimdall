import Foundation
import HeimdallDomain
import Observation

public final class UserDefaultsAppSessionStateStore: @unchecked Sendable, AppSessionStatePersisting {
    private enum Keys {
        static let selectedProvider = "heimdallbar.app_session.selected_provider"
        static let selectedMergeTab = "heimdallbar.app_session.selected_merge_tab"
        static let liveMonitorFocus = "heimdallbar.app_session.live_monitor.focus"
        static let liveMonitorDensity = "heimdallbar.app_session.live_monitor.density"
        static let liveMonitorHiddenPanels = "heimdallbar.app_session.live_monitor.hidden_panels"
    }

    private let defaults: UserDefaults

    public init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
    }

    public func loadAppSessionState() -> PersistedAppSessionState? {
        let hasStoredState =
            self.defaults.object(forKey: Keys.selectedProvider) != nil
            || self.defaults.object(forKey: Keys.selectedMergeTab) != nil
            || self.defaults.object(forKey: Keys.liveMonitorFocus) != nil
            || self.defaults.object(forKey: Keys.liveMonitorDensity) != nil
            || self.defaults.object(forKey: Keys.liveMonitorHiddenPanels) != nil
        guard hasStoredState else {
            return nil
        }

        let provider = self.defaults.string(forKey: Keys.selectedProvider)
            .flatMap(ProviderID.init(rawValue:))
            ?? .claude
        let mergeTab = self.defaults.string(forKey: Keys.selectedMergeTab)
            .flatMap(MergeMenuTab.init(rawValue:))
            ?? .overview
        let liveMonitorPreferences = self.loadLiveMonitorPreferences()
        return PersistedAppSessionState(
            selectedProvider: provider,
            selectedMergeTab: mergeTab,
            liveMonitorPreferences: liveMonitorPreferences
        )
    }

    public func saveAppSessionState(_ state: PersistedAppSessionState) {
        self.defaults.set(state.selectedProvider.rawValue, forKey: Keys.selectedProvider)
        self.defaults.set(state.selectedMergeTab.rawValue, forKey: Keys.selectedMergeTab)
        if let preferences = state.liveMonitorPreferences {
            self.defaults.set(preferences.focus.rawValue, forKey: Keys.liveMonitorFocus)
            self.defaults.set(preferences.density.rawValue, forKey: Keys.liveMonitorDensity)
            self.defaults.set(preferences.hiddenPanels.map(\.rawValue), forKey: Keys.liveMonitorHiddenPanels)
        } else {
            self.defaults.removeObject(forKey: Keys.liveMonitorFocus)
            self.defaults.removeObject(forKey: Keys.liveMonitorDensity)
            self.defaults.removeObject(forKey: Keys.liveMonitorHiddenPanels)
        }
    }

    private func loadLiveMonitorPreferences() -> LiveMonitorPreferences? {
        let hasPreferences =
            self.defaults.object(forKey: Keys.liveMonitorFocus) != nil
            || self.defaults.object(forKey: Keys.liveMonitorDensity) != nil
            || self.defaults.object(forKey: Keys.liveMonitorHiddenPanels) != nil
        guard hasPreferences else {
            return nil
        }

        let focus = self.defaults.string(forKey: Keys.liveMonitorFocus)
            .flatMap(LiveMonitorFocus.init(rawValue:))
            ?? .all
        let density = self.defaults.string(forKey: Keys.liveMonitorDensity)
            .flatMap(LiveMonitorDensity.init(rawValue:))
            ?? .expanded
        let hiddenPanels = (self.defaults.array(forKey: Keys.liveMonitorHiddenPanels) as? [String] ?? [])
            .compactMap(LiveMonitorPanelID.init(rawValue:))

        return LiveMonitorPreferences(
            focus: focus,
            density: density,
            hiddenPanels: Array(Set(hiddenPanels)).sorted { $0.rawValue < $1.rawValue }
        )
    }
}

public final class UserDefaultsCloudSyncStateStore: @unchecked Sendable, CloudSyncStatePersisting {
    private enum Keys {
        static let installationID = "heimdallbar.cloud_sync.installation_id"
        static let cloudSyncState = "heimdallbar.cloud_sync.state"
    }

    private let defaults: UserDefaults
    private let encoder = JSONEncoder()
    private let decoder = JSONDecoder()

    public init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
    }

    public func loadInstallationID() -> String? {
        self.defaults.string(forKey: Keys.installationID)
    }

    public func saveInstallationID(_ installationID: String) {
        self.defaults.set(installationID, forKey: Keys.installationID)
    }

    public func loadCloudSyncSpaceState() -> CloudSyncSpaceState? {
        guard let data = self.defaults.data(forKey: Keys.cloudSyncState) else {
            return nil
        }
        return try? self.decoder.decode(CloudSyncSpaceState.self, from: data)
    }

    public func saveCloudSyncSpaceState(_ state: CloudSyncSpaceState) {
        guard let data = try? self.encoder.encode(state) else { return }
        self.defaults.set(data, forKey: Keys.cloudSyncState)
    }
}

@MainActor
@Observable
public final class AppSessionStore {
    public var config: HeimdallBarConfig
    public var installationID: String {
        didSet {
            self.cloudSyncStatePersistence.saveInstallationID(self.installationID)
        }
    }
    public var cloudSyncState: CloudSyncSpaceState {
        didSet {
            self.cloudSyncStatePersistence.saveCloudSyncSpaceState(self.cloudSyncState)
        }
    }
    public var selectedProvider: ProviderID {
        didSet {
            self.persistSelections()
        }
    }
    public var selectedMergeTab: MergeMenuTab {
        didSet {
            self.persistSelections()
        }
    }
    public var liveMonitorPreferences: LiveMonitorPreferences? {
        didSet {
            self.persistSelections()
        }
    }

    private let persistence: any AppSessionStatePersisting
    private let cloudSyncStatePersistence: any CloudSyncStatePersisting

    public init(
        config: HeimdallBarConfig = .default,
        selectedProvider: ProviderID = .claude,
        selectedMergeTab: MergeMenuTab = .overview,
        liveMonitorPreferences: LiveMonitorPreferences? = nil,
        persistence: any AppSessionStatePersisting = UserDefaultsAppSessionStateStore(),
        cloudSyncStatePersistence: any CloudSyncStatePersisting = UserDefaultsCloudSyncStateStore()
    ) {
        self.config = config
        self.persistence = persistence
        self.cloudSyncStatePersistence = cloudSyncStatePersistence
        let persistedState = persistence.loadAppSessionState()
        self.installationID = cloudSyncStatePersistence.loadInstallationID() ?? UUID().uuidString.lowercased()
        self.cloudSyncState = cloudSyncStatePersistence.loadCloudSyncSpaceState() ?? CloudSyncSpaceState()
        self.selectedProvider = persistedState?.selectedProvider ?? selectedProvider
        self.selectedMergeTab = persistedState?.selectedMergeTab ?? selectedMergeTab
        self.liveMonitorPreferences = persistedState?.liveMonitorPreferences ?? liveMonitorPreferences
        self.cloudSyncStatePersistence.saveInstallationID(self.installationID)
    }

    public var visibleProviders: [ProviderID] {
        ProviderID.allCases.filter { self.config.providerConfig(for: $0).enabled }
    }

    public var visibleTabs: [MergeMenuTab] {
        MenuProjectionBuilder.availableTabs(config: self.config)
    }

    private func persistSelections() {
        self.persistence.saveAppSessionState(
            PersistedAppSessionState(
                selectedProvider: self.selectedProvider,
                selectedMergeTab: self.selectedMergeTab,
                liveMonitorPreferences: self.liveMonitorPreferences
            )
        )
    }
}
