import Foundation

public enum WidgetProjectionBuilder {
    public static func snapshot(
        entries: [WidgetProviderEntry],
        refreshIntervalSeconds: Int,
        generatedAt: String
    ) -> WidgetSnapshot {
        WidgetSnapshot(
            generatedAt: generatedAt,
            refreshIntervalSeconds: refreshIntervalSeconds,
            entries: entries
        )
    }

    public static func entry(
        from projection: ProviderMenuProjection,
        costSummary: ProviderCostSummary
    ) -> WidgetProviderEntry {
        let warningLabel = projection.warningLabels.first
        let unavailableLabel = projection.laneDetails.allSatisfy { $0.remainingPercent == nil }
            ? projection.sourceExplanationLabel ?? "No usable provider data."
            : nil
        return WidgetProviderEntry(
            provider: projection.provider,
            title: projection.title,
            visualState: projection.visualState,
            statusLabel: projection.stateLabel,
            refreshLabel: projection.refreshStatusLabel,
            usageLines: projection.laneDetails.map { detail in
                WidgetUsageLine(
                    title: detail.title,
                    valueLabel: detail.remainingPercent.map { "\($0)%" } ?? "—",
                    detailLabel: [detail.paceLabel.map { "pace \($0.lowercased())" }, detail.resetDetail]
                        .compactMap { $0 }
                        .joined(separator: " · ")
                        .nilIfEmpty,
                    fraction: detail.remainingPercent.map { Double($0) / 100 }
                )
            },
            creditsLabel: projection.creditsLabel,
            warningLabel: warningLabel,
            unavailableLabel: unavailableLabel,
            loginRequired: projection.warningLabels.contains(where: { $0.localizedCaseInsensitiveContains("login") }),
            historyFractions: projection.historyFractions,
            costSummary: costSummary,
            todayCostLabel: String(format: "$%.2f today", costSummary.todayCostUSD),
            last30DaysCostLabel: String(format: "$%.2f in 30d", costSummary.last30DaysCostUSD),
            todayTokensLabel: costSummary.todayTokens > 0 ? "\(costSummary.todayTokens) tokens today" : "Tokens unavailable",
            activityLabel: projection.historyFractions.isEmpty ? "No recent activity" : "Recent activity available",
            sourceLabel: projection.sourceLabel,
            updatedAt: projection.lastRefreshLabel
        )
    }
}

public enum WidgetSelection {
    public static func providerEntry(
        in snapshot: WidgetSnapshot,
        provider: ProviderID
    ) -> WidgetProviderEntry? {
        snapshot.entries.first(where: { $0.provider == provider })
    }

    public static func cadenceSeconds(
        snapshot: WidgetSnapshot,
        provider: ProviderID
    ) -> TimeInterval {
        let providerEntry = self.providerEntry(in: snapshot, provider: provider)
        if providerEntry?.loginRequired == true {
            return 300
        }
        if providerEntry?.unavailableLabel != nil {
            return 420
        }
        return TimeInterval(max(300, min(1800, snapshot.refreshIntervalSeconds)))
    }
}

private extension String {
    var nilIfEmpty: String? {
        self.isEmpty ? nil : self
    }
}
