import Foundation

public enum SourceResolver {
    public static func resolve(
        provider: ProviderID,
        config: ProviderConfig,
        snapshot: ProviderSnapshot?,
        adjunct: DashboardAdjunctSnapshot?
    ) -> ProviderSourceResolution {
        let requestedSource = config.source
        let liveSource = normalizedSource(from: snapshot?.sourceUsed)
        let liveSourceDetail = snapshot?.sourceUsed
        let fallbackChain = fallbackChain(snapshot: snapshot, adjunct: adjunct)
        let auth = snapshot?.auth
        let webConfigured = config.dashboardExtrasEnabled
        let webSessionReady = adjunct != nil && adjunct?.isLoginRequired == false
        let webLoginRequired = adjunct?.isLoginRequired == true
        let webExtras = adjunct?.webExtras
        let webUsageAvailable = webExtras?.primaryLane != nil || webExtras?.secondaryLane != nil || webExtras?.tertiaryLane != nil

        switch requestedSource {
        case .auto:
            if let snapshot {
                var warnings = [String]()
                if snapshot.resolvedViaFallback {
                    warnings.append("Auto accepted helper fallback to \(snapshot.sourceUsed).")
                }
                return ProviderSourceResolution(
                    provider: provider,
                    requestedSource: requestedSource,
                    effectiveSource: liveSource,
                    effectiveSourceDetail: liveSourceDetail,
                    sourceLabel: "Source: auto -> \(liveSourceDetail ?? "unavailable")",
                    explanation: snapshotExplanation(snapshot, requested: requestedSource, matched: true),
                    warnings: warnings + authWarnings(for: auth, requestedSource: requestedSource),
                    fallbackChain: fallbackChain,
                    usageAvailable: snapshot.available && snapshot.primary != nil,
                    isUnsupported: false,
                    requiresLogin: authRequiresLogin(auth),
                    usesFallback: snapshot.resolvedViaFallback
                )
            }

            if webSessionReady {
                return ProviderSourceResolution(
                    provider: provider,
                    requestedSource: requestedSource,
                    effectiveSource: .web,
                    effectiveSourceDetail: "web",
                    sourceLabel: "Source: auto -> web",
                    explanation: webUsageAvailable
                        ? "No live helper snapshot is available, so HeimdallBar is using the cached web dashboard quotas."
                        : "No live helper snapshot is available. Web dashboard extras are available for credits and account context.",
                    warnings: [],
                    fallbackChain: fallbackChain,
                    usageAvailable: webUsageAvailable,
                    isUnsupported: false,
                    requiresLogin: false,
                    usesFallback: false
                )
            }

            return ProviderSourceResolution(
                provider: provider,
                requestedSource: requestedSource,
                effectiveSource: nil,
                effectiveSourceDetail: nil,
                sourceLabel: "Source: auto",
                explanation: "Waiting for a live snapshot from Heimdall.",
                warnings: [],
                fallbackChain: fallbackChain,
                usageAvailable: false,
                isUnsupported: false,
                requiresLogin: false,
                usesFallback: false
            )

        case .web:
            var warnings = [String]()
            if !self.supportedSources(for: provider).contains(.web) {
                warnings.append("\(provider.title) does not support web as a live source.")
                return unavailableResolution(
                    provider: provider,
                    requestedSource: requestedSource,
                    effectiveSource: liveSource,
                    effectiveSourceDetail: liveSourceDetail,
                    explanation: "Web is not a supported live source for \(provider.title).",
                    warnings: warnings,
                    fallbackChain: fallbackChain,
                    requiresLogin: false,
                    isUnsupported: true
                )
            }
            if !webConfigured {
                warnings.append("Enable dashboard extras before selecting the web source.")
                return unavailableResolution(
                    provider: provider,
                    requestedSource: requestedSource,
                    effectiveSource: nil,
                    effectiveSourceDetail: nil,
                    explanation: "Web source was requested, but dashboard extras are disabled.",
                    warnings: warnings,
                    fallbackChain: fallbackChain,
                    requiresLogin: false,
                    isUnsupported: false
                )
            }
            if webLoginRequired || adjunct == nil {
                warnings.append("Import a browser session to satisfy the web source requirement.")
                return unavailableResolution(
                    provider: provider,
                    requestedSource: requestedSource,
                    effectiveSource: nil,
                    effectiveSourceDetail: nil,
                    explanation: "Web source was requested, but no authenticated browser session is available.",
                    warnings: warnings,
                    fallbackChain: fallbackChain,
                    requiresLogin: true,
                    isUnsupported: false
                )
            }

            return ProviderSourceResolution(
                provider: provider,
                requestedSource: requestedSource,
                effectiveSource: .web,
                effectiveSourceDetail: "web",
                sourceLabel: "Source: web",
                explanation: webUsageAvailable
                    ? "Using hidden WebKit extraction for dashboard-only quotas and credits."
                    : "Using hidden WebKit extraction for dashboard-only credits and account context.",
                warnings: warnings,
                fallbackChain: fallbackChain,
                usageAvailable: webUsageAvailable,
                isUnsupported: false,
                requiresLogin: false,
                usesFallback: false
            )

        case .oauth, .cli:
            guard self.supportedSources(for: provider).contains(requestedSource) else {
                return unavailableResolution(
                    provider: provider,
                    requestedSource: requestedSource,
                    effectiveSource: liveSource,
                    effectiveSourceDetail: liveSourceDetail,
                    explanation: "\(provider.title) does not support \(requestedSource.rawValue) as a live source.",
                    warnings: ["Requested \(requestedSource.rawValue), but \(provider.title) has no matching live source."],
                    fallbackChain: fallbackChain,
                    requiresLogin: false,
                    isUnsupported: true
                )
            }

            guard let snapshot else {
                return unavailableResolution(
                    provider: provider,
                    requestedSource: requestedSource,
                    effectiveSource: nil,
                    effectiveSourceDetail: nil,
                    explanation: "No live snapshot is available for the requested \(requestedSource.rawValue) source.",
                    warnings: [],
                    fallbackChain: fallbackChain,
                    requiresLogin: false,
                    isUnsupported: false
                )
            }

            if liveSource == requestedSource, snapshot.available {
                return ProviderSourceResolution(
                    provider: provider,
                    requestedSource: requestedSource,
                    effectiveSource: liveSource,
                    effectiveSourceDetail: liveSourceDetail,
                    sourceLabel: "Source: \(liveSourceDetail ?? requestedSource.rawValue)",
                    explanation: snapshotExplanation(snapshot, requested: requestedSource, matched: true),
                    warnings: authWarnings(for: auth, requestedSource: requestedSource),
                    fallbackChain: fallbackChain,
                    usageAvailable: snapshot.primary != nil,
                    isUnsupported: false,
                    requiresLogin: authRequiresLogin(auth),
                    usesFallback: snapshot.resolvedViaFallback
                )
            }

            var warnings = [String]()
            let resolvedDetail = liveSourceDetail ?? "unavailable"
            // Only surface the 'resolved X' note when there's a real fallback
            // to report. When resolvedDetail == 'unavailable' the ERROR state
            // and top-level message already carry the info — the note just
            // becomes dev noise ('Requested oauth, but Heimdall resolved
            // unavailable.' below a big red ERROR badge).
            if resolvedDetail != "unavailable" {
                warnings.append("Requested \(requestedSource.rawValue), but Heimdall resolved \(resolvedDetail).")
            }
            if snapshot.resolvedViaFallback {
                warnings.append("The helper fell back during refresh, so the requested source is withheld instead of silently mixing sources.")
            }
            warnings.append(contentsOf: authWarnings(for: auth, requestedSource: requestedSource))
            return unavailableResolution(
                provider: provider,
                requestedSource: requestedSource,
                effectiveSource: liveSource,
                effectiveSourceDetail: liveSourceDetail,
                explanation: auth?.failureReason
                    ?? "The requested \(requestedSource.rawValue) source is not the active live snapshot.",
                warnings: warnings,
                fallbackChain: fallbackChain,
                requiresLogin: authRequiresLogin(auth),
                isUnsupported: false
            )
        }
    }

