import Foundation

public enum MenuProjectionBuilder {
    public static func availableTabs(config: HeimdallBarConfig) -> [MergeMenuTab] {
        var tabs = [MergeMenuTab.overview]
        if config.claude.enabled {
            tabs.append(.claude)
        }
        if config.codex.enabled {
            tabs.append(.codex)
        }
        return tabs
    }

    public static func projection(
        from presentation: ProviderPresentationState,
        config: HeimdallBarConfig,
        isRefreshing: Bool,
        lastGlobalError: String?
    ) -> ProviderMenuProjection {
        let provider = presentation.provider
        let snapshot = presentation.snapshot
        let adjunct = presentation.adjunct
        let resolution = presentation.resolution
        let history = snapshot.map { historyFractions($0.costSummary.daily) } ?? []
        let statusLabel = snapshot?.status.map { status -> String in
            let normalized = status.indicator.lowercased().trimmingCharacters(in: .whitespaces)
            if normalized.isEmpty || normalized == "none" {
                return status.description
            }
            return "[\(status.indicator.uppercased())] \(status.description)"
        }
        let incidentLabel = statusLabel.flatMap { $0.contains("major") || $0.contains("critical") ? $0 : nil }
        let identityLabel = presentation.displayIdentityLabel
        let creditsLabel = presentation.displayCredits.map { value in
            resolution.effectiveSource == .web
                ? String(format: "Web credits: %.2f", value)
                : String(format: "Credits: %.2f", value)
        }
        let lanePrimary = presentation.primary
        let laneSecondary = presentation.secondary
        let laneTertiary = presentation.tertiary
        let auth = presentation.auth
        let hasCachedSnapshot = snapshot != nil
        let connectivityFailure = hasCachedSnapshot ? lastGlobalError : nil
        let effectiveError = snapshot?.error ?? (hasCachedSnapshot ? nil : lastGlobalError)
        let lastRefreshLabel = if let lastRefresh = snapshot?.lastRefresh {
            "Updated \(relativeLabel(lastRefresh))"
        } else if let lastUpdated = adjunct?.lastUpdated {
            "Web data \(relativeLabel(lastUpdated))"
        } else {
            "Waiting for data"
        }
        let costLabel = if let snapshot {
            String(
                format: "Today: $%.2f · 30d: $%.2f",
                snapshot.costSummary.todayCostUSD,
                snapshot.costSummary.last30DaysCostUSD
            )
        } else {
            "Today: unavailable"
        }
        let visualState = self.visualState(
            statusIndicator: snapshot?.status?.indicator,
            stale: snapshot?.stale ?? false,
            error: effectiveError,
            isRefreshing: isRefreshing
        )
        let displayState = self.displayState(
            from: visualState,
            showingCachedData: connectivityFailure?.isEmpty == false
        )
        let refreshStatusLabel = self.refreshStatusLabel(
            visualState: displayState,
            isRefreshing: isRefreshing,
            lastRefreshLabel: lastRefreshLabel,
            error: effectiveError,
            showingCachedData: connectivityFailure?.isEmpty == false
        )
        let authHeadline = auth.flatMap { self.authHeadline(for: $0) }
        let authDetail = auth.flatMap {
            self.authDetail(
                for: $0,
                requestedSource: resolution.requestedSource,
                provider: provider
            )
        }
        let laneDetails = [
            laneDetail(title: "Session", window: lanePrimary, config: config),
            laneDetail(title: "Weekly", window: laneSecondary, config: config),
        ] + (laneTertiary.map { [laneDetail(title: "Extra", window: $0, config: config)] } ?? [])
        var warningLabels = resolution.warnings + fallbackChainWarnings(resolution.fallbackChain)
        if let auth, !auth.isSourceCompatible {
            warningLabels.append("\(provider.title) auth cannot satisfy the selected \(resolution.requestedSource.rawValue) source.")
        }
        if let auth, auth.requiresRelogin {
            warningLabels.append("\(provider.title) requires re-login before live quota can refresh.")
        }
        // "Showing cached data" is already conveyed by the StateBadge; suppress the duplicate warning.
        warningLabels = warningLabels.uniqued()

        return ProviderMenuProjection(
            provider: provider,
            title: provider.title,
            sourceLabel: resolution.sourceLabel,
            sourceExplanationLabel: resolution.explanation,
            authHeadline: authHeadline,
            authDetail: authDetail,
            authDiagnosticCode: auth?.diagnosticCode,
            authSummaryLabel: presentation.authSummaryLabel,
            authRecoveryActions: auth?.recoveryActions ?? [],
            warningLabels: warningLabels,
            visualState: displayState,
            stateLabel: stateLabel(for: displayState),
            statusLabel: statusLabel,
            identityLabel: identityLabel,
            lastRefreshLabel: lastRefreshLabel,
            refreshStatusLabel: refreshStatusLabel,
            costLabel: costLabel,
            laneDetails: laneDetails,
            creditsLabel: creditsLabel,
            incidentLabel: incidentLabel,
            stale: snapshot?.stale ?? false,
            isShowingCachedData: connectivityFailure?.isEmpty == false,
            isRefreshing: isRefreshing,
            error: effectiveError,
            globalIssueLabel: connectivityFailure.map(self.presentableRefreshFailure),
            historyFractions: history,
            claudeFactors: snapshot?.claudeUsage?.factors ?? [],
            adjunct: adjunct
        )
    }

