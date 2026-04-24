import Foundation
import HeimdallDomain
import HeimdallServices
import SwiftUI
@testable import HeimdallMobileApp
import Testing

@MainActor
struct MobileDashboardModelTests {
    @Test
    func startupLoadsCachedAggregateBeforeLiveRefresh() async {
        let cachedAggregate = SyncedAggregateEnvelope.singleInstallation(
            mobileSnapshot: .fixture(
                generatedAt: "2026-04-20T11:00:00Z",
                providers: [.fixture(provider: .claude)]
            ),
            installationID: "cached-installation"
        )
        let liveAggregate = SyncedAggregateEnvelope.singleInstallation(
            mobileSnapshot: .fixture(
                generatedAt: "2026-04-21T11:00:00Z",
                providers: [.fixture(provider: .codex)]
            ),
            installationID: "live-installation"
        )
        let cache = StubAggregateCache(
            cached: CachedSyncedAggregateEnvelope(
                aggregate: cachedAggregate,
                cachedAt: "2026-04-20T11:01:00Z",
                lastSuccessfulRefreshAt: "2026-04-20T11:01:00Z"
            )
        )
        let store = ControlledSnapshotStore(
            liveAggregate: liveAggregate,
            liveDelayNanoseconds: 25_000_000
        )
        let model = MobileDashboardModel(store: store, cache: cache)

        let refreshTask = Task {
            await model.refresh(reason: .startup)
        }

        await waitUntil { await store.liveAggregateLoadCount == 1 }

        #expect(model.aggregate?.generatedAt == cachedAggregate.generatedAt)

        await refreshTask.value

        #expect(model.aggregate?.generatedAt == liveAggregate.generatedAt)
        #expect(model.selectedProvider == .codex)
    }

    @Test
    func foregroundRefreshRespectsThrottle() async {
        let clock = MutableNow(Date(timeIntervalSince1970: 1_000))
        let aggregate = SyncedAggregateEnvelope.singleInstallation(
            mobileSnapshot: .fixture(generatedAt: "2026-04-21T11:00:00Z"),
            installationID: "fixture-installation"
        )
        let store = ControlledSnapshotStore(liveAggregate: aggregate)
        let model = MobileDashboardModel(
            store: store,
            now: { clock.current },
            foregroundRefreshThrottle: 60
        )

        await model.refresh(reason: .manual)
        clock.current = clock.current.addingTimeInterval(30)
        await model.refresh(reason: .foreground)

        #expect(await store.liveAggregateLoadCount == 1)
    }

    @Test
    func manualRefreshBypassesThrottle() async {
        let clock = MutableNow(Date(timeIntervalSince1970: 1_000))
        let aggregate = SyncedAggregateEnvelope.singleInstallation(
            mobileSnapshot: .fixture(generatedAt: "2026-04-21T11:00:00Z"),
            installationID: "fixture-installation"
        )
        let store = ControlledSnapshotStore(liveAggregate: aggregate)
        let model = MobileDashboardModel(
            store: store,
            now: { clock.current },
            foregroundRefreshThrottle: 60
        )

        await model.refresh(reason: .manual)
        clock.current = clock.current.addingTimeInterval(30)
        await model.refresh(reason: .manual)

        #expect(await store.liveAggregateLoadCount == 2)
    }

    @Test
    func shareAcceptanceForcesRefreshAndUpdatesCloudSyncState() async {
        let aggregate = SyncedAggregateEnvelope.singleInstallation(
            mobileSnapshot: .fixture(generatedAt: "2026-04-21T11:00:00Z"),
            installationID: "fixture-installation"
        )
        let store = ControlledSnapshotStore(
            liveAggregate: aggregate,
            acceptShareState: CloudSyncSpaceState(
                role: .participant,
                status: .participantJoined,
                lastAcceptedAt: "2026-04-21T11:05:00Z"
            )
        )
        let model = MobileDashboardModel(store: store)

        await model.acceptShareURL(URL(string: "https://example.com/share")!)

        #expect(model.aggregate?.generatedAt == aggregate.generatedAt)
        #expect(model.cloudSyncState.status == .participantJoined)
        #expect(await store.acceptShareCount == 1)
        #expect(await store.liveAggregateLoadCount == 1)
    }

    @Test
    func coordinatorStartsOnceAndRefreshesWhenSceneBecomesActive() async {
        let clock = MutableNow(Date(timeIntervalSince1970: 1_000))
        let aggregate = SyncedAggregateEnvelope.singleInstallation(
            mobileSnapshot: .fixture(generatedAt: "2026-04-21T11:00:00Z"),
            installationID: "fixture-installation"
        )
        let store = ControlledSnapshotStore(liveAggregate: aggregate)
        let coordinator = MobileDashboardCoordinator(
            dashboard: MobileDashboardModel(
                store: store,
                now: { clock.current },
                foregroundRefreshThrottle: 60
            )
        )

        await coordinator.start()
        await coordinator.start()
        #expect(await store.liveAggregateLoadCount == 1)

        coordinator.handleScenePhaseChange(.background)
        clock.current = clock.current.addingTimeInterval(61)
        coordinator.handleScenePhaseChange(.active)

        await waitUntil { await store.liveAggregateLoadCount == 2 }
    }

