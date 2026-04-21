import Foundation

public enum WidgetSnapshotBuilder {
    public static func snapshot(
        providers: [ProviderID],
        snapshots: [ProviderID: ProviderSnapshot],
        adjuncts: [ProviderID: DashboardAdjunctSnapshot],
        config: HeimdallBarConfig,
        generatedAt: String
    ) -> WidgetSnapshot {
        let providerSnapshots = Dictionary(uniqueKeysWithValues: providers.map { provider in
            (
                provider.rawValue,
                self.providerSnapshot(
                    provider: provider,
                    config: config.providerConfig(for: provider),
                    snapshot: snapshots[provider],
                    adjunct: adjuncts[provider]
                )
            )
        })

        let issues = providers.isEmpty
            ? [WidgetSnapshotIssue(code: "no-providers", message: "No providers are enabled for widgets.", severity: .warning)]
            : []

        return WidgetSnapshot(
            generatedAt: generatedAt,
            defaultRefreshIntervalSeconds: config.refreshIntervalSeconds,
            providers: providerSnapshots,
            issues: issues
        )
    }

    public static func providerSnapshot(
        provider: ProviderID,
        config: ProviderConfig,
        snapshot: ProviderSnapshot?,
        adjunct: DashboardAdjunctSnapshot?
    ) -> WidgetProviderSnapshot {
        let presentation = SourceResolver.presentation(
            for: provider,
            config: config,
            snapshot: snapshot,
            adjunct: adjunct
        )
        let resolution = presentation.resolution
        let freshness = WidgetProviderFreshnessSnapshot(
            visualState: self.visualState(
                statusIndicator: snapshot?.status?.indicator,
                stale: snapshot?.stale ?? false,
                error: snapshot?.error
            ),
            available: snapshot?.available ?? false,
            stale: snapshot?.stale ?? false,
            lastRefreshAt: snapshot?.lastRefresh ?? adjunct?.lastUpdated,
            error: snapshot?.error,
            statusIndicator: snapshot?.status?.indicator,
            statusDescription: snapshot?.status?.description
        )

        let lanes = [
            self.laneSnapshot(slot: 0, title: "Session", window: presentation.primary),
            self.laneSnapshot(slot: 1, title: "Weekly", window: presentation.secondary),
            self.laneSnapshot(slot: 2, title: "Extra", window: presentation.tertiary),
        ].compactMap { $0 }

        let auth = snapshot.map { self.authSnapshot(from: $0.auth) }
        let cost = WidgetProviderCostSnapshot(
            todayTokens: snapshot?.costSummary.todayTokens ?? 0,
            todayCostUSD: snapshot?.costSummary.todayCostUSD ?? 0,
            last30DaysTokens: snapshot?.costSummary.last30DaysTokens ?? 0,
            last30DaysCostUSD: snapshot?.costSummary.last30DaysCostUSD ?? 0,
            daily: snapshot?.costSummary.daily ?? []
        )

        return WidgetProviderSnapshot(
            provider: provider,
            source: WidgetProviderSourceSnapshot(
                requested: resolution.requestedSource,
                effective: resolution.effectiveSource,
                detail: resolution.effectiveSourceDetail,
                usesFallback: resolution.usesFallback,
                isUnsupported: resolution.isUnsupported,
                usageAvailable: resolution.usageAvailable
            ),
            freshness: freshness,
            auth: auth,
            identity: self.redactedIdentity(snapshot?.identity),
            lanes: lanes,
            credits: presentation.displayCredits,
            cost: cost,
            issues: self.providerIssues(
                provider: provider,
                resolution: resolution,
                auth: auth,
                freshness: freshness
            ),
            adjunct: adjunct.map {
                WidgetProviderAdjunctSnapshot(
                    source: $0.source,
                    isLoginRequired: $0.isLoginRequired,
                    hasWebExtras: $0.webExtras != nil,
                    lastUpdatedAt: $0.lastUpdated
                )
            }
        )
    }

    private static func laneSnapshot(
        slot: Int,
        title: String,
        window: ProviderRateWindow?
    ) -> WidgetProviderLaneSnapshot? {
        guard let window else { return nil }
        return WidgetProviderLaneSnapshot(
            slot: slot,
            title: title,
            usedPercent: window.usedPercent,
            remainingPercent: max(0, 100 - window.usedPercent),
            resetsAt: window.resetsAt,
            resetsInMinutes: window.resetsInMinutes,
            windowMinutes: window.windowMinutes
        )
    }

    private static func authSnapshot(from auth: ProviderAuthHealth) -> WidgetProviderAuthSnapshot {
        WidgetProviderAuthSnapshot(
            loginMethod: auth.loginMethod,
            credentialBackend: auth.credentialBackend,
            authMode: auth.authMode,
            isAuthenticated: auth.isAuthenticated,
            isSourceCompatible: auth.isSourceCompatible,
            requiresRelogin: auth.requiresRelogin,
            diagnosticCode: auth.diagnosticCode,
            failureReason: auth.failureReason,
            lastValidatedAt: auth.lastValidatedAt
        )
    }

    private static func redactedIdentity(_ identity: ProviderIdentity?) -> ProviderIdentity? {
        guard let identity else { return nil }
        return ProviderIdentity(
            provider: identity.provider,
            accountEmail: nil,
            accountOrganization: nil,
            loginMethod: identity.loginMethod,
            plan: identity.plan
        )
    }