    public static func overview(
        from items: [ProviderMenuProjection],
        isRefreshing: Bool,
        lastGlobalError: String?
    ) -> OverviewMenuProjection {
        let refreshedLabel = items.map(\.lastRefreshLabel).first ?? "Waiting for data"
        let totalCost = items
            .compactMap { item in
                item.costLabel.split(separator: "·").first?
                    .replacingOccurrences(of: "Today: $", with: "")
                    .trimmingCharacters(in: .whitespaces)
            }
            .compactMap(Double.init)
            .reduce(0.0, +)
        let historyFractions = mergedHistory(items: items)
        let warningLabels = items
            .flatMap(\.warningLabels)
            .uniqued()
        let hottestProvider = items.max(by: { lhs, rhs in
            numericTodayCost(lhs.costLabel) < numericTodayCost(rhs.costLabel)
        })
        let activitySummaryLabel = if let hottestProvider {
            "Most active: \(hottestProvider.title) · \(hottestProvider.stateLabel.lowercased())"
        } else {
            "Waiting for provider activity"
        }
        let refreshStatusLabel: String
        let isShowingCachedData = lastGlobalError?.isEmpty == false && !items.isEmpty
        let globalIssueLabel: String?
        if isRefreshing {
            let providerNames = items.map(\.title).joined(separator: " + ")
            refreshStatusLabel = if providerNames.isEmpty {
                "Refreshing providers…"
            } else {
                "Refreshing \(providerNames)…"
            }
        } else if isShowingCachedData {
            refreshStatusLabel = "Showing cached data"
        } else if lastGlobalError?.isEmpty == false {
            refreshStatusLabel = "Refresh failed"
        } else {
            refreshStatusLabel = refreshedLabel
        }

        if let lastGlobalError, !lastGlobalError.isEmpty {
            // Always surface the specific reason. refreshStatusLabel already
            // communicates "Showing cached data" / "Refresh failed", so the
            // banner can stay focused on the 'why' instead of repeating the
            // cached-state tag.
            globalIssueLabel = self.presentableRefreshFailure(lastGlobalError)
        } else {
            globalIssueLabel = nil
        }

        return OverviewMenuProjection(
            items: items,
            combinedCostLabel: String(format: "Combined today: $%.2f", totalCost),
            refreshedAtLabel: refreshedLabel,
            activitySummaryLabel: activitySummaryLabel,
            historyFractions: historyFractions,
            warningLabels: warningLabels,
            isShowingCachedData: isShowingCachedData,
            isRefreshing: isRefreshing,
            refreshStatusLabel: refreshStatusLabel,
            globalIssueLabel: globalIssueLabel
        )
    }

    public static func menuTitle(
        for presentation: ProviderPresentationState?,
        provider: ProviderID?,
        config: HeimdallBarConfig
    ) -> String {
        guard let presentation, let snapshot = presentation.snapshot, let primary = presentation.primary else {
            if let presentation, let primary = presentation.primary {
                let value = config.showUsedValues ? primary.usedPercent : max(0, 100 - primary.usedPercent)
                let suffix = config.showUsedValues ? "used" : "left"
                let label = provider?.title ?? presentation.provider.title
                return "\(label) \(Int(value.rounded()))% \(suffix)"
            }
            if let presentation, presentation.resolution.requestedSource == .web {
                return provider?.title ?? presentation.provider.title
            }
            return provider?.title ?? presentation?.provider.title ?? "Heimdall"
        }

        let value = config.showUsedValues ? primary.usedPercent : max(0, 100 - primary.usedPercent)
        let suffix = config.showUsedValues ? "used" : "left"
        let label = provider?.title ?? snapshot.provider.capitalized
        return "\(label) \(Int(value.rounded()))% \(suffix)"
    }