    @Test
    func coordinatorRoutesOpenURLsThroughDashboardModel() async {
        let aggregate = SyncedAggregateEnvelope.singleInstallation(
            mobileSnapshot: .fixture(generatedAt: "2026-04-21T11:00:00Z"),
            installationID: "fixture-installation"
        )
        let store = ControlledSnapshotStore(
            liveAggregate: aggregate,
            acceptShareState: CloudSyncSpaceState(
                role: .participant,
                status: .participantJoined,
                lastAcceptedAt: "2026-04-21T11:05:00Z"
            )
        )
        let model = MobileDashboardModel(store: store)
        let coordinator = MobileDashboardCoordinator(dashboard: model)

        coordinator.handleOpenURL(URL(string: "https://example.com/share")!)

        await waitUntil { await store.acceptShareCount == 1 }

        #expect(model.cloudSyncState.status == .participantJoined)
        #expect(model.aggregate?.generatedAt == aggregate.generatedAt)
    }

    @Test
    func failedLiveRefreshPreservesCachedDataAndShowsWarning() async {
        let cachedAggregate = SyncedAggregateEnvelope.singleInstallation(
            mobileSnapshot: .fixture(generatedAt: "2026-04-21T10:00:00Z"),
            installationID: "cached-installation"
        )
        let cache = StubAggregateCache(
            cached: CachedSyncedAggregateEnvelope(
                aggregate: cachedAggregate,
                cachedAt: "2026-04-21T10:00:05Z",
                lastSuccessfulRefreshAt: "2026-04-21T10:00:05Z"
            )
        )
        let store = ControlledSnapshotStore(liveError: FixtureError.syncFailed)
        let model = MobileDashboardModel(store: store, cache: cache)

        await model.refresh(reason: .startup)

        #expect(model.aggregate?.generatedAt == cachedAggregate.generatedAt)
        #expect(model.lastRefreshError == "sync failed")
        #expect(model.staleSnapshotWarning == "sync failed")
    }

    @Test
    func loadFailureWithoutCacheStillShowsErrorState() async {
        let store = ControlledSnapshotStore(liveError: FixtureError.syncFailed)
        let model = MobileDashboardModel(store: store)

        await model.refresh(reason: .manual)

        #expect(model.aggregate == nil)
        #expect(model.lastRefreshError == "sync failed")
    }

    @Test
    func widgetSnapshotAdapterBuildsStableProviderPayloadsFromAggregate() {
        let aggregate = SyncedAggregateEnvelope.singleInstallation(
            mobileSnapshot: .fixture(
                generatedAt: "2026-04-21T11:00:00Z",
                providers: [
                    .fixture(
                        provider: .claude,
                        identity: ProviderIdentity(
                            provider: ProviderID.claude.rawValue,
                            accountEmail: "private@example.com",
                            accountOrganization: "Private",
                            loginMethod: "oauth",
                            plan: "Max"
                        ),
                        status: ProviderStatusSummary(
                            indicator: "minor",
                            description: "Minor incident",
                            pageURL: "https://status.example.com"
                        ),
                        primary: ProviderRateWindow(
                            usedPercent: 62,
                            resetsAt: "2026-04-21T12:00:00Z",
                            resetsInMinutes: 42,
                            windowMinutes: 300,
                            resetLabel: "42m"
                        )
                    ),
                ]
            ),
            installationID: "fixture-installation"
        )

        let widgetSnapshot = WidgetSnapshotBuilder.snapshot(aggregate: aggregate, defaultRefreshIntervalSeconds: 900)
        let provider = widgetSnapshot.providerSnapshot(for: .claude)

        #expect(widgetSnapshot.generatedAt == aggregate.generatedAt)
        #expect(widgetSnapshot.defaultRefreshIntervalSeconds == 900)
        #expect(provider?.cost.todayTokens == 900)
        #expect(provider?.freshness.visualState == .degraded)
        #expect(provider?.identity?.accountEmail == nil)
        #expect(provider?.identity?.plan == "Max")
        #expect(provider?.lanes.first?.usedPercent == 62)
    }

    @Test
    func widgetSnapshotPersistenceReloadsOnlyWhenPayloadChanges() async throws {
        let writer = InMemoryWidgetSnapshotWriter()
        let reloader = CountingWidgetReloader()
        let coordinator = WidgetSnapshotCoordinator(writer: writer, reloader: reloader)
        let aggregate = SyncedAggregateEnvelope.singleInstallation(
            mobileSnapshot: .fixture(generatedAt: "2026-04-21T11:00:00Z"),
            installationID: "fixture-installation"
        )
        let store = ControlledSnapshotStore(liveAggregate: aggregate)
        let model = MobileDashboardModel(
            store: store,
            widgetSnapshotCoordinator: coordinator
        )

        await model.refresh(reason: .manual)
        await model.refresh(reason: .manual)

        #expect(reloader.reloadCount == 1)
        #expect(writer.savedSnapshots.count == 2)
        switch writer.load() {
        case .success(let snapshot):
            #expect(snapshot.generatedAt == aggregate.generatedAt)
        case .empty, .failure:
            Issue.record("expected widget snapshot to be persisted")
        }
    }

