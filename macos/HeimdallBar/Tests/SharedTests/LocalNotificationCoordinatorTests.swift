import Foundation
import Testing
import HeimdallDomain
import HeimdallServices
@testable import HeimdallAppUI

struct LocalNotificationCoordinatorTests {
    @Test
    func edgeTriggeredConditionsDebounceAndRearm() async {
        let auth = NotificationAuthorizationSpy(status: .authorized, granted: true)
        let scheduler = NotificationSchedulerSpy()
        let stateStore = InMemoryLocalNotificationStateStore()
        let coordinator = LocalNotificationCoordinator(
            authorizationManager: auth,
            scheduler: scheduler,
            stateStore: stateStore
        )
        var config = HeimdallBarConfig.default
        config.localNotificationsEnabled = true

        _ = await coordinator.process(
            envelope: Self.envelope(
                generatedAt: "2026-04-23T10:00:00Z",
                conditions: [
                    Self.sessionCondition(isActive: true)
                ]
            ),
            config: config
        )
        _ = await coordinator.process(
            envelope: Self.envelope(
                generatedAt: "2026-04-23T10:01:00Z",
                conditions: [
                    Self.sessionCondition(isActive: true)
                ]
            ),
            config: config
        )
        _ = await coordinator.process(
            envelope: Self.envelope(
                generatedAt: "2026-04-23T10:02:00Z",
                conditions: [
                    Self.sessionCondition(isActive: false)
                ]
            ),
            config: config
        )
        _ = await coordinator.process(
            envelope: Self.envelope(
                generatedAt: "2026-04-23T10:03:00Z",
                conditions: [
                    Self.sessionCondition(isActive: true)
                ]
            ),
            config: config
        )

        let requests = await scheduler.requests()
        #expect(requests.map(\.title) == [
            "Claude session depleted",
            "Claude session restored",
            "Claude session depleted",
        ])
    }

    @Test
    func dailyCostThresholdPersistsDayKeysAcrossCoordinatorReloads() async {
        let auth = NotificationAuthorizationSpy(status: .authorized, granted: true)
        let scheduler = NotificationSchedulerSpy()
        let stateStore = InMemoryLocalNotificationStateStore()
        var config = HeimdallBarConfig.default
        config.localNotificationsEnabled = true

        let firstCoordinator = LocalNotificationCoordinator(
            authorizationManager: auth,
            scheduler: scheduler,
            stateStore: stateStore
        )
        _ = await firstCoordinator.process(
            envelope: Self.envelope(
                generatedAt: "2026-04-23T11:00:00Z",
                costThresholdUSD: 25,
                conditions: [
                    Self.dailyCostCondition(dayKey: "2026-04-23", isActive: true)
                ]
            ),
            config: config
        )

        let secondCoordinator = LocalNotificationCoordinator(
            authorizationManager: auth,
            scheduler: scheduler,
            stateStore: stateStore
        )
        _ = await secondCoordinator.process(
            envelope: Self.envelope(
                generatedAt: "2026-04-23T12:00:00Z",
                costThresholdUSD: 25,
                conditions: [
                    Self.dailyCostCondition(dayKey: "2026-04-23", isActive: true)
                ]
            ),
            config: config
        )
        _ = await secondCoordinator.process(
            envelope: Self.envelope(
                generatedAt: "2026-04-24T09:00:00Z",
                costThresholdUSD: 25,
                conditions: [
                    Self.dailyCostCondition(dayKey: "2026-04-24", isActive: true)
                ]
            ),
            config: config
        )

        let requests = await scheduler.requests()
        #expect(requests.count == 2)
        #expect(requests.map(\.title) == [
            "Daily cost threshold crossed",
            "Daily cost threshold crossed",
        ])
    }

    @Test
    func disabledNotificationsSkipAuthorizationAndScheduling() async {
        let auth = NotificationAuthorizationSpy(status: .authorized, granted: true)
        let scheduler = NotificationSchedulerSpy()
        let coordinator = LocalNotificationCoordinator(
            authorizationManager: auth,
            scheduler: scheduler,
            stateStore: InMemoryLocalNotificationStateStore()
        )

        _ = await coordinator.process(
            envelope: Self.envelope(
                generatedAt: "2026-04-23T10:00:00Z",
                conditions: [Self.sessionCondition(isActive: true)]
            ),
            config: .default
        )

        #expect(await auth.statusCallCount() == 0)
        #expect((await scheduler.requests()).isEmpty)
    }

    @Test
    func enablingNotificationsRequestsPermissionAndSurfacesDenial() async {
        let auth = NotificationAuthorizationSpy(status: .denied, granted: false)
        let coordinator = LocalNotificationCoordinator(
            authorizationManager: auth,
            scheduler: NotificationSchedulerSpy(),
            stateStore: InMemoryLocalNotificationStateStore()
        )

        let issue = await coordinator.handleConfigChange(
            previous: .default,
            current: {
                var config = HeimdallBarConfig.default
                config.localNotificationsEnabled = true
                return config
            }()
        )

        #expect(await auth.requestCallCount() == 1)
        #expect(issue?.kind == .localNotifications)
    }
}