    private static func laneDetail(
        title: String,
        window: ProviderRateWindow?,
        config: HeimdallBarConfig
    ) -> LaneDetailProjection {
        guard let window else {
            return LaneDetailProjection(
                title: title,
                summary: "\(title): unavailable",
                remainingPercent: nil,
                resetDetail: nil,
                paceLabel: nil
            )
        }

        let value = config.showUsedValues ? window.usedPercent : max(0, 100 - window.usedPercent)
        let remainingPercent = Int(max(0, 100 - window.usedPercent).rounded())
        let modeLabel = config.showUsedValues ? "used" : "left"
        let resetLabel: String
        switch config.resetDisplayMode {
        case .countdown:
            if let minutes = window.resetsInMinutes {
                resetLabel = "resets in \(Self.humanizeMinutes(minutes))"
            } else {
                resetLabel = window.resetLabel ?? "reset unknown"
            }
        case .absolute:
            resetLabel = window.resetsAt ?? window.resetLabel ?? "reset unknown"
        }
        let paceLabel = paceLabel(forRemainingPercent: remainingPercent)
        return LaneDetailProjection(
            title: title,
            summary: "\(title): \(Int(value.rounded()))% \(modeLabel) · pace \(paceLabel.lowercased()) · \(resetLabel)",
            remainingPercent: remainingPercent,
            resetDetail: resetLabel,
            paceLabel: paceLabel
        )
    }

    private static func relativeLabel(_ timestamp: String) -> String {
        guard let date = parseISO8601(timestamp) else { return "just now" }
        let delta = max(0, Int(Date().timeIntervalSince(date)))
        if delta < 60 {
            return "\(delta)s ago"
        }
        if delta < 3600 {
            return "\(delta / 60)m ago"
        }
        if delta < 86_400 {
            return "\(delta / 3600)h ago"
        }
        return "\(delta / 86_400)d ago"
    }

    private static func parseISO8601(_ timestamp: String) -> Date? {
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        if let date = formatter.date(from: timestamp) {
            return date
        }
        formatter.formatOptions = [.withInternetDateTime]
        return formatter.date(from: timestamp)
    }

    private static func historyFractions(_ points: [CostHistoryPoint]) -> [Double] {
        let recent = Array(points.suffix(7))
        let maxValue = recent.map(\.costUSD).max() ?? 0
        guard maxValue > 0 else { return recent.map { _ in 0 } }
        return recent.map { min(1, $0.costUSD / maxValue) }
    }

    private static func fallbackChainWarnings(_ chain: [String]) -> [String] {
        guard chain.count > 1 else { return [] }
        // Suppress the chain when the final step is 'unavailable' — the ERROR
        // badge and top message already tell the user the whole pipeline
        // failed. A chain like 'oauth -> cli-rpc' where we DID recover is
        // still worth showing.
        if chain.last == "unavailable" {
            return []
        }
        return ["Resolution chain: \(chain.joined(separator: " -> "))"]
    }

    private static func mergedHistory(items: [ProviderMenuProjection]) -> [Double] {
        guard let count = items.map(\.historyFractions.count).max(), count > 0 else { return [] }
        var merged = Array(repeating: 0.0, count: count)
        var totals = Array(repeating: 0, count: count)
        for item in items {
            for (index, value) in item.historyFractions.enumerated() {
                merged[index] += value
                totals[index] += 1
            }
        }
        return zip(merged, totals).map { total, count in
            guard count > 0 else { return 0 }
            return min(1, total / Double(count))
        }
    }

    private static func numericTodayCost(_ label: String) -> Double {
        guard let todayLabel = label.split(separator: "·").first else {
            return 0
        }
        let normalized = String(todayLabel)
            .replacingOccurrences(of: "Today: $", with: "")
            .trimmingCharacters(in: .whitespaces)
        return Double(normalized) ?? 0
    }

    private static func visualState(
        statusIndicator: String?,
        stale: Bool,
        error: String?,
        isRefreshing: Bool
    ) -> ProviderVisualState {
        if error?.isEmpty == false {
            return .error
        }
        if stale {
            return .stale
        }
        let indicator = statusIndicator?.lowercased() ?? ""
        if indicator.contains("critical") || indicator.contains("major") {
            return .incident
        }
        if indicator.contains("minor")
            || indicator.contains("degraded")
            || indicator.contains("maintenance")
            || indicator.contains("partial")
        {
            return .degraded
        }
        if isRefreshing {
            return .refreshing
        }
        return .healthy
    }

    private static func displayState(
        from visualState: ProviderVisualState,
        showingCachedData: Bool
    ) -> ProviderVisualState {
        guard showingCachedData else { return visualState }
        switch visualState {
        case .healthy, .refreshing:
            return .stale
        default:
            return visualState
        }
    }

