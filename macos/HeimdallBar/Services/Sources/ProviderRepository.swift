import Foundation
import HeimdallDomain
import Observation

@MainActor
@Observable
public final class ProviderContentStore {
    public var snapshots: [ProviderSnapshot]
    public var adjunctSnapshots: [ProviderID: DashboardAdjunctSnapshot?]
    public var syncedAggregate: SyncedAggregateEnvelope?
    public var cloudSyncState: CloudSyncSpaceState

    public init(
        snapshots: [ProviderSnapshot] = [],
        adjunctSnapshots: [ProviderID: DashboardAdjunctSnapshot?] = [:],
        syncedAggregate: SyncedAggregateEnvelope? = nil,
        cloudSyncState: CloudSyncSpaceState = CloudSyncSpaceState()
    ) {
        self.snapshots = snapshots
        self.adjunctSnapshots = adjunctSnapshots
        self.syncedAggregate = syncedAggregate
        self.cloudSyncState = cloudSyncState
    }

    public func snapshot(for provider: ProviderID) -> ProviderSnapshot? {
        self.snapshots.first(where: { $0.providerID == provider })
    }

    public var snapshotsByProvider: [ProviderID: ProviderSnapshot] {
        Dictionary(uniqueKeysWithValues: self.snapshots.compactMap { snapshot in
            snapshot.providerID.map { ($0, snapshot) }
        })
    }

    public func apply(_ incoming: [ProviderSnapshot], replacing: Bool) {
        if replacing {
            self.snapshots = incoming
            return
        }

        var merged = Dictionary(uniqueKeysWithValues: self.snapshots.compactMap { snapshot in
            snapshot.providerID.map { ($0, snapshot) }
        })
        for snapshot in incoming {
            if let provider = snapshot.providerID {
                merged[provider] = snapshot
            }
        }
        self.snapshots = ProviderID.allCases.compactMap { merged[$0] }
    }
}

@MainActor
@Observable
public final class RefreshStateStore {
    public var state: RefreshOperationState

    public init(state: RefreshOperationState = RefreshOperationState()) {
        self.state = state
    }

    public var isRefreshing: Bool {
        self.state.isRefreshing
    }

    public var refreshingProvider: ProviderID? {
        self.state.provider
    }
}

@MainActor
@Observable
public final class BrowserSessionStateStore {
    public var importedSessions: [ProviderID: ImportedBrowserSession]
    public var browserImportCandidates: [ProviderID: [BrowserSessionImportCandidate]]
    public var state: SessionImportOperationState

    public init(
        importedSessions: [ProviderID: ImportedBrowserSession] = [:],
        browserImportCandidates: [ProviderID: [BrowserSessionImportCandidate]] = [:],
        state: SessionImportOperationState = SessionImportOperationState()
    ) {
        self.importedSessions = importedSessions
        self.browserImportCandidates = browserImportCandidates
        self.state = state
    }

    public var isImportingSession: Bool {
        self.state.isActive
    }
}

@MainActor
@Observable
public final class IssueStore {
    public var issuesByKind: [AppIssueKind: AppIssue]

    public init(issuesByKind: [AppIssueKind: AppIssue] = [:]) {
        self.issuesByKind = issuesByKind
    }

    public var current: AppIssue? {
        self.issuesByKind.values.max(by: { $0.occurredAt < $1.occurredAt })
    }

    public func issue(for kind: AppIssueKind) -> AppIssue? {
        self.issuesByKind[kind]
    }

    public func record(_ issue: AppIssue) {
        self.issuesByKind[issue.kind] = issue
    }

    public func clear(kind: AppIssueKind) {
        self.issuesByKind.removeValue(forKey: kind)
    }

    public func clear(kinds: [AppIssueKind]) {
        for kind in kinds {
            self.issuesByKind.removeValue(forKey: kind)
        }
    }
}

@MainActor
@Observable
public final class ProviderRepository {
    public let content: ProviderContentStore
    public let refreshState: RefreshStateStore
    public let browserSessionState: BrowserSessionStateStore
    public let issues: IssueStore