struct LocalNotificationIntegrationTests {
    @MainActor
    @Test
    func refreshCoordinatorProcessesNotificationsOnlyForGlobalRefreshes() async {
        let notificationCoordinator = LocalNotificationCoordinatorSpy()
        let coordinator = RefreshCoordinator(
            sessionStore: AppSessionStore(persistence: NoopAppSessionStateStore()),
            repository: ProviderRepository(),
            helperRuntime: ReadyHelperRuntime(),
            adjunctLoader: NoopAdjunctLoader(),
            browserSessionManager: NoopBrowserSessionManager(),
            widgetSnapshotCoordinator: WidgetSnapshotCoordinator(
                writer: StubWidgetSnapshotWriter(),
                reloader: NoopWidgetReloader()
            ),
            providerDataSource: StubProviderDataSource(),
            localNotificationCoordinator: notificationCoordinator
        )

        await coordinator.refresh(force: false, provider: nil)
        await coordinator.refresh(force: true, provider: .claude)

        let deadline = DispatchTime.now().uptimeNanoseconds + 1_000_000_000
        while DispatchTime.now().uptimeNanoseconds < deadline,
              await notificationCoordinator.processCallCount() < 1 {
            try? await Task.sleep(nanoseconds: 10_000_000)
        }

        #expect(await notificationCoordinator.processCallCount() == 1)
    }

    @MainActor
    @Test
    func settingsSaveOnlyInvokesNotificationHandlingWhenToggleChanges() async {
        let notificationCoordinator = LocalNotificationCoordinatorSpy()
        let sessionStore = AppSessionStore(persistence: NoopAppSessionStateStore())
        let settingsStore = SettingsStoreSpy()
        let featureModel = SettingsFeatureModel(
            sessionStore: sessionStore,
            repository: ProviderRepository(),
            settingsStore: settingsStore,
            refreshCoordinator: RefreshCoordinator(
                sessionStore: sessionStore,
                repository: ProviderRepository(),
                helperRuntime: ReadyHelperRuntime(),
                adjunctLoader: NoopAdjunctLoader(),
                browserSessionManager: NoopBrowserSessionManager(),
                widgetSnapshotCoordinator: WidgetSnapshotCoordinator(
                    writer: StubWidgetSnapshotWriter(),
                    reloader: NoopWidgetReloader()
                ),
                providerDataSource: StubProviderDataSource(),
                localNotificationCoordinator: notificationCoordinator
            ),
            localNotificationCoordinator: notificationCoordinator
        )

        featureModel.draftConfig.localNotificationsEnabled = true
        await featureModel.save()
        await featureModel.save()

        #expect(await notificationCoordinator.configChangeCallCount() == 1)
        #expect(settingsStore.savedConfigs.count == 2)
        #expect(sessionStore.config.localNotificationsEnabled)
    }
}

private extension LocalNotificationCoordinatorTests {
    static func envelope(
        generatedAt: String,
        costThresholdUSD: Double? = nil,
        conditions: [LocalNotificationCondition]
    ) -> ProviderSnapshotEnvelope {
        ProviderSnapshotEnvelope(
            providers: [],
            fetchedAt: generatedAt,
            requestedProvider: nil,
            responseScope: "all",
            cacheHit: false,
            refreshedProviders: ["claude", "codex"],
            localNotificationState: LocalNotificationState(
                generatedAt: generatedAt,
                costThresholdUSD: costThresholdUSD,
                conditions: conditions
            )
        )
    }

    static func sessionCondition(isActive: Bool) -> LocalNotificationCondition {
        LocalNotificationCondition(
            id: "claude-session-depleted",
            kind: "session_depleted",
            provider: "claude",
            serviceLabel: "Claude",
            isActive: isActive,
            activationTitle: "Claude session depleted",
            activationBody: "Claude session is depleted.",
            recoveryTitle: "Claude session restored",
            recoveryBody: "Claude session capacity is available again."
        )
    }

    static func dailyCostCondition(dayKey: String, isActive: Bool) -> LocalNotificationCondition {
        LocalNotificationCondition(
            id: "daily-cost-threshold",
            kind: "daily_cost_threshold",
            provider: nil,
            serviceLabel: "Heimdall",
            isActive: isActive,
            activationTitle: "Daily cost threshold crossed",
            activationBody: "Today's Heimdall cost crossed the threshold.",
            dayKey: dayKey
        )
    }
}

private actor NotificationAuthorizationSpy: NotificationAuthorizationManaging {
    private let statusValue: NotificationAuthorizationStatus
    private let grantedValue: Bool
    private var statusCalls = 0
    private var requestCalls = 0

    init(status: NotificationAuthorizationStatus, granted: Bool) {
        self.statusValue = status
        self.grantedValue = granted
    }

    func authorizationStatus() async -> NotificationAuthorizationStatus {
        self.statusCalls += 1
        return self.statusValue
    }

    func requestAuthorization() async throws -> Bool {
        self.requestCalls += 1
        return self.grantedValue
    }

    func statusCallCount() -> Int { self.statusCalls }
    func requestCallCount() -> Int { self.requestCalls }
}