    @Test
    func handleRemotePushForcesWidgetReloadAndReportsNewData() async {
        let writer = InMemoryWidgetSnapshotWriter()
        let reloader = CountingWidgetReloader()
        let coordinator = WidgetSnapshotCoordinator(writer: writer, reloader: reloader)
        let aggregate = SyncedAggregateEnvelope.singleInstallation(
            mobileSnapshot: .fixture(generatedAt: "2026-04-21T11:00:00Z"),
            installationID: "fixture-installation"
        )
        let store = ControlledSnapshotStore(liveAggregate: aggregate)
        let model = MobileDashboardModel(
            store: store,
            widgetSnapshotCoordinator: coordinator
        )

        await model.refresh(reason: .manual)
        #expect(reloader.reloadCount == 1)

        let firstResult = await model.handleRemotePush()
        #expect(firstResult == .newData)
        #expect(reloader.reloadCount == 2)

        let secondResult = await model.handleRemotePush()
        #expect(secondResult == .newData)
        // Push-triggered refresh forces a timeline reload even when the
        // persisted payload hasn't changed; the "last refreshed" chrome
        // in the widget should tick on every push.
        #expect(reloader.reloadCount == 3)
    }

    @Test
    func handleRemotePushReportsFailureWhenRefreshErrorsAndNoAggregateAvailable() async {
        let reloader = CountingWidgetReloader()
        let coordinator = WidgetSnapshotCoordinator(
            writer: InMemoryWidgetSnapshotWriter(),
            reloader: reloader
        )
        let store = ControlledSnapshotStore(liveError: FixtureError.syncFailed)
        let model = MobileDashboardModel(
            store: store,
            widgetSnapshotCoordinator: coordinator
        )

        let result = await model.handleRemotePush()
        #expect(result == .failed)
        #expect(reloader.reloadCount == 1)
    }

    @Test
    func cloudSyncStatusCopyMapsUnavailableStates() async {
        let unavailableModel = MobileDashboardModel(
            store: ControlledSnapshotStore(
                liveAggregate: nil,
                cloudSyncState: CloudSyncSpaceState(status: .iCloudUnavailable)
            )
        )
        await unavailableModel.refresh(reason: .manual)
        #expect(unavailableModel.cloudSyncStatusTitle == "iCloud unavailable")
        #expect(unavailableModel.cloudSyncStatusDetail == "Sign in to iCloud on this iPhone to refresh synced usage data.")

        let restrictedModel = MobileDashboardModel(
            store: ControlledSnapshotStore(
                liveAggregate: nil,
                cloudSyncState: CloudSyncSpaceState(status: .sharingBlocked)
            )
        )
        await restrictedModel.refresh(reason: .manual)
        #expect(restrictedModel.cloudSyncStatusTitle == "Sharing blocked")
        #expect(restrictedModel.cloudSyncStatusDetail == "CloudKit sharing is restricted on this iPhone.")
    }

    @Test
    func accountOptionsDeriveEmailOrganizationAndDeviceFallback() async {
        let aggregate = SyncedAggregateEnvelope.aggregate(
            installations: [
                .fixture(
                    installationID: "claude-email",
                    sourceDevice: "personal-macbook",
                    providers: [
                        .fixture(
                            provider: .claude,
                            identity: ProviderIdentity(
                                provider: ProviderID.claude.rawValue,
                                accountEmail: "work@example.com",
                                accountOrganization: nil,
                                loginMethod: "oauth",
                                plan: "Max"
                            )
                        ),
                    ]
                ),
                .fixture(
                    installationID: "codex-org",
                    sourceDevice: "work-macbook",
                    providers: [
                        .fixture(
                            provider: .codex,
                            identity: ProviderIdentity(
                                provider: ProviderID.codex.rawValue,
                                accountEmail: nil,
                                accountOrganization: "ACME Corp",
                                loginMethod: "oauth",
                                plan: "Team"
                            )
                        ),
                    ]
                ),
                .fixture(
                    installationID: "claude-device",
                    sourceDevice: "travel-macbook",
                    providers: [
                        .fixture(provider: .claude, identity: nil),
                    ]
                ),
            ],
            generatedAt: "2026-04-21T11:00:00Z"
        )
        let model = MobileDashboardModel(store: ControlledSnapshotStore(liveAggregate: aggregate))

        await model.refresh(reason: .manual)

        #expect(model.rawAccountOptions.map(\.displayTitle).contains("Claude: work@example.com"))
        #expect(model.rawAccountOptions.map(\.displayTitle).contains("Codex: ACME Corp"))
        #expect(model.rawAccountOptions.map(\.displayTitle).contains("Claude: travel-macbook"))
    }