    private static func providerIssues(
        provider: ProviderID,
        resolution: ProviderSourceResolution,
        auth: WidgetProviderAuthSnapshot?,
        freshness: WidgetProviderFreshnessSnapshot
    ) -> [WidgetSnapshotIssue] {
        var issues = [WidgetSnapshotIssue]()

        if let error = freshness.error, !error.isEmpty {
            issues.append(WidgetSnapshotIssue(code: "refresh-error", message: error, severity: .error))
        }
        if freshness.visualState == .incident, let description = freshness.statusDescription {
            issues.append(WidgetSnapshotIssue(code: "incident", message: description, severity: .error))
        } else if freshness.visualState == .degraded, let description = freshness.statusDescription {
            issues.append(WidgetSnapshotIssue(code: "degraded", message: description, severity: .warning))
        }
        if freshness.stale {
            issues.append(WidgetSnapshotIssue(code: "stale", message: "Live provider data is stale.", severity: .warning))
        }
        if resolution.isUnsupported {
            issues.append(
                WidgetSnapshotIssue(
                    code: "unsupported-source",
                    message: "\(provider.title) does not support the selected \(resolution.requestedSource.rawValue) source.",
                    severity: .warning
                )
            )
        } else if resolution.requiresLogin {
            issues.append(
                WidgetSnapshotIssue(
                    code: "login-required",
                    message: auth?.failureReason ?? "A valid login is required before widget data can refresh.",
                    severity: .warning
                )
            )
        } else if let auth, !auth.isSourceCompatible {
            issues.append(
                WidgetSnapshotIssue(
                    code: "auth-incompatible",
                    message: auth.failureReason ?? "The current auth mode cannot satisfy the selected source.",
                    severity: .warning
                )
            )
        } else if !resolution.usageAvailable {
            issues.append(
                WidgetSnapshotIssue(
                    code: "source-unavailable",
                    message: resolution.explanation,
                    severity: .info
                )
            )
        }
        if resolution.usesFallback {
            issues.append(
                WidgetSnapshotIssue(
                    code: "fallback",
                    message: "The helper resolved this provider through a fallback source.",
                    severity: .info
                )
            )
        }

        return issues.uniqued(by: \.id)
    }

    private static func visualState(
        statusIndicator: String?,
        stale: Bool,
        error: String?
    ) -> ProviderVisualState {
        if error != nil {
            return .error
        }

        switch statusIndicator?.lowercased() {
        case "critical", "major":
            return .incident
        case "minor":
            return .degraded
        default:
            break
        }

        if stale {
            return .stale
        }

        return .healthy
    }
}

public enum WidgetSelection {
    public static func providerSnapshot(
        in snapshot: WidgetSnapshot,
        provider: ProviderID
    ) -> WidgetProviderSnapshot? {
        snapshot.providerSnapshot(for: provider)
    }

    public static func orderedProviders(in snapshot: WidgetSnapshot) -> [WidgetProviderSnapshot] {
        snapshot.allProviders.sorted { lhs, rhs in
            let lhsRank = self.severityRank(lhs.freshness.visualState)
            let rhsRank = self.severityRank(rhs.freshness.visualState)
            if lhsRank != rhsRank {
                return lhsRank < rhsRank
            }
            if lhs.cost.todayCostUSD != rhs.cost.todayCostUSD {
                return lhs.cost.todayCostUSD > rhs.cost.todayCostUSD
            }
            return lhs.provider.rawValue < rhs.provider.rawValue
        }
    }

    public static func cadenceSeconds(
        snapshot: WidgetSnapshot,
        provider: ProviderID?
    ) -> TimeInterval {
        let normalCadence = TimeInterval(max(900, min(1800, snapshot.defaultRefreshIntervalSeconds)))
        let mediumCadence = TimeInterval(min(max(420, snapshot.defaultRefreshIntervalSeconds), 900))
        let shortCadence = TimeInterval(300)

        let providerSnapshots = if let provider {
            self.providerSnapshot(in: snapshot, provider: provider).map { [$0] } ?? []
        } else {
            self.orderedProviders(in: snapshot)
        }

        if providerSnapshots.isEmpty || !snapshot.issues.isEmpty {
            return shortCadence
        }

        if providerSnapshots.contains(where: { provider in
            provider.auth?.requiresRelogin == true
                || provider.adjunct?.isLoginRequired == true
                || provider.issues.contains(where: { $0.code == "login-required" })
        }) {
            return shortCadence
        }

        if providerSnapshots.contains(where: { provider in
            provider.freshness.visualState == .incident
                || provider.freshness.visualState == .error
                || provider.freshness.visualState == .degraded
                || provider.freshness.visualState == .stale
                || !provider.source.usageAvailable
        }) {
            return mediumCadence
        }

        return normalCadence
    }

    private static func severityRank(_ state: ProviderVisualState) -> Int {
        switch state {
        case .error, .incident:
            return 0
        case .degraded:
            return 1
        case .stale:
            return 2
        case .refreshing:
            return 3
        case .healthy:
            return 4
        }
    }
}

private extension Array {
    func uniqued<T: Hashable>(by keyPath: KeyPath<Element, T>) -> [Element] {
        var seen = Set<T>()
        return self.filter { seen.insert($0[keyPath: keyPath]).inserted }
    }
}