private actor NotificationSchedulerSpy: LocalNotificationScheduling {
    private var values: [LocalNotificationRequest] = []

    func schedule(_ request: LocalNotificationRequest) async throws {
        self.values.append(request)
    }

    func requests() -> [LocalNotificationRequest] {
        self.values
    }
}

private final class InMemoryLocalNotificationStateStore: @unchecked Sendable, LocalNotificationStatePersisting {
    private var state = PersistedLocalNotificationState()

    func loadState() -> PersistedLocalNotificationState {
        self.state
    }

    func saveState(_ state: PersistedLocalNotificationState) {
        self.state = state
    }

    func clearState() {
        self.state = PersistedLocalNotificationState()
    }
}

private actor LocalNotificationCoordinatorSpy: LocalNotificationCoordinating {
    private var processCalls = 0
    private var configChangeCalls = 0

    func handleConfigChange(previous _: HeimdallBarConfig, current _: HeimdallBarConfig) async -> AppIssue? {
        self.configChangeCalls += 1
        return nil
    }

    func process(envelope _: ProviderSnapshotEnvelope, config _: HeimdallBarConfig) async -> AppIssue? {
        self.processCalls += 1
        return nil
    }

    func processCallCount() -> Int { self.processCalls }
    func configChangeCallCount() -> Int { self.configChangeCalls }
}

private final class SettingsStoreSpy: @unchecked Sendable, SettingsStore {
    private(set) var savedConfigs: [HeimdallBarConfig] = []

    func load() -> HeimdallBarConfig { .default }
    func save(_ config: HeimdallBarConfig) throws {
        self.savedConfigs.append(config)
    }
    func validate() throws {}
}

private struct StubProviderDataSource: ProviderDataSource {
    func fetchSnapshots(
        config _: HeimdallBarConfig,
        refresh _: Bool,
        provider _: ProviderID?
    ) async throws -> ProviderSnapshotEnvelope {
        ProviderSnapshotEnvelope(
            providers: [],
            fetchedAt: "2026-04-23T10:00:00Z",
            requestedProvider: nil,
            responseScope: "all",
            cacheHit: false,
            refreshedProviders: ["claude"],
            localNotificationState: LocalNotificationState(
                generatedAt: "2026-04-23T10:00:00Z",
                conditions: [
                    LocalNotificationCondition(
                        id: "claude-session-depleted",
                        kind: "session_depleted",
                        provider: "claude",
                        serviceLabel: "Claude",
                        isActive: false,
                        activationTitle: "Claude session depleted",
                        activationBody: "Claude session is depleted.",
                        recoveryTitle: "Claude session restored",
                        recoveryBody: "Claude session capacity is available again."
                    )
                ]
            )
        )
    }

    func fetchCostSummary(
        config _: HeimdallBarConfig,
        provider: ProviderID
    ) async throws -> CostSummaryEnvelope {
        CostSummaryEnvelope(
            provider: provider.rawValue,
            summary: ProviderCostSummary(
                todayTokens: 0,
                todayCostUSD: 0,
                last30DaysTokens: 0,
                last30DaysCostUSD: 0,
                daily: []
            )
        )
    }
}

private struct ReadyHelperRuntime: HelperRuntime {
    func ensureServerRunning(port _: Int) async -> Bool { true }
    func stopOwnedHelper() async {}
}

private struct NoopAdjunctLoader: DashboardAdjunctLoading {
    func loadAdjunct(
        provider _: ProviderID,
        config _: ProviderConfig,
        snapshot _: ProviderSnapshot?,
        forceRefresh _: Bool,
        allowLiveNavigation _: Bool
    ) async -> DashboardAdjunctSnapshot? { nil }
}

private struct NoopBrowserSessionManager: BrowserSessionManaging {
    func importedSession(provider _: ProviderID) async -> ImportedBrowserSession? { nil }
    func discoverImportCandidates(provider _: ProviderID) async -> [BrowserSessionImportCandidate] { [] }
    func importBrowserSession(
        provider _: ProviderID,
        candidate _: BrowserSessionImportCandidate
    ) async throws -> ImportedBrowserSession {
        throw NSError(domain: "test", code: 1)
    }
    func resetImportedSession(provider _: ProviderID) async throws {}
}

private struct StubWidgetSnapshotWriter: WidgetSnapshotWriter {
    func save(_ snapshot: WidgetSnapshot) throws -> WidgetSnapshotSaveResult { .unchanged }
    func load() -> WidgetSnapshotLoadResult { .empty }
}

private struct NoopAppSessionStateStore: AppSessionStatePersisting {
    func loadAppSessionState() -> PersistedAppSessionState? { nil }
    func saveAppSessionState(_ state: PersistedAppSessionState) {}
}
