import Foundation
import HeimdallDomain
import HeimdallServices
import Observation
import os.log

@MainActor
@Observable
public final class OverviewFeatureModel {
    private let sessionStore: AppSessionStore
    private let repository: ProviderRepository
    private let refreshCoordinator: RefreshCoordinator

    private static let logger = Logger(subsystem: "dev.heimdall.HeimdallBar", category: "OverviewFeatureModel")

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
        let globalIssue = self.presentableGlobalIssue()
        return MenuProjectionBuilder.overview(
            from: self.visibleProviders.map { provider in
                MenuProjectionBuilder.projection(
                    from: self.repository.presentation(for: provider, sessionStore: self.sessionStore),
                    config: self.sessionStore.config,
                    isRefreshing: self.repository.refreshActivity == .refreshingAll || self.repository.refreshActivity == .refreshingProvider(provider),
                    lastGlobalError: globalIssue
                )
            },
            isRefreshing: self.repository.isRefreshing,
            lastGlobalError: globalIssue
        )
    }

    /// Strip developer-only diagnostics (widget-snapshot cache faults) from
    /// the user-facing overview banner — same pattern as ProviderFeatureModel
    /// and SettingsFeatureModel. The issue is still logged.
    private func presentableGlobalIssue() -> String? {
        guard let candidate = self.repository.issue(for: nil) else { return nil }
        if candidate.kind == .widgetPersistence {
            Self.logger.debug("Suppressing widgetPersistence issue from Overview UI: \(candidate.message)")
            return nil
        }
        return candidate.message
    }

    public func refreshAll() async {
        await self.refreshCoordinator.refresh(force: true, provider: nil)
    }
}