    private static func stateLabel(for state: ProviderVisualState) -> String {
        switch state {
        case .healthy: return "Operational"
        case .refreshing: return "Refreshing"
        case .stale: return "Stale"
        case .degraded: return "Degraded"
        case .incident: return "Incident"
        case .error: return "Error"
        }
    }

    private static func refreshStatusLabel(
        visualState: ProviderVisualState,
        isRefreshing: Bool,
        lastRefreshLabel: String,
        error: String?,
        showingCachedData: Bool
    ) -> String {
        if isRefreshing {
            return "Refreshing…"
        }
        if showingCachedData {
            return "Showing cached data"
        }
        if let error, !error.isEmpty {
            return "Refresh failed"
        }
        switch visualState {
        case .incident:
            return "Incident active · \(lastRefreshLabel.lowercased())"
        case .degraded:
            return "Provider degraded · \(lastRefreshLabel.lowercased())"
        case .stale:
            return "Data is stale · \(lastRefreshLabel.lowercased())"
        default:
            return lastRefreshLabel
        }
    }

    private static func authHeadline(for auth: ProviderAuthHealth) -> String? {
        if let code = auth.diagnosticCode {
            switch code {
            case "authenticated-compatible":
                return nil
            case "authenticated-incompatible-source":
                return "Authenticated, but incompatible with selected source"
            case "expired-refreshable":
                return "Expired, but refreshable"
            case "requires-relogin":
                return "Expired and requires re-login"
            case "env-override":
                return "Environment override blocks subscription auth"
            case "keychain-unavailable":
                return "Credential store access is blocked"
            case "managed-policy":
                return "Managed policy blocks this auth mode"
            case "missing-credentials":
                return "No saved login was found"
            case "headless-oauth-env":
                return "Headless OAuth token is active"
            default:
                break
            }
        }
        if auth.isAuthenticated && auth.isSourceCompatible {
            return nil
        }
        if auth.isAuthenticated && !auth.isSourceCompatible {
            return "Authenticated, but incompatible with selected source"
        }
        if auth.requiresRelogin {
            return "Re-login required"
        }
        return auth.failureReason == nil ? nil : "Authentication needs attention"
    }

    private static func authDetail(
        for auth: ProviderAuthHealth,
        requestedSource: UsageSourcePreference,
        provider: ProviderID
    ) -> String? {
        if let failureReason = auth.failureReason, !failureReason.isEmpty {
            return failureReason
        }

        if auth.isAuthenticated && auth.isSourceCompatible && !auth.requiresRelogin {
            return nil
        }

        var fragments = [String]()
        if let loginMethod = auth.loginMethod?.replacingOccurrences(of: "-", with: " ") {
            fragments.append("Login: \(loginMethod.capitalized)")
        }
        if let backend = auth.credentialBackend {
            fragments.append("Store: \(backend.capitalized)")
        }
        if !auth.isSourceCompatible {
            fragments.append("Does not satisfy \(provider.title) \(requestedSource.rawValue) source")
        }
        if auth.requiresRelogin {
            fragments.append("Run login again to restore live quota")
        }

        return fragments.isEmpty ? nil : fragments.joined(separator: " · ")
    }

    private static func presentableRefreshFailure(_ error: String) -> String {
        let normalized = error.trimmingCharacters(in: .whitespacesAndNewlines)
        let lowercased = normalized.lowercased()
        if lowercased.contains("could not connect to the server")
            || lowercased.contains("failed to connect")
            || lowercased.contains("connection refused")
        {
            return "Cannot reach the local Heimdall server."
        }
        if lowercased.contains("timed out") {
            return "The local Heimdall server did not respond in time."
        }
        return normalized
    }
    /// Convert a minute count into a compact human-readable window:
    /// 45 -> "45m", 225 -> "3h 45m", 1440 -> "1d", 3945 -> "2d 17h".
    /// Keeps at most two magnitudes so the label stays short.
    static func humanizeMinutes(_ minutes: Int) -> String {
        if minutes < 60 {
            return "\(minutes)m"
        }
        if minutes < 1440 {
            let hours = minutes / 60
            let mins = minutes % 60
            return mins == 0 ? "\(hours)h" : "\(hours)h \(mins)m"
        }
        let days = minutes / 1440
        let hours = (minutes % 1440) / 60
        return hours == 0 ? "\(days)d" : "\(days)d \(hours)h"
    }

    private static func paceLabel(forRemainingPercent remainingPercent: Int) -> String {
        switch remainingPercent {
        case ..<15:
            return "Critical"
        case ..<35:
            return "Heavy"
        case ..<65:
            return "Steady"
        default:
            return "Comfortable"
        }
    }
}

private extension Array where Element: Hashable {
    func uniqued() -> [Element] {
        var seen = Set<Element>()
        return self.filter { seen.insert($0).inserted }
    }
}