    public static func presentation(
        for provider: ProviderID,
        config: ProviderConfig,
        snapshot: ProviderSnapshot?,
        adjunct: DashboardAdjunctSnapshot?
    ) -> ProviderPresentationState {
        ProviderPresentationState(
            provider: provider,
            snapshot: snapshot,
            adjunct: adjunct,
            resolution: self.resolve(provider: provider, config: config, snapshot: snapshot, adjunct: adjunct)
        )
    }

    public static func supportedSources(for provider: ProviderID) -> [UsageSourcePreference] {
        switch provider {
        case .claude:
            return [.auto, .oauth, .web]
        case .codex:
            return [.auto, .oauth, .web, .cli]
        }
    }

    public static func normalizedSource(from raw: String?) -> UsageSourcePreference? {
        guard let raw else { return nil }
        if raw.hasPrefix("oauth") {
            return .oauth
        }
        if raw.hasPrefix("cli") {
            return .cli
        }
        if raw == "web" {
            return .web
        }
        return nil
    }

    private static func fallbackChain(
        snapshot: ProviderSnapshot?,
        adjunct: DashboardAdjunctSnapshot?
    ) -> [String] {
        var chain = snapshot?.sourceAttempts.map(\.source) ?? []
        if let sourceUsed = snapshot?.sourceUsed, !sourceUsed.isEmpty, !chain.contains(sourceUsed) {
            chain.append(sourceUsed)
        }
        if let adjunct, adjunct.source == .web, (!chain.contains("web") || adjunct.webExtras != nil) {
            chain.append("web")
        }
        return chain
    }

