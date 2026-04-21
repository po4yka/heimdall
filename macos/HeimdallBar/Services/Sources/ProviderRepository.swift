import Foundation
import HeimdallDomain
import Observation

@MainActor
@Observable
public final class ProviderRepository {
    public var snapshots: [ProviderSnapshot]
    public var adjunctSnapshots: [ProviderID: DashboardAdjunctSnapshot]
    public var importedSessions: [ProviderID: ImportedBrowserSession]
    public var browserImportCandidates: [ProviderID: [BrowserSessionImportCandidate]]
    public var lastError: String?
    public var isRefreshing: Bool
    public var refreshingProvider: ProviderID?
    public var lastRefreshCompletedAt: Date?
    public var isImportingSession: Bool

    public init() {
        self.snapshots = []
        self.adjunctSnapshots = [:]
        self.importedSessions = [:]
        self.browserImportCandidates = [:]
        self.lastError = nil
        self.isRefreshing = false
        self.refreshingProvider = nil
        self.lastRefreshCompletedAt = nil
        self.isImportingSession = false
    }

    public func snapshot(for provider: ProviderID) -> ProviderSnapshot? {
        self.snapshots.first(where: { $0.providerID == provider })
    }

    public func presentation(
        for provider: ProviderID,
        sessionStore: AppSessionStore
    ) -> ProviderPresentationState {
        SourceResolver.presentation(
            for: provider,
            config: sessionStore.config.providerConfig(for: provider),
            snapshot: self.snapshot(for: provider),
            adjunct: self.adjunctSnapshots[provider]
        )
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

    public func syncSelections(sessionStore: AppSessionStore) {
        if !sessionStore.visibleProviders.contains(sessionStore.selectedProvider) {
            sessionStore.selectedProvider = sessionStore.visibleProviders.first ?? .claude
        }
        if !sessionStore.visibleTabs.contains(sessionStore.selectedMergeTab) {
            sessionStore.selectedMergeTab = sessionStore.visibleTabs.first ?? .overview
        }
    }
}