    @Test
    func aliasGroupsOverrideRawPresentationAndCanSpanProviders() async {
        let aggregate = SyncedAggregateEnvelope.aggregate(
            installations: [
                .fixture(
                    installationID: "claude-personal",
                    sourceDevice: "personal-macbook",
                    providers: [
                        .fixture(
                            provider: .claude,
                            identity: ProviderIdentity(
                                provider: ProviderID.claude.rawValue,
                                accountEmail: "personal@example.com",
                                accountOrganization: nil,
                                loginMethod: "oauth",
                                plan: "Max"
                            )
                        ),
                    ]
                ),
                .fixture(
                    installationID: "codex-personal",
                    sourceDevice: "personal-macbook",
                    providers: [
                        .fixture(
                            provider: .codex,
                            identity: ProviderIdentity(
                                provider: ProviderID.codex.rawValue,
                                accountEmail: "personal@example.com",
                                accountOrganization: nil,
                                loginMethod: "oauth",
                                plan: "Pro"
                            )
                        ),
                    ]
                ),
            ],
            generatedAt: "2026-04-21T11:00:00Z"
        )
        let preferences = StubMobileDashboardPreferencesStore(
            preferences: PersistedMobileDashboardPreferences(
                aliases: [
                    MobileAccountAlias(
                        id: "personal",
                        title: "Personal",
                        sourceLabelKeys: [
                            "claude|email|personal@example.com",
                            "codex|email|personal@example.com",
                        ]
                    ),
                ]
            )
        )
        let model = MobileDashboardModel(
            store: ControlledSnapshotStore(liveAggregate: aggregate),
            preferencesStore: preferences
        )

        await model.refresh(reason: .manual)

        #expect(model.aliasAccountOptions.map(\.displayTitle) == ["Personal"])
        #expect(!model.rawAccountOptions.map(\.displayTitle).contains("Claude: personal@example.com"))
        #expect(!model.rawAccountOptions.map(\.displayTitle).contains("Codex: personal@example.com"))
    }

    @Test
    func filteredTotalsHistoryAndCurrentLimitRowsRecomputeForSelectedAccountScope() async {
        let aggregate = SyncedAggregateEnvelope.aggregate(
            installations: [
                .fixture(
                    installationID: "work-claude",
                    sourceDevice: "work-macbook",
                    providers: [
                        .fixture(
                            provider: .claude,
                            identity: ProviderIdentity(
                                provider: ProviderID.claude.rawValue,
                                accountEmail: "work@example.com",
                                accountOrganization: nil,
                                loginMethod: "oauth",
                                plan: "Max"
                            ),
                            primary: ProviderRateWindow(
                                usedPercent: 40,
                                resetsAt: "2026-04-21T13:00:00Z",
                                resetsInMinutes: 60,
                                windowMinutes: 300,
                                resetLabel: "1h"
                            ),
                            todayTokens: 500,
                            todayCostUSD: 5,
                            last30DaysTokens: 1500,
                            last30DaysCostUSD: 15
                        ),
                    ],
                    history90d: [
                        .fixture(provider: .claude, totalTokens: 3000, totalCostUSD: 30),
                    ]
                ),
                .fixture(
                    installationID: "personal-codex",
                    sourceDevice: "personal-macbook",
                    providers: [
                        .fixture(
                            provider: .codex,
                            identity: ProviderIdentity(
                                provider: ProviderID.codex.rawValue,
                                accountEmail: "personal@example.com",
                                accountOrganization: nil,
                                loginMethod: "oauth",
                                plan: "Pro"
                            ),
                            todayTokens: 800,
                            todayCostUSD: 8,
                            last30DaysTokens: 2000,
                            last30DaysCostUSD: 20
                        ),
                    ],
                    history90d: [
                        .fixture(provider: .codex, totalTokens: 4000, totalCostUSD: 40),
                    ]
                ),
            ],
            generatedAt: "2026-04-21T11:00:00Z"
        )
        let model = MobileDashboardModel(store: ControlledSnapshotStore(liveAggregate: aggregate))

        await model.refresh(reason: .manual)
        model.selectAccountScope(.account("claude|email|work@example.com"))

        #expect(model.scopedAggregate?.aggregateTotals.todayTokens == 500)
        #expect(model.scopedAggregate?.aggregateTotals.last90DaysTokens == 3000)
        #expect(model.selectedHistorySeries?.providerID == .claude)
        #expect(model.visibleInstallations.map(\.installationID) == ["work-claude"])
        #expect(model.scopedAggregate?.aggregateProviderViews.first?.currentLimitInstallationIDs == ["work-claude"])
    }

