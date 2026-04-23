import HeimdallDomain
import HeimdallServices
import Testing
@testable import HeimdallAppUI

struct AppShellModelTests {
    @MainActor
    @Test
    func navigationItemsIncludeLiveMonitor() {
        let model = AppShellModel(
            sessionStore: AppSessionStore(
                config: .default,
                selectedProvider: .claude,
                selectedMergeTab: .overview,
                persistence: TestAppSessionStateStore()
            )
        )

        #expect(model.navigationItems.contains(AppNavigationItem.liveMonitor))
    }

    @MainActor
    @Test
    func appSessionStoreLoadsPersistedLiveMonitorPreferences() {
        let persistedPreferences = LiveMonitorPreferences(
            focus: .codex,
            density: .compact,
            hiddenPanels: [.contextWindow, .warnings]
        )
        let store = AppSessionStore(
            persistence: TestAppSessionStateStore(
                initialState: PersistedAppSessionState(
                    selectedProvider: .claude,
                    selectedMergeTab: .overview,
                    liveMonitorPreferences: persistedPreferences
                )
            )
        )

        #expect(store.liveMonitorPreferences == persistedPreferences)
    }

    @MainActor
    @Test
    func appSessionStorePersistsLiveMonitorPreferencesAdditively() {
        let persistence = TestAppSessionStateStore()
        let store = AppSessionStore(persistence: persistence)
        let preferences = LiveMonitorPreferences(
            focus: .claude,
            density: .compact,
            hiddenPanels: [.activeBlock, .recentSession]
        )

        store.liveMonitorPreferences = preferences

        #expect(persistence.savedStates.last == PersistedAppSessionState(
            selectedProvider: .claude,
            selectedMergeTab: .overview,
            liveMonitorPreferences: preferences
        ))
    }
}

private final class TestAppSessionStateStore: AppSessionStatePersisting, @unchecked Sendable {
    let initialState: PersistedAppSessionState?
    private(set) var savedStates: [PersistedAppSessionState] = []

    init(initialState: PersistedAppSessionState? = nil) {
        self.initialState = initialState
    }

    func loadAppSessionState() -> PersistedAppSessionState? {
        self.initialState
    }

    func saveAppSessionState(_ state: PersistedAppSessionState) {
        self.savedStates.append(state)
    }
}