    public init(
        content: ProviderContentStore = ProviderContentStore(),
        refreshState: RefreshStateStore = RefreshStateStore(),
        browserSessionState: BrowserSessionStateStore = BrowserSessionStateStore(),
        issues: IssueStore = IssueStore()
    ) {
        self.content = content
        self.refreshState = refreshState
        self.browserSessionState = browserSessionState
        self.issues = issues
    }

    public var snapshots: [ProviderSnapshot] {
        get { self.content.snapshots }
        set { self.content.snapshots = newValue }
    }

    public var adjunctSnapshots: [ProviderID: DashboardAdjunctSnapshot] {
        get { self.content.adjunctSnapshots.compactMapValues { $0 } }
        set { self.content.adjunctSnapshots = newValue.mapValues(Optional.some) }
    }

    public var syncedAggregate: SyncedAggregateEnvelope? {
        get { self.content.syncedAggregate }
        set { self.content.syncedAggregate = newValue }
    }

    public var cloudSyncState: CloudSyncSpaceState {
        get { self.content.cloudSyncState }
        set { self.content.cloudSyncState = newValue }
    }

    public var importedSessions: [ProviderID: ImportedBrowserSession] {
        get { self.browserSessionState.importedSessions }
        set { self.browserSessionState.importedSessions = newValue }
    }

    public var browserImportCandidates: [ProviderID: [BrowserSessionImportCandidate]] {
        get { self.browserSessionState.browserImportCandidates }
        set { self.browserSessionState.browserImportCandidates = newValue }
    }

    public var lastError: String? {
        get { self.currentIssue?.message }
        set {
            guard let newValue, !newValue.isEmpty else {
                self.issues.clear(kind: .refresh)
                self.issues.clear(kind: .browserImport)
                self.issues.clear(kind: .settingsSave)
                self.issues.clear(kind: .authRecovery)
                self.issues.clear(kind: .snapshotSync)
                self.issues.clear(kind: .widgetPersistence)
                self.issues.clear(kind: .helperStartup)
                return
            }
            self.recordIssue(AppIssue(kind: .refresh, message: newValue))
        }
    }

    public var currentIssue: AppIssue? {
        [
            self.refreshState.state.lastIssue,
            self.browserSessionState.state.lastIssue,
            self.issues.issue(for: .settingsSave),
            self.issues.issue(for: .authRecovery),
            self.issues.issue(for: .snapshotSync),
            self.issues.issue(for: .widgetPersistence),
            self.issues.issue(for: .localNotifications),
            self.issues.issue(for: .helperStartup),
        ]
        .compactMap { $0 }
        .max(by: { $0.occurredAt < $1.occurredAt })
    }

    public var isRefreshing: Bool {
        self.refreshState.isRefreshing
    }

    public var refreshActivity: RefreshActivity {
        get { self.refreshState.state.activity }
        set { self.refreshState.state.activity = newValue }
    }

    public var refreshingProvider: ProviderID? {
        self.refreshState.refreshingProvider
    }

    public var lastRefreshCompletedAt: Date? {
        get { self.refreshState.state.lastCompletedAt }
        set { self.refreshState.state.lastCompletedAt = newValue }
    }

    public var isImportingSession: Bool {
        self.browserSessionState.isImportingSession
    }

    public var sessionImportActivity: SessionImportActivity {
        get { self.browserSessionState.state.activity }
        set { self.browserSessionState.state.activity = newValue }
    }

    public func beginRefresh(provider: ProviderID?) {
        self.refreshState.state.activity = provider.map(RefreshActivity.refreshingProvider) ?? .refreshingAll
    }

    public func finishRefresh(issue: AppIssue?) {
        self.refreshState.state.activity = .idle
        self.refreshState.state.lastCompletedAt = Date()
        self.refreshState.state.lastIssue = issue
        if let issue {
            self.recordIssue(issue)
        } else {
            self.clearIssue(kind: .refresh)
            self.clearIssue(kind: .helperStartup)
        }
    }