    @Test
    func selectedProviderFallsBackWhenFilteredOutByAccountScope() async {
        let aggregate = SyncedAggregateEnvelope.aggregate(
            installations: [
                .fixture(
                    installationID: "claude-installation",
                    sourceDevice: "claude-mac",
                    providers: [
                        .fixture(
                            provider: .claude,
                            identity: ProviderIdentity(
                                provider: ProviderID.claude.rawValue,
                                accountEmail: "claude@example.com",
                                accountOrganization: nil,
                                loginMethod: "oauth",
                                plan: "Max"
                            )
                        ),
                    ]
                ),
                .fixture(
                    installationID: "codex-installation",
                    sourceDevice: "codex-mac",
                    providers: [
                        .fixture(
                            provider: .codex,
                            identity: ProviderIdentity(
                                provider: ProviderID.codex.rawValue,
                                accountEmail: "codex@example.com",
                                accountOrganization: nil,
                                loginMethod: "oauth",
                                plan: "Pro"
                            )
                        ),
                    ]
                ),
            ],
            generatedAt: "2026-04-21T11:00:00Z"
        )
        let model = MobileDashboardModel(store: ControlledSnapshotStore(liveAggregate: aggregate))

        await model.refresh(reason: .manual)
        model.selectProvider(.codex)
        model.selectAccountScope(.account("claude|email|claude@example.com"))

        #expect(model.selectedProvider == .claude)
    }

    @Test
    func scopedEmptyStateAppearsForAliasWithoutVisibleProviders() async {
        let aggregate = SyncedAggregateEnvelope.aggregate(
            installations: [
                .fixture(
                    installationID: "claude-installation",
                    sourceDevice: "claude-mac",
                    providers: [
                        .fixture(
                            provider: .claude,
                            identity: ProviderIdentity(
                                provider: ProviderID.claude.rawValue,
                                accountEmail: "claude@example.com",
                                accountOrganization: nil,
                                loginMethod: "oauth",
                                plan: "Max"
                            )
                        ),
                    ]
                ),
            ],
            generatedAt: "2026-04-21T11:00:00Z"
        )
        let preferences = StubMobileDashboardPreferencesStore(
            preferences: PersistedMobileDashboardPreferences(
                selectedAccountScope: .alias("work"),
                aliases: [
                    MobileAccountAlias(
                        id: "work",
                        title: "Work",
                        sourceLabelKeys: ["codex|email|missing@example.com"]
                    ),
                ]
            )
        )
        let model = MobileDashboardModel(
            store: ControlledSnapshotStore(liveAggregate: aggregate),
            preferencesStore: preferences
        )

        await model.refresh(reason: .manual)

        #expect(model.hasSnapshot)
        #expect(!model.hasScopedData)
        #expect(model.selectedScopeTitle == "Work")
    }

    @Test
    func compactModeRollsUpLowSignalRowsButKeepsStaleAndLiveInstallationsVisible() async {
        let preferences = StubMobileDashboardPreferencesStore(
            preferences: PersistedMobileDashboardPreferences(
                compressionPreference: .compact
            )
        )
        let aggregate = SyncedAggregateEnvelope.aggregate(
            installations: [
                .fixture(
                    installationID: "stale-installation",
                    sourceDevice: "stale-mac",
                    providers: [
                        .fixture(
                            provider: .claude,
                            identity: ProviderIdentity(
                                provider: ProviderID.claude.rawValue,
                                accountEmail: "stale@example.com",
                                accountOrganization: nil,
                                loginMethod: "oauth",
                                plan: "Max"
                            ),
                            stale: true,
                            todayTokens: 10,
                            todayCostUSD: 0.1,
                            last30DaysTokens: 50,
                            last30DaysCostUSD: 0.5
                        ),
                    ],
                    history90d: [
                        .fixture(provider: .claude, totalTokens: 50, totalCostUSD: 0.5),
                    ]
                ),
                .fixture(
                    installationID: "live-installation",
                    sourceDevice: "live-mac",
                    providers: [
                        .fixture(
                            provider: .codex,
                            identity: ProviderIdentity(
                                provider: ProviderID.codex.rawValue,
                                accountEmail: "live@example.com",
                                accountOrganization: nil,
                                loginMethod: "oauth",
                                plan: "Pro"
                            ),
                            primary: ProviderRateWindow(
                                usedPercent: 30,
                                resetsAt: "2026-04-21T13:00:00Z",
                                resetsInMinutes: 30,
                                windowMinutes: 60,
                                resetLabel: "30m"
                            ),
                            todayTokens: 700,
                            todayCostUSD: 7,
                            last30DaysTokens: 1400,
                            last30DaysCostUSD: 14
                        ),
                    ],
                    history90d: [
                        .fixture(provider: .codex, totalTokens: 1400, totalCostUSD: 14),
                    ]
                ),
                .fixture(installationID: "small-1", sourceDevice: "small-1", providers: [.fixture(provider: .claude, available: false, todayTokens: 50, todayCostUSD: 0.5, last30DaysTokens: 100, last30DaysCostUSD: 1)], history90d: [.fixture(provider: .claude, totalTokens: 100, totalCostUSD: 1)]),
                .fixture(installationID: "small-2", sourceDevice: "small-2", providers: [.fixture(provider: .claude, available: false, todayTokens: 40, todayCostUSD: 0.4, last30DaysTokens: 90, last30DaysCostUSD: 0.9)], history90d: [.fixture(provider: .claude, totalTokens: 90, totalCostUSD: 0.9)]),
                .fixture(installationID: "small-3", sourceDevice: "small-3", providers: [.fixture(provider: .codex, available: false, todayTokens: 30, todayCostUSD: 0.3, last30DaysTokens: 80, last30DaysCostUSD: 0.8)], history90d: [.fixture(provider: .codex, totalTokens: 80, totalCostUSD: 0.8)]),
            ],
            generatedAt: "2026-04-21T11:00:00Z"
        )
        let model = MobileDashboardModel(
            store: ControlledSnapshotStore(liveAggregate: aggregate),
            preferencesStore: preferences
        )

        await model.refresh(reason: .manual)

        #expect(model.visibleInstallations.map(\.installationID).contains("stale-installation"))
        #expect(model.visibleInstallations.map(\.installationID).contains("live-installation"))
        #expect(!model.rolledUpInstallations.isEmpty)
    }

