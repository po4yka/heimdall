import HeimdallDomain
import SwiftUI

enum WidgetPalette {
    static let panel = Color.primary.opacity(0.08)
    static let muted = Color.primary.opacity(0.55)
    static let barTrack = Color.primary.opacity(0.14)
    static let barFill = Color.primary
    static let warning = Color.orange
}

struct WidgetUsageRow: Identifiable {
    let title: String
    let valueLabel: String
    let detailLabel: String?
    let fraction: Double

    var id: String { self.title }
}

struct WidgetProviderRenderModel: Identifiable {
    let provider: ProviderID
    let statusLabel: String
    let sourceLabel: String
    let refreshLabel: String
    let primaryMetric: String
    let primaryCaption: String
    let usageRows: [WidgetUsageRow]
    let todayCostLabel: String
    let last30DaysCostLabel: String
    let todayTokensLabel: String
    let activityLabel: String
    let creditsLabel: String?
    let warningLabel: String?
    let unavailableLabel: String?
    let authLabel: String?
    let historyFractions: [Double]
    let visualState: ProviderVisualState

    var id: String { self.provider.rawValue }
    var title: String { self.provider.title }
}

enum WidgetRenderModelBuilder {
    static func providerModel(from snapshot: WidgetProviderSnapshot) -> WidgetProviderRenderModel {
        let usageRows = snapshot.lanes.prefix(3).map { lane in
            WidgetUsageRow(
                title: lane.title,
                valueLabel: Self.percentLabel(lane.remainingPercent),
                detailLabel: Self.resetLabel(for: lane),
                fraction: max(0, min(1, lane.remainingPercent / 100))
            )
        }
        let authLabel = Self.authLabel(snapshot.auth)
        let topIssue = snapshot.issues.sorted { lhs, rhs in
            Self.severityRank(lhs.severity) > Self.severityRank(rhs.severity)
        }.first
        let unavailableLabel: String? = if usageRows.isEmpty {
            topIssue?.message ?? "No live widget data"
        } else {
            nil
        }
        let primaryMetric: String
        let primaryCaption: String
        if let firstRow = usageRows.first {
            primaryMetric = firstRow.valueLabel
            primaryCaption = firstRow.title
        } else if let credits = snapshot.credits {
            primaryMetric = Self.currencyLabel(credits)
            primaryCaption = "Credits"
        } else if snapshot.auth?.requiresRelogin == true || snapshot.adjunct?.isLoginRequired == true {
            primaryMetric = "Sign in"
            primaryCaption = "Authentication required"
        } else {
            primaryMetric = "Unavailable"
            primaryCaption = "Waiting for data"
        }

        return WidgetProviderRenderModel(
            provider: snapshot.provider,
            statusLabel: Self.statusLabel(for: snapshot.freshness.visualState),
            sourceLabel: Self.sourceLabel(snapshot.source),
            refreshLabel: Self.refreshLabel(snapshot.freshness.lastRefreshAt),
            primaryMetric: primaryMetric,
            primaryCaption: primaryCaption,
            usageRows: usageRows,
            todayCostLabel: "\(Self.currencyLabel(snapshot.cost.todayCostUSD)) today",
            last30DaysCostLabel: "\(Self.currencyLabel(snapshot.cost.last30DaysCostUSD)) in 30d",
            todayTokensLabel: snapshot.cost.todayTokens > 0 ? "\(snapshot.cost.todayTokens) tokens today" : "Tokens unavailable",
            activityLabel: Self.activityLabel(snapshot.cost.daily),
            creditsLabel: snapshot.credits.map(Self.currencyLabel),
            warningLabel: topIssue?.severity == .info ? nil : topIssue?.message,
            unavailableLabel: unavailableLabel,
            authLabel: authLabel,
            historyFractions: Self.historyFractions(snapshot.cost.daily),
            visualState: snapshot.freshness.visualState
        )
    }

    static func switcherModels(from snapshot: WidgetSnapshot) -> [WidgetProviderRenderModel] {
        WidgetSelection.orderedProviders(in: snapshot).map(self.providerModel(from:))
    }

    static func emptyStateModel(
        provider: ProviderID,
        message: String
    ) -> WidgetProviderRenderModel {
        WidgetProviderRenderModel(
            provider: provider,
            statusLabel: "WAITING",
            sourceLabel: "No source",
            refreshLabel: "Waiting for first sync",
            primaryMetric: "Unavailable",
            primaryCaption: message,
            usageRows: [],
            todayCostLabel: "$0.00 today",
            last30DaysCostLabel: "$0.00 in 30d",
            todayTokensLabel: "Tokens unavailable",
            activityLabel: "No recent activity",
            creditsLabel: nil,
            warningLabel: nil,
            unavailableLabel: message,
            authLabel: nil,
            historyFractions: [],
            visualState: .stale
        )
    }

    private static func sourceLabel(_ source: WidgetProviderSourceSnapshot) -> String {
        let requested = source.requested.rawValue.uppercased()
        guard let effective = source.effective else {
            return requested
        }
        let effectiveLabel = effective.rawValue.uppercased()
        return requested == effectiveLabel ? effectiveLabel : "\(requested)→\(effectiveLabel)"
    }

    private static func refreshLabel(_ isoTimestamp: String?) -> String {
        guard let isoTimestamp else { return "Waiting for first sync" }
        return "Updated \(Self.relativeLabel(isoTimestamp))"
    }