    public func beginImport(provider: ProviderID, resetting: Bool) {
        self.browserSessionState.state.activity = resetting ? .resetting(provider) : .importing(provider)
    }

    public func finishImport(issue: AppIssue?) {
        self.browserSessionState.state.activity = .idle
        self.browserSessionState.state.lastIssue = issue
        if let issue {
            self.recordIssue(issue)
        } else {
            self.clearIssue(kind: .browserImport)
        }
    }

    public func recordIssue(_ issue: AppIssue) {
        self.issues.record(issue)
        switch issue.kind {
        case .refresh, .helperStartup:
            self.refreshState.state.lastIssue = issue
        case .browserImport:
            self.browserSessionState.state.lastIssue = issue
        case .settingsSave, .authRecovery, .snapshotSync, .widgetPersistence, .localNotifications:
            break
        }
    }

    public func setIssue(_ issue: AppIssue?) {
        guard let issue else {
            self.clearIssue(kind: .settingsSave)
            self.clearIssue(kind: .authRecovery)
            self.clearIssue(kind: .snapshotSync)
            return
        }
        self.recordIssue(issue)
    }

    public func issue(for provider: ProviderID?) -> AppIssue? {
        if let provider {
            if self.refreshState.state.lastIssue?.provider == provider {
                return self.refreshState.state.lastIssue
            }
            if self.browserSessionState.state.lastIssue?.provider == provider {
                return self.browserSessionState.state.lastIssue
            }
            return self.issues.issuesByKind.values
                .filter { $0.provider == provider }
                .max(by: { $0.occurredAt < $1.occurredAt })
        }
        return self.currentIssue
    }

    public func clearIssue(kind: AppIssueKind) {
        self.issues.clear(kind: kind)
        switch kind {
        case .refresh, .helperStartup:
            if self.refreshState.state.lastIssue?.kind == kind {
                self.refreshState.state.lastIssue = nil
            }
        case .browserImport:
            if self.browserSessionState.state.lastIssue?.kind == kind {
                self.browserSessionState.state.lastIssue = nil
            }
        case .settingsSave, .authRecovery, .snapshotSync, .widgetPersistence, .localNotifications:
            break
        }
    }

    public func snapshot(for provider: ProviderID) -> ProviderSnapshot? {
        self.content.snapshot(for: provider)
    }

    public func presentation(
        for provider: ProviderID,
        sessionStore: AppSessionStore
    ) -> ProviderPresentationState {
        SourceResolver.presentation(
            for: provider,
            config: sessionStore.config.providerConfig(for: provider),
            snapshot: self.snapshot(for: provider),
            adjunct: self.content.adjunctSnapshots[provider] ?? nil
        )
    }

    public var snapshotsByProvider: [ProviderID: ProviderSnapshot] {
        self.content.snapshotsByProvider
    }

    public func apply(_ incoming: [ProviderSnapshot], replacing: Bool) {
        self.content.apply(incoming, replacing: replacing)
    }

    public func setAdjunctSnapshot(_ snapshot: DashboardAdjunctSnapshot?, for provider: ProviderID) {
        self.content.adjunctSnapshots[provider] = snapshot
    }

    public func setSyncedAggregate(_ aggregate: SyncedAggregateEnvelope?) {
        self.content.syncedAggregate = aggregate
    }

    public func setCloudSyncState(_ state: CloudSyncSpaceState) {
        self.content.cloudSyncState = state
    }

    public func syncSelections(sessionStore: AppSessionStore) {
        if !sessionStore.visibleProviders.contains(sessionStore.selectedProvider) {
            sessionStore.selectedProvider = sessionStore.visibleProviders.first ?? .claude
        }
        if !sessionStore.visibleTabs.contains(sessionStore.selectedMergeTab) {
            sessionStore.selectedMergeTab = sessionStore.visibleTabs.first ?? .overview
        }
    }
}