    @Test
    func expandedModeDisablesRollupsAndRestoresFullLists() async {
        let preferences = StubMobileDashboardPreferencesStore(
            preferences: PersistedMobileDashboardPreferences(
                compressionPreference: .expanded
            )
        )
        let aggregate = SyncedAggregateEnvelope.aggregate(
            installations: [
                .fixture(installationID: "one", sourceDevice: "one", providers: [.fixture(provider: .claude)], history90d: [.fixture(provider: .claude, totalTokens: 100, totalCostUSD: 1)]),
                .fixture(installationID: "two", sourceDevice: "two", providers: [.fixture(provider: .codex)], history90d: [.fixture(provider: .codex, totalTokens: 110, totalCostUSD: 1.1)]),
                .fixture(installationID: "three", sourceDevice: "three", providers: [.fixture(provider: .claude)], history90d: [.fixture(provider: .claude, totalTokens: 120, totalCostUSD: 1.2)]),
                .fixture(installationID: "four", sourceDevice: "four", providers: [.fixture(provider: .codex)], history90d: [.fixture(provider: .codex, totalTokens: 130, totalCostUSD: 1.3)]),
            ],
            generatedAt: "2026-04-21T11:00:00Z"
        )
        let model = MobileDashboardModel(
            store: ControlledSnapshotStore(liveAggregate: aggregate),
            preferencesStore: preferences
        )

        await model.refresh(reason: .manual)

        #expect(model.visibleInstallations.count == 4)
        #expect(model.rolledUpInstallations.isEmpty)
    }

    @Test
    func preferencesPersistSelectedAccountScopeAliasesAndCompression() async {
        let preferences = StubMobileDashboardPreferencesStore()
        let aggregate = SyncedAggregateEnvelope.aggregate(
            installations: [
                .fixture(
                    installationID: "claude-installation",
                    sourceDevice: "claude-mac",
                    providers: [
                        .fixture(
                            provider: .claude,
                            identity: ProviderIdentity(
                                provider: ProviderID.claude.rawValue,
                                accountEmail: "claude@example.com",
                                accountOrganization: nil,
                                loginMethod: "oauth",
                                plan: "Max"
                            )
                        ),
                    ]
                ),
            ],
            generatedAt: "2026-04-21T11:00:00Z"
        )
        let firstModel = MobileDashboardModel(
            store: ControlledSnapshotStore(liveAggregate: aggregate),
            preferencesStore: preferences
        )

        await firstModel.refresh(reason: .manual)
        firstModel.setCompressionPreference(.expanded)
        firstModel.replaceAliases([
            MobileAccountAlias(
                id: "personal",
                title: "Personal",
                sourceLabelKeys: ["claude|email|claude@example.com"]
            ),
        ])
        firstModel.selectAccountScope(.alias("personal"))

        let secondModel = MobileDashboardModel(
            store: ControlledSnapshotStore(liveAggregate: aggregate),
            preferencesStore: preferences
        )
        await secondModel.refresh(reason: .manual)

        #expect(secondModel.compressionPreference == .expanded)
        #expect(secondModel.aliases.map(\.title) == ["Personal"])
        #expect(secondModel.selectedAccountScope == .alias("personal"))
    }
}

@MainActor
private func waitUntil(
    maxAttempts: Int = 100,
    predicate: @escaping () async -> Bool
) async {
    for _ in 0..<maxAttempts {
        if await predicate() {
            return
        }
        await Task.yield()
    }
    Issue.record("timed out waiting for async condition")
}

private enum FixtureError: LocalizedError {
    case syncFailed

    var errorDescription: String? {
        switch self {
        case .syncFailed:
            return "sync failed"
        }
    }
}