    private static func unavailableResolution(
        provider: ProviderID,
        requestedSource: UsageSourcePreference,
        effectiveSource: UsageSourcePreference?,
        effectiveSourceDetail: String?,
        explanation: String,
        warnings: [String],
        fallbackChain: [String],
        requiresLogin: Bool,
        isUnsupported: Bool
    ) -> ProviderSourceResolution {
        ProviderSourceResolution(
            provider: provider,
            requestedSource: requestedSource,
            effectiveSource: effectiveSource,
            effectiveSourceDetail: effectiveSourceDetail,
            sourceLabel: "Source: requested \(requestedSource.rawValue)",
            explanation: explanation,
            warnings: warnings,
            fallbackChain: fallbackChain,
            usageAvailable: false,
            isUnsupported: isUnsupported,
            requiresLogin: requiresLogin,
            usesFallback: effectiveSource != nil && effectiveSource != requestedSource
        )
    }

    private static func snapshotExplanation(
        _ snapshot: ProviderSnapshot,
        requested: UsageSourcePreference,
        matched: Bool
    ) -> String {
        let refreshLabel = snapshot.refreshDurationMs > 0 ? " in \(snapshot.refreshDurationMs)ms" : ""
        let sourceLabel = matched ? "Using \(requested.rawValue)" : "Resolved \(snapshot.sourceUsed)"
        // Only surface the 'after attempting X' fallback tail when we ACTUALLY
        // fell through from a different source. If the attempted source and
        // the winning source are the same (e.g. oauth -> oauth), the tail is
        // redundant noise — "Using oauth after attempting oauth in 576ms."
        if let attempted = snapshot.lastAttemptedSource,
           attempted != snapshot.sourceUsed {
            return "\(sourceLabel) after attempting \(attempted)\(refreshLabel)."
        }
        return "\(sourceLabel)\(refreshLabel)."
    }

    private static func authRequiresLogin(_ auth: ProviderAuthHealth?) -> Bool {
        guard let auth else { return false }
        return !auth.isAuthenticated || auth.requiresRelogin
    }

    private static func authWarnings(
        for auth: ProviderAuthHealth?,
        requestedSource: UsageSourcePreference
    ) -> [String] {
        guard let auth else { return [] }
        var warnings = [String]()
        if !auth.isSourceCompatible {
            warnings.append("Current auth does not satisfy the requested \(requestedSource.rawValue) source.")
        }
        if auth.requiresRelogin {
            warnings.append("Saved login is expired and needs re-authentication.")
        }
        return warnings
    }
}
