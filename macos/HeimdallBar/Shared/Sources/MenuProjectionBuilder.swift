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
        for provider: ProviderID,
        snapshot: ProviderSnapshot?,
        config: HeimdallBarConfig,
        adjunct: DashboardAdjunctSnapshot?
    ) -> ProviderMenuProjection {
        guard let snapshot else {
            return ProviderMenuProjection(
                provider: provider,
                title: provider.title,
                sourceLabel: "Source: unavailable",
                statusLabel: nil,
                identityLabel: nil,
                lastRefreshLabel: "Last refresh: waiting for data",
                costLabel: "Today: unavailable",
                laneSummaries: ["Session unavailable", "Weekly unavailable"],
                creditsLabel: nil,
                incidentLabel: nil,
                stale: true,
                error: nil,
                historyFractions: [],
                claudeFactors: [],
                adjunct: adjunct
            )
        }

        let history = historyFractions(snapshot.costSummary.daily)
        let statusLabel = snapshot.status.map { "[\($0.indicator.uppercased())] \($0.description)" }
        let incidentLabel = statusLabel.flatMap { $0.contains("major") || $0.contains("critical") ? $0 : nil }
        let identityLabel = identityLabel(snapshot.identity)
        let creditsLabel = snapshot.credits.map { String(format: "Credits: %.2f", $0) }

        return ProviderMenuProjection(
            provider: provider,
            title: provider.title,
            sourceLabel: "Source: \(snapshot.sourceUsed)",
            statusLabel: statusLabel,
            identityLabel: identityLabel,
            lastRefreshLabel: "Last refresh: \(relativeLabel(snapshot.lastRefresh))",
            costLabel: String(
                format: "Today: $%.2f · 30d: $%.2f",
                snapshot.costSummary.todayCostUSD,
                snapshot.costSummary.last30DaysCostUSD
            ),
            laneSummaries: [
                laneSummary(title: "Session", window: snapshot.primary, config: config),
                laneSummary(title: "Weekly", window: snapshot.secondary, config: config),
            ] + (snapshot.tertiary.map { [laneSummary(title: "Extra", window: $0, config: config)] } ?? []),
            creditsLabel: creditsLabel,
            incidentLabel: incidentLabel,
            stale: snapshot.stale,
            error: snapshot.error,
            historyFractions: history,
            claudeFactors: snapshot.claudeUsage?.factors ?? [],
            adjunct: adjunct
        )
    }

    public static func overview(from items: [ProviderMenuProjection]) -> OverviewMenuProjection {
        let refreshedLabel = items.map(\.lastRefreshLabel).first ?? "Last refresh: waiting for data"
        let totalCost = items
            .compactMap { item in
                item.costLabel.split(separator: "·").first?
                    .replacingOccurrences(of: "Today: $", with: "")
                    .trimmingCharacters(in: .whitespaces)
            }
            .compactMap(Double.init)
            .reduce(0.0, +)

        return OverviewMenuProjection(
            items: items,
            combinedCostLabel: String(format: "Combined today: $%.2f", totalCost),
            refreshedAtLabel: refreshedLabel
        )
    }

    public static func menuTitle(
        for snapshot: ProviderSnapshot?,
        provider: ProviderID?,
        config: HeimdallBarConfig
    ) -> String {
        guard let snapshot, let primary = snapshot.primary else {
            return provider?.title ?? "Heimdall"
        }

        let value = config.showUsedValues ? primary.usedPercent : max(0, 100 - primary.usedPercent)
        let suffix = config.showUsedValues ? "used" : "left"
        let label = provider?.title ?? snapshot.provider.capitalized
        return "\(label) \(Int(value.rounded()))% \(suffix)"
    }

    private static func laneSummary(
        title: String,
        window: ProviderRateWindow?,
        config: HeimdallBarConfig
    ) -> String {
        guard let window else {
            return "\(title): unavailable"
        }

        let value = config.showUsedValues ? window.usedPercent : max(0, 100 - window.usedPercent)
        let modeLabel = config.showUsedValues ? "used" : "left"
        let resetLabel: String
        switch config.resetDisplayMode {
        case .countdown:
            if let minutes = window.resetsInMinutes {
                resetLabel = "resets in \(minutes)m"
            } else {
                resetLabel = window.resetLabel ?? "reset unknown"
            }
        case .absolute:
            resetLabel = window.resetsAt ?? window.resetLabel ?? "reset unknown"
        }
        return "\(title): \(Int(value.rounded()))% \(modeLabel) · \(resetLabel)"
    }

    private static func relativeLabel(_ timestamp: String) -> String {
        let formatter = ISO8601DateFormatter()
        guard let date = formatter.date(from: timestamp) else { return timestamp }
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

    private static func identityLabel(_ identity: ProviderIdentity?) -> String? {
        guard let identity else { return nil }
        return [identity.accountEmail, identity.plan]
            .compactMap { $0 }
            .joined(separator: " · ")
    }

    private static func historyFractions(_ points: [CostHistoryPoint]) -> [Double] {
        let recent = Array(points.suffix(7))
        let maxValue = recent.map(\.costUSD).max() ?? 0
        guard maxValue > 0 else { return recent.map { _ in 0 } }
        return recent.map { min(1, $0.costUSD / maxValue) }
    }
}