private actor ControlledSnapshotStore: SnapshotSyncStore {
    let liveAggregate: SyncedAggregateEnvelope?
    let cloudSyncState: CloudSyncSpaceState
    let acceptShareState: CloudSyncSpaceState
    let liveError: Error?
    let aggregateError: Error?
    let liveDelayNanoseconds: UInt64

    private(set) var liveAggregateLoadCount = 0
    private(set) var acceptShareCount = 0

    init(
        liveAggregate: SyncedAggregateEnvelope? = nil,
        cloudSyncState: CloudSyncSpaceState = CloudSyncSpaceState(role: .participant, status: .participantJoined),
        acceptShareState: CloudSyncSpaceState = CloudSyncSpaceState(role: .participant, status: .participantJoined),
        liveError: Error? = nil,
        aggregateError: Error? = nil,
        liveDelayNanoseconds: UInt64 = 0
    ) {
        self.liveAggregate = liveAggregate
        self.cloudSyncState = cloudSyncState
        self.acceptShareState = acceptShareState
        self.liveError = liveError
        self.aggregateError = aggregateError
        self.liveDelayNanoseconds = liveDelayNanoseconds
    }

    func loadLiveAggregateSnapshot() async throws -> SyncedAggregateEnvelope? {
        self.liveAggregateLoadCount += 1
        if self.liveDelayNanoseconds > 0 {
            try? await Task.sleep(nanoseconds: self.liveDelayNanoseconds)
        }
        if let liveError {
            throw liveError
        }
        return self.liveAggregate
    }

    func saveLatestSnapshot(_ snapshot: MobileSnapshotEnvelope) async throws -> SyncedAggregateEnvelope {
        SyncedAggregateEnvelope.singleInstallation(mobileSnapshot: snapshot, installationID: "stub-installation")
    }

    func loadAggregateSnapshot() async throws -> SyncedAggregateEnvelope? {
        if let aggregateError {
            throw aggregateError
        }
        return self.liveAggregate
    }

    func loadCloudSyncSpaceState() async throws -> CloudSyncSpaceState {
        self.cloudSyncState
    }

    func prepareOwnerShare() async throws -> CloudSyncSpaceState {
        CloudSyncSpaceState(role: .owner, status: .inviteReady, shareURL: "https://example.com/share")
    }

    func acceptShareURL(_: URL) async throws -> CloudSyncSpaceState {
        self.acceptShareCount += 1
        return self.acceptShareState
    }
}

private actor StubAggregateCache: SyncedAggregateCaching {
    var cached: CachedSyncedAggregateEnvelope?

    init(cached: CachedSyncedAggregateEnvelope? = nil) {
        self.cached = cached
    }

    func loadCachedAggregate() async throws -> CachedSyncedAggregateEnvelope? {
        self.cached
    }

    func saveCachedAggregate(_ cached: CachedSyncedAggregateEnvelope) async throws {
        self.cached = cached
    }

    func purgeCachedAggregate() async throws {
        self.cached = nil
    }
}

private final class StubMobileDashboardPreferencesStore: @unchecked Sendable, MobileDashboardPreferencesPersisting {
    var preferences: PersistedMobileDashboardPreferences?

    init(preferences: PersistedMobileDashboardPreferences? = nil) {
        self.preferences = preferences
    }

    func loadPreferences() -> PersistedMobileDashboardPreferences? {
        self.preferences
    }

    func savePreferences(_ preferences: PersistedMobileDashboardPreferences) {
        self.preferences = preferences
    }
}

private final class MutableNow: @unchecked Sendable {
    var current: Date

    init(_ current: Date) {
        self.current = current
    }
}

private final class InMemoryWidgetSnapshotWriter: @unchecked Sendable, WidgetSnapshotWriter {
    private(set) var savedSnapshots: [WidgetSnapshot] = []
    private var latestSnapshot: WidgetSnapshot?
    private let encoder: JSONEncoder = {
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.sortedKeys]
        return encoder
    }()

    func save(_ snapshot: WidgetSnapshot) throws -> WidgetSnapshotSaveResult {
        self.savedSnapshots.append(snapshot)
        defer { self.latestSnapshot = snapshot }
        guard let latestSnapshot else {
            return .saved
        }
        let latestData = try self.encoder.encode(latestSnapshot)
        let nextData = try self.encoder.encode(snapshot)
        return latestData == nextData ? .unchanged : .saved
    }

    func load() -> WidgetSnapshotLoadResult {
        guard let latestSnapshot else { return .empty }
        return .success(latestSnapshot)
    }
}

private final class CountingWidgetReloader: @unchecked Sendable, WidgetReloading {
    private(set) var reloadCount = 0

    func reloadAllTimelines() {
        self.reloadCount += 1
    }
}

