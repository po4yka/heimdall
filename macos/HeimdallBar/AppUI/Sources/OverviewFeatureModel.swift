import Foundation
import HeimdallDomain
import HeimdallServices
import Observation

@MainActor
@Observable
public final class OverviewFeatureModel {
    private let sessionStore: AppSessionStore
    private let repository: ProviderRepository
    private let refreshCoordinator: RefreshCoordinator

    public init(
        sessionStore: AppSessionStore,
        repository: ProviderRepository,
        refreshCoordinator: RefreshCoordinator
    ) {
        self.sessionStore = sessionStore
        self.repository = repository
        self.refreshCoordinator = refreshCoordinator
    }

    public var visibleProviders: [ProviderID] {
        self.sessionStore.visibleProviders
    }

    public var projection: OverviewMenuProjection {
        MenuProjectionBuilder.overview(
            from: self.visibleProviders.map { provider in
                MenuProjectionBuilder.projection(
                    from: self.repository.presentation(for: provider, sessionStore: self.sessionStore),
                    config: self.sessionStore.config,
                    isRefreshing: self.repository.refreshActivity == .refreshingAll || self.repository.refreshActivity == .refreshingProvider(provider),
                    lastGlobalError: self.repository.issue(for: nil)?.message
                )
            },
            isRefreshing: self.repository.isRefreshing,
            lastGlobalError: self.repository.issue(for: nil)?.message
        )
    }

    public func refreshAll() async {
        await self.refreshCoordinator.refresh(force: true, provider: nil)
    }
}