    private static func resetLabel(for lane: WidgetProviderLaneSnapshot) -> String? {
        if let minutes = lane.resetsInMinutes {
            return "Resets in \(minutes)m"
        }
        if let resetsAt = lane.resetsAt {
            return "Resets \(Self.relativeLabel(resetsAt))"
        }
        return nil
    }

    private static func percentLabel(_ value: Double) -> String {
        "\(Int(value.rounded()))%"
    }

    private static func currencyLabel(_ value: Double) -> String {
        String(format: "$%.2f", value)
    }

    private static func authLabel(_ auth: WidgetProviderAuthSnapshot?) -> String? {
        guard let auth else { return nil }
        let parts = [
            auth.loginMethod?.replacingOccurrences(of: "-", with: " ").capitalized,
            auth.credentialBackend?.capitalized,
        ].compactMap { $0 }
        return parts.isEmpty ? nil : parts.joined(separator: " · ")
    }

    private static func historyFractions(_ daily: [CostHistoryPoint]) -> [Double] {
        let recent = daily.suffix(7)
        guard let maxCost = recent.map(\.costUSD).max(), maxCost > 0 else {
            return Array(repeating: 0, count: recent.count)
        }
        return recent.map { min(1, $0.costUSD / maxCost) }
    }

    private static func activityLabel(_ daily: [CostHistoryPoint]) -> String {
        let recent = daily.suffix(7)
        let activeDays = recent.filter { $0.costUSD > 0 || $0.totalTokens > 0 }.count
        if activeDays == 0 {
            return "No recent activity"
        }
        return "\(activeDays)d active this week"
    }

    private static func relativeLabel(_ isoTimestamp: String) -> String {
        let formatter = ISO8601DateFormatter()
        guard let date = formatter.date(from: isoTimestamp) else {
            return "recently"
        }
        let seconds = max(0, Int(Date().timeIntervalSince(date)))
        if seconds < 60 {
            return "\(seconds)s ago"
        }
        if seconds < 3600 {
            return "\(seconds / 60)m ago"
        }
        if seconds < 86_400 {
            return "\(seconds / 3600)h ago"
        }
        return "\(seconds / 86_400)d ago"
    }

    private static func statusLabel(for state: ProviderVisualState) -> String {
        switch state {
        case .healthy:
            return "OPERATIONAL"
        case .refreshing:
            return "REFRESHING"
        case .stale:
            return "STALE"
        case .degraded:
            return "DEGRADED"
        case .incident:
            return "INCIDENT"
        case .error:
            return "ERROR"
        }
    }

    private static func severityRank(_ severity: WidgetIssueSeverity) -> Int {
        switch severity {
        case .info:
            return 0
        case .warning:
            return 1
        case .error:
            return 2
        }
    }
}

struct UsageBarView: View {
    let row: WidgetUsageRow

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(self.row.title)
                    .font(.caption)
                Spacer()
                Text(self.row.valueLabel)
                    .font(.caption.monospacedDigit())
            }
            GeometryReader { proxy in
                ZStack(alignment: .leading) {
                    Capsule().fill(WidgetPalette.barTrack)
                    Capsule()
                        .fill(WidgetPalette.barFill)
                        .frame(width: max(4, proxy.size.width * self.row.fraction))
                }
            }
            .frame(height: 5)
            if let detailLabel = self.row.detailLabel {
                Text(detailLabel)
                    .font(.caption2)
                    .foregroundStyle(WidgetPalette.muted)
            }
        }
    }
}

struct WidgetStatusChip: View {
    let label: String

    var body: some View {
        Text(self.label)
            .font(.caption2.monospaced())
            .padding(.horizontal, 6)
            .padding(.vertical, 3)
            .background(WidgetPalette.panel)
            .clipShape(Capsule())
    }
}

struct WidgetProviderHeaderView: View {
    let model: WidgetProviderRenderModel

    var body: some View {
        HStack(alignment: .top) {
            VStack(alignment: .leading, spacing: 2) {
                Text(self.model.title)
                    .font(.headline)
                Text(self.model.sourceLabel)
                    .font(.caption2)
                    .foregroundStyle(WidgetPalette.muted)
            }
            Spacer()
            WidgetStatusChip(label: self.model.statusLabel)
        }
    }
}

struct WidgetUnavailableCard: View {
    let model: WidgetProviderRenderModel

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            WidgetProviderHeaderView(model: self.model)
            Text(self.model.unavailableLabel ?? self.model.primaryCaption)
                .font(.caption)
            if let warningLabel = self.model.warningLabel {
                Text(warningLabel)
                    .font(.caption2)
                    .foregroundStyle(WidgetPalette.warning)
            }
            if let authLabel = self.model.authLabel {
                Text(authLabel)
                    .font(.caption2)
                    .foregroundStyle(WidgetPalette.muted)
            }
            Text(self.model.refreshLabel)
                .font(.caption2)
                .foregroundStyle(WidgetPalette.muted)
        }
    }
}

struct WidgetFailureStateView: View {
    let title: String
    let message: String

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(self.title)
                .font(.headline)
            Text(self.message)
                .font(.caption)
            Text("Widget data is waiting for a valid shared snapshot.")
                .font(.caption2)
                .foregroundStyle(WidgetPalette.muted)
        }
        .padding()
    }
}