private extension MobileSnapshotEnvelope {
    static func fixture(
        generatedAt: String = "2026-04-21T11:00:00Z",
        providers: [ProviderSnapshot] = [.fixture(provider: .claude), .fixture(provider: .codex)],
        staleProviders: [String] = []
    ) -> MobileSnapshotEnvelope {
        MobileSnapshotEnvelope(
            generatedAt: generatedAt,
            sourceDevice: "macbook-air",
            providers: providers,
            history90d: providers.map { provider in
                MobileProviderHistorySeries(
                    provider: provider.provider,
                    daily: [
                        CostHistoryPoint(day: "2026-04-20", totalTokens: 1000, costUSD: 10.0),
                        CostHistoryPoint(day: "2026-04-21", totalTokens: 800, costUSD: 8.0),
                    ],
                    totalTokens: 1800,
                    totalCostUSD: 18.0
                )
            },
            totals: MobileSnapshotTotals(
                todayTokens: 1800,
                todayCostUSD: 18.0,
                last90DaysTokens: 3600,
                last90DaysCostUSD: 36.0
            ),
            freshness: MobileSnapshotFreshness(
                newestProviderRefresh: generatedAt,
                oldestProviderRefresh: "2026-04-21T10:30:00Z",
                staleProviders: staleProviders,
                hasStaleProviders: !staleProviders.isEmpty
            )
        )
    }
}

private extension ProviderSnapshot {
    static func fixture(
        provider: ProviderID,
        identity: ProviderIdentity? = nil,
        status: ProviderStatusSummary? = nil,
        primary: ProviderRateWindow? = nil,
        available: Bool = true,
        stale: Bool = false,
        todayTokens: Int = 900,
        todayCostUSD: Double = 9.0,
        last30DaysTokens: Int = 1800,
        last30DaysCostUSD: Double = 18.0
    ) -> ProviderSnapshot {
        ProviderSnapshot(
            provider: provider.rawValue,
            available: available,
            sourceUsed: "auto",
            lastAttemptedSource: "auto",
            resolvedViaFallback: false,
            refreshDurationMs: 0,
            sourceAttempts: [],
            identity: identity,
            primary: primary,
            secondary: nil,
            tertiary: nil,
            credits: nil,
            status: status,
            auth: ProviderAuthHealth(
                loginMethod: "sync",
                credentialBackend: nil,
                authMode: "sync",
                isAuthenticated: true,
                isRefreshable: false,
                isSourceCompatible: true,
                requiresRelogin: false,
                managedRestriction: nil,
                diagnosticCode: nil,
                failureReason: nil,
                lastValidatedAt: "2026-04-21T11:00:00Z",
                recoveryActions: []
            ),
            costSummary: ProviderCostSummary(
                todayTokens: todayTokens,
                todayCostUSD: todayCostUSD,
                last30DaysTokens: last30DaysTokens,
                last30DaysCostUSD: last30DaysCostUSD,
                daily: []
            ),
            claudeUsage: nil,
            lastRefresh: "2026-04-21T11:00:00Z",
            stale: stale,
            error: nil
        )
    }
}

private extension MobileProviderHistorySeries {
    static func fixture(
        provider: ProviderID,
        totalTokens: Int,
        totalCostUSD: Double
    ) -> MobileProviderHistorySeries {
        MobileProviderHistorySeries(
            provider: provider.rawValue,
            daily: [
                CostHistoryPoint(day: "2026-04-20", totalTokens: totalTokens / 2, costUSD: totalCostUSD / 2),
                CostHistoryPoint(day: "2026-04-21", totalTokens: totalTokens - (totalTokens / 2), costUSD: totalCostUSD / 2),
            ],
            totalTokens: totalTokens,
            totalCostUSD: totalCostUSD
        )
    }
}

private extension SyncedInstallationSnapshot {
    static func fixture(
        installationID: String,
        sourceDevice: String,
        providers: [ProviderSnapshot],
        history90d: [MobileProviderHistorySeries]? = nil,
        publishedAt: String = "2026-04-21T11:00:00Z"
    ) -> SyncedInstallationSnapshot {
        let history = history90d ?? providers.compactMap { snapshot in
            guard let providerID = snapshot.providerID else { return nil }
            return .fixture(
                provider: providerID,
                totalTokens: snapshot.costSummary.last30DaysTokens,
                totalCostUSD: snapshot.costSummary.last30DaysCostUSD
            )
        }
        return SyncedInstallationSnapshot(
            installationID: installationID,
            sourceDevice: sourceDevice,
            publishedAt: publishedAt,
            providers: providers,
            history90d: history,
            totals: MobileSnapshotTotals(
                todayTokens: providers.reduce(0) { partial, snapshot in partial + snapshot.costSummary.todayTokens },
                todayCostUSD: providers.reduce(0) { partial, snapshot in partial + snapshot.costSummary.todayCostUSD },
                last90DaysTokens: history.reduce(0) { partial, series in partial + series.totalTokens },
                last90DaysCostUSD: history.reduce(0) { partial, series in partial + series.totalCostUSD }
            ),
            freshness: MobileSnapshotFreshness(
                newestProviderRefresh: providers.map(\.lastRefresh).max(),
                oldestProviderRefresh: providers.map(\.lastRefresh).min(),
                staleProviders: providers.filter(\.stale).map(\.provider),
                hasStaleProviders: providers.contains(where: \.stale)
            )
        )
    }
}
