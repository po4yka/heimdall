import HeimdallDomain
import HeimdallServices

public struct LocalProviderDataSource: StartupOptimizedProviderDataSource {
    private let clientFactory: @Sendable (Int) -> any LiveProviderClient

    public init(clientFactory: @escaping @Sendable (Int) -> any LiveProviderClient) {
        self.clientFactory = clientFactory
    }

    public func fetchStartupSnapshots(config: HeimdallBarConfig) async throws -> ProviderSnapshotEnvelope {
        let client = self.clientFactory(config.helperPort)
        return try await client.fetchStartupSnapshots()
    }

    public func fetchSnapshots(
        config: HeimdallBarConfig,
        refresh: Bool,
        provider: ProviderID?
    ) async throws -> ProviderSnapshotEnvelope {
        let client = self.clientFactory(config.helperPort)
        if refresh {
            return try await client.refresh(provider: provider)
        }
        return try await client.fetchSnapshots()
    }

    public func fetchCostSummary(
        config: HeimdallBarConfig,
        provider: ProviderID
    ) async throws -> CostSummaryEnvelope {
        let client = self.clientFactory(config.helperPort)
        return try await client.fetchCostSummary(provider: provider)
    }
}
