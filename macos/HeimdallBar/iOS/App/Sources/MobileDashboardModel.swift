import Foundation
import HeimdallDomain
import HeimdallServices
import Observation

@MainActor
@Observable
final class MobileDashboardModel {
    var aggregate: SyncedAggregateEnvelope?
    var isLoading = false
    var lastError: String?
    var selectedProvider: ProviderID = .claude

    private let store: any SnapshotSyncStore

    init(store: any SnapshotSyncStore) {
        self.store = store
    }

    var providerSnapshots: [ProviderSnapshot] {
        self.aggregate?.aggregateProviderViews.map(\.providerSnapshot) ?? []
    }

    var selectedProviderSnapshot: ProviderSnapshot? {
        self.providerSnapshots.first(where: { $0.providerID == self.selectedProvider })
            ?? self.providerSnapshots.first
    }

    var selectedHistorySeries: MobileProviderHistorySeries? {
        self.aggregate?.aggregateHistorySeries(for: self.selectedProvider)
            ?? self.aggregate?.aggregateHistory90d().first
    }

    var installations: [SyncedInstallationSnapshot] {
        self.aggregate?.installations ?? []
    }

    var hasSnapshot: Bool {
        self.aggregate != nil
    }

    var staleSnapshotWarning: String? {
        guard self.aggregate != nil else { return nil }
        return self.lastError
    }

    func load() async {
        self.isLoading = true
        defer { self.isLoading = false }

        do {
            self.aggregate = try await self.store.loadAggregateSnapshot()
            self.lastError = nil
            self.syncSelectedProvider()
        } catch {
            self.lastError = error.localizedDescription
        }
    }

    func acceptShareURL(_ url: URL) async {
        self.isLoading = true
        defer { self.isLoading = false }

        do {
            _ = try await self.store.acceptShareURL(url)
            self.aggregate = try await self.store.loadAggregateSnapshot()
            self.lastError = nil
            self.syncSelectedProvider()
        } catch {
            self.lastError = error.localizedDescription
        }
    }

    private func syncSelectedProvider() {
        guard let aggregate else { return }
        let availableProviders = aggregate.aggregateProviderViews.compactMap(\.providerID)
        if availableProviders.contains(self.selectedProvider) {
            return
        }
        self.selectedProvider = availableProviders.first ?? .claude
    }
}
