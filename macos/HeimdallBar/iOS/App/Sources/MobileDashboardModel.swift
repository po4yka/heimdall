import Foundation
import HeimdallDomain
import HeimdallServices
import Observation

@MainActor
@Observable
final class MobileDashboardModel {
    var snapshot: MobileSnapshotEnvelope?
    var isLoading = false
    var lastError: String?
    var selectedProvider: ProviderID = .claude

    private let store: any SnapshotSyncStore

    init(store: any SnapshotSyncStore) {
        self.store = store
    }

    var providerSnapshots: [ProviderSnapshot] {
        self.snapshot?.providers ?? []
    }

    var selectedProviderSnapshot: ProviderSnapshot? {
        self.providerSnapshots.first(where: { $0.providerID == self.selectedProvider })
            ?? self.providerSnapshots.first
    }

    var selectedHistorySeries: MobileProviderHistorySeries? {
        self.snapshot?.history90d.first(where: { $0.providerID == self.selectedProvider })
            ?? self.snapshot?.history90d.first
    }

    var hasSnapshot: Bool {
        self.snapshot != nil
    }

    func load() async {
        self.isLoading = true
        defer { self.isLoading = false }

        do {
            let snapshot = try await self.store.loadLatestSnapshot()
            self.snapshot = snapshot
            self.lastError = nil
            self.syncSelectedProvider()
        } catch {
            self.lastError = error.localizedDescription
        }
    }

    private func syncSelectedProvider() {
        guard let snapshot else { return }
        let availableProviders = snapshot.providers.compactMap(\.providerID)
        if availableProviders.contains(self.selectedProvider) {
            return
        }
        self.selectedProvider = availableProviders.first ?? .claude
    }
}
