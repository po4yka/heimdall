import AppKit
import HeimdallDomain
import SwiftUI

struct RootMenuView: View {
    @Bindable var shell: AppShellModel
    @Bindable var overview: OverviewFeatureModel
    let providerModel: (ProviderID) -> ProviderFeatureModel
    let helperPort: Int
    let onQuit: () -> Void

    var body: some View {
        let overview = self.overview.projection

        VStack(alignment: .leading, spacing: 10) {
            MenuChromeHeader(
                title: "HeimdallBar",
                status: overview.refreshedAtLabel,
                isRefreshing: overview.isRefreshing,
                attentionLabel: self.attentionLabel(for: overview)
            )

            if let globalIssueLabel = overview.globalIssueLabel {
                GlobalIssueBanner(
                    message: globalIssueLabel,
                    detail: overview.isShowingCachedData ? "Showing last known provider data" : nil
                )
            }

            if self.shell.visibleTabs.count > 1 {
                MergeTabSwitcher(
                    tabs: self.shell.visibleTabs,
                    selection: Binding(
                        get: { self.shell.selectedMenuTab },
                        set: { self.shell.selectMenuTab($0) }
                    )
                )
            }

            if self.shell.selectedMenuTab == .overview {
                OverviewMenuCard(providerModel: self.providerModel, projection: overview)
            } else if let provider = self.shell.selectedMenuTab.providerID {
                let providerModel = self.providerModel(provider)
                ProviderMenuCard(providerModel: providerModel)
                SessionActionGroup(models: [providerModel])
            }

            Divider()

            MenuActionRow(
                shell: self.shell,
                overview: self.overview,
                providerModel: self.providerModel,
                helperPort: self.helperPort,
                tab: self.shell.selectedMenuTab,
                onQuit: self.onQuit
            )

            if self.shell.selectedMenuTab == .overview {
                SessionActionGroup(models: self.shell.visibleProviders.map(self.providerModel))
            }
        }
        .padding(10)
        .frame(width: 336)
    }

    private func attentionLabel(for overview: OverviewMenuProjection) -> String? {
        guard let item = overview.items.max(by: { self.severityRank(for: $0.visualState) < self.severityRank(for: $1.visualState) }),
              self.severityRank(for: item.visualState) > 0 else {
            return nil
        }
        return "Needs attention: \(item.title) \(item.stateLabel.lowercased())"
    }

    private func severityRank(for state: ProviderVisualState) -> Int {
        switch state {
        case .error: return 5
        case .incident: return 4
        case .degraded: return 3
        case .stale: return 2
        case .refreshing: return 1
        case .healthy: return 0
        }
    }
}

struct ProviderMenuView: View {
    @Bindable var model: ProviderFeatureModel
    let helperPort: Int
    let onQuit: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            MenuChromeHeader(
                title: self.model.provider.title,
                status: self.model.projection.lastRefreshLabel,
                isRefreshing: self.model.projection.isRefreshing,
                attentionLabel: self.providerAttentionLabel
            )
            if let globalIssueLabel = self.model.projection.globalIssueLabel {
                GlobalIssueBanner(
                    message: globalIssueLabel,
                    detail: self.model.projection.isShowingCachedData ? "Showing last known provider data" : nil
                )
            }
            ProviderMenuCard(providerModel: self.model)
            SessionActionGroup(models: [self.model])

            Divider()

            MenuActionRow(
                shell: nil,
                overview: nil,
                providerModel: { _ in self.model },
                helperPort: self.helperPort,
                tab: self.model.provider == .claude ? .claude : .codex,
                onQuit: self.onQuit
            )
        }
        .padding(12)
        .frame(width: 336)
    }

    private var providerAttentionLabel: String? {
        let projection = self.model.projection
        switch projection.visualState {
        case .healthy, .refreshing:
            return nil
        default:
            return "\(projection.title) is \(projection.stateLabel.lowercased())"
        }
    }
}

struct ProviderMenuCard: View {
    @Bindable var providerModel: ProviderFeatureModel

    var body: some View {
        let projection = self.projection
        VStack(alignment: .leading, spacing: 8) {
            HStack(alignment: .firstTextBaseline, spacing: 6) {
                Text(self.projection.title)
                    .font(.system(size: 13, weight: .semibold))
                StateBadge(state: self.projection.visualState, label: self.projection.stateLabel)
                if let plan = self.planBadgeLabel {
                    Text(plan.uppercased())
                        .font(.caption2.weight(.bold))
                        .tracking(0.5)
                        .padding(.horizontal, 6)
                        .padding(.vertical, 2)
                        .background(
                            RoundedRectangle(cornerRadius: 4, style: .continuous)
                                .fill(Color.accentColor.opacity(0.18))
                        )
                        .foregroundStyle(.primary)
                }
                Spacer(minLength: 0)
            }
            TopMetricRow(
                title: self.primaryMetricTitle,
                value: self.primaryMetricText,
                detail: self.secondaryMetricText
            )
            AuthStatusSection(model: self.providerModel, projection: projection)
            if projection.laneDetails.count > 1 {
                Divider()
                    .padding(.vertical, 1)
            }
            if let sourceExplanationLabel = projection.sourceExplanationLabel {
                Text(sourceExplanationLabel)
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
            ForEach(projection.warningLabels.prefix(2), id: \.self) { warning in
                Text(warning)
                    .font(.caption2)
                    .foregroundStyle(.orange)
            }
            // Only render identityLabel when it carries more than just the
            // plan tier (which is already shown as a header badge). An
            // identityLabel of 'email · plan' still renders so the user sees
            // which account is active.
            if let identityLabel = projection.identityLabel,
               identityLabel.contains("·") || identityLabel.contains("@") {
                Text(identityLabel)
                    .font(.caption)
            }
            if projection.laneDetails.count > 1 && !projection.isShowingCachedData {
                LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 8) {
                    ForEach(projection.laneDetails.dropFirst()) { detail in
                        LaneStatusCard(detail: detail)
                    }
                }
            }
            if !projection.historyFractions.isEmpty {
                HistoryBarStrip(fractions: projection.historyFractions)
            }
            Text(projection.costLabel)
                .font(.caption)
                .foregroundStyle(.secondary)
            if let creditsLabel = projection.creditsLabel {
                Text(creditsLabel)
                    .font(.caption)
            }
            if let statusLabel = projection.statusLabel {
                Text(statusLabel)
                    .font(.caption)
                    .foregroundStyle(.primary)
            }
            if let incidentLabel = projection.incidentLabel {
                Text(incidentLabel)
                    .font(.caption)
                    .foregroundStyle(.orange)
            }
            if !projection.claudeFactors.isEmpty {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Usage Factors")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    ForEach(projection.claudeFactors) { factor in
                        Text("\(factor.displayLabel): \(Int(factor.percent.rounded()))%")
                            .font(.caption2)
                    }
                }
            }
            if let adjunct = projection.adjunct {
                AdjunctSummaryCard(adjunct: adjunct)
            }
            if let error = projection.error {
                Text(error)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
        .padding(10)
        .menuCardBackground(opacity: self.cardBackgroundOpacity)
    }

    private var projection: ProviderMenuProjection {
        self.providerModel.projection
    }

    /// Extract the plan tier out of the projection's identityLabel for the
    /// header badge — identityLabel is either 'email · plan' or just 'plan'.
    private var planBadgeLabel: String? {
        guard let raw = self.projection.identityLabel,
              !raw.trimmingCharacters(in: .whitespaces).isEmpty
        else { return nil }
        let parts = raw.split(separator: "·").map {
            $0.trimmingCharacters(in: .whitespaces)
        }
        // 'email · plan' -> take the last part
        // 'plan' -> take the only part
        return parts.last
    }

    private var primaryMetricTitle: String {
        self.projection.laneDetails.first?.remainingPercent == nil ? "Availability" : "Session"
    }

    private var primaryMetricText: String {
        guard let primaryLane = self.projection.laneDetails.first else {
            return self.unavailableValueLabel
        }
        if let remaining = primaryLane.remainingPercent {
            return "\(remaining)%"
        }
        return self.unavailableValueLabel
    }

    private var secondaryMetricText: String {
        guard let primaryLane = self.projection.laneDetails.first else {
            return self.projection.costLabel
        }
        if let resetDetail = primaryLane.resetDetail, let paceLabel = primaryLane.paceLabel, primaryLane.remainingPercent != nil {
            return "\(resetDetail) · \(paceLabel.lowercased()) pace"
        }
        return self.unavailableDetailLabel
    }

    private var cardBackgroundOpacity: Double {
        switch self.projection.visualState {
        case .healthy:
            return 0.05
        case .refreshing:
            return 0.07
        case .stale:
            return 0.055
        case .degraded:
            return 0.11
        case .incident:
            return 0.14
        case .error:
            return 0.13
        }
    }

    private var unavailableValueLabel: String {
        let sourceLabel = self.projection.sourceLabel.lowercased()
        if sourceLabel.contains("oauth") {
            return "OAuth quota unavailable"
        }
        if sourceLabel.contains("web") {
            return "Web quota unavailable"
        }
        if sourceLabel.contains("cli") {
            return "CLI quota unavailable"
        }
        return "Live quota unavailable"
    }

    private var unavailableDetailLabel: String {
        let sourceLabel = self.projection.sourceLabel.lowercased()
        if self.projection.provider == .claude,
           sourceLabel.contains("oauth"),
           let credentialDetail = self.providerModel.missingCredentialDetail {
            return credentialDetail
        }
        return "Live \(self.projection.title) session not available"
    }

}

struct OverviewMenuCard: View {
    let providerModel: (ProviderID) -> ProviderFeatureModel
    let projection: OverviewMenuProjection

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            ForEach(self.sortedItems) { item in
                OverviewProviderCard(model: self.providerModel(item.provider), item: item)
            }
            OverviewSummaryCard(projection: self.projection)
        }
        .padding(10)
        .menuCardBackground(opacity: 0.03, cornerRadius: 14)
    }

    private var sortedItems: [ProviderMenuProjection] {
        self.projection.items.sorted {
            let lhs = self.severityRank(for: $0.visualState)
            let rhs = self.severityRank(for: $1.visualState)
            if lhs == rhs {
                return $0.title < $1.title
            }
            return lhs > rhs
        }
    }

    private func severityRank(for state: ProviderVisualState) -> Int {
        switch state {
        case .error: return 5
        case .incident: return 4
        case .degraded: return 3
        case .stale: return 2
        case .refreshing: return 1
        case .healthy: return 0
        }
    }
}

private struct OverviewSummaryCard: View {
    let projection: OverviewMenuProjection

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(alignment: .top, spacing: 12) {
                VStack(alignment: .leading, spacing: 6) {
                    Text("Overview")
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(.secondary)
                    Text(self.projection.combinedCostLabel)
                        .font(.system(size: 15, weight: .semibold))
                        .foregroundStyle(.primary)
                        .fixedSize(horizontal: false, vertical: true)
                    Text(self.projection.activitySummaryLabel)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .fixedSize(horizontal: false, vertical: true)
                }

                Spacer(minLength: 8)

                VStack(alignment: .leading, spacing: 8) {
                    if !self.projection.historyFractions.isEmpty {
                        VStack(alignment: .leading, spacing: 5) {
                            Text("Last 7 days")
                                .font(.caption2.weight(.semibold))
                                .foregroundStyle(.secondary)
                            HistoryBarStrip(fractions: self.projection.historyFractions)
                        }
                    }

                    HStack(spacing: 6) {
                        OverviewMetaBadge(
                            title: "Providers",
                            value: "\(self.projection.items.count)"
                        )
                        if !self.projection.warningLabels.isEmpty {
                            OverviewMetaBadge(
                                title: "Flags",
                                value: "\(self.projection.warningLabels.count)"
                            )
                        }
                    }
                }
                .frame(minWidth: 92, alignment: .leading)
            }

            if !self.projection.warningLabels.isEmpty {
                Divider()
                Label("Source limits affect some provider data", systemImage: "exclamationmark.triangle.fill")
                    .font(.caption2.weight(.medium))
                    .foregroundStyle(.orange)
            }
        }
        .padding(10)
        .frame(maxWidth: .infinity, alignment: .leading)
        .menuCardBackground(opacity: 0.02, cornerRadius: 12)
    }
}

private struct OverviewMetaBadge: View {
    let title: String
    let value: String

    var body: some View {
        VStack(alignment: .leading, spacing: 1) {
            Text(self.title)
                .font(.system(size: 9, weight: .semibold))
                .foregroundStyle(.secondary)
                .textCase(.uppercase)
            Text(self.value)
                .font(.caption.monospacedDigit().weight(.semibold))
                .foregroundStyle(.primary)
        }
        .padding(.horizontal, 8)
        .padding(.vertical, 6)
        .frame(minWidth: 40, alignment: .leading)
        .menuCardBackground(opacity: 0.04, cornerRadius: 8)
    }
}

private struct MenuChromeHeader: View {
    let title: String
    let status: String
    let isRefreshing: Bool
    let attentionLabel: String?

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack(alignment: .firstTextBaseline) {
                Text(self.title)
                    .font(.system(size: 15, weight: .semibold))
                Spacer()
                if self.isRefreshing {
                    ProgressView()
                        .controlSize(.small)
                } else {
                    Text(self.status)
                        .font(.caption2.monospacedDigit().weight(.medium))
                        .foregroundStyle(.secondary)
                }
            }
            if let attentionLabel {
                Label(attentionLabel, systemImage: "exclamationmark.circle.fill")
                    .font(.caption2.weight(.semibold))
                    .foregroundStyle(self.attentionColor)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(
                        Capsule(style: .continuous)
                            .fill(self.attentionColor.opacity(0.12))
                    )
            }
        }
    }

    private var attentionColor: Color {
        self.attentionLabel == nil ? .secondary : .orange
    }
}

private struct TopMetricRow: View {
    let title: String
    let value: String
    let detail: String

    var body: some View {
        HStack(alignment: .firstTextBaseline) {
            VStack(alignment: .leading, spacing: 3) {
                Text(self.title)
                    .font(.caption.weight(.semibold))
                    .textCase(.uppercase)
                    .foregroundStyle(.secondary)
                Text(self.detail)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            Spacer(minLength: 12)
            Text(self.value)
                .font(.system(size: 18, weight: .bold, design: .rounded).monospacedDigit())
                .foregroundStyle(.primary)
        }
    }
}

private struct StateBadge: View {
    let state: ProviderVisualState
    let label: String

    var body: some View {
        Text(self.label.uppercased())
            .font(.caption2.monospaced())
            .padding(.horizontal, 7)
            .padding(.vertical, 3)
            .foregroundStyle(self.foregroundColor)
            .background(self.backgroundColor)
            .clipShape(Capsule())
    }

    private var foregroundColor: Color {
        switch self.state {
        case .incident, .error:
            return .white
        case .stale, .degraded:
            return .orange
        default:
            return .primary
        }
    }

    private var backgroundColor: Color {
        switch self.state {
        case .healthy:
            return Color.primary.opacity(0.08)
        case .refreshing:
            return Color.blue.opacity(0.14)
        case .stale:
            return Color.orange.opacity(0.12)
        case .degraded:
            return Color.orange
                .opacity(0.18)
        case .incident:
            return Color.red.opacity(0.85)
        case .error:
            return Color.red
        }
    }
}

private struct LaneStatusCard: View {
    let detail: LaneDetailProjection

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(self.detail.title)
                    .font(.caption.weight(.semibold))
                Spacer()
                if let remainingPercent = self.detail.remainingPercent {
                    Text("\(remainingPercent)% left")
                        .font(.caption.monospacedDigit().weight(.semibold))
                        .foregroundStyle(Self.paceTint(remainingPercent: remainingPercent))
                }
            }
            if let remainingPercent = self.detail.remainingPercent {
                GeometryReader { geo in
                    ZStack(alignment: .leading) {
                        RoundedRectangle(cornerRadius: 2, style: .continuous)
                            .fill(Color.primary.opacity(0.1))
                            .frame(height: 4)
                        RoundedRectangle(cornerRadius: 2, style: .continuous)
                            .fill(Self.paceTint(remainingPercent: remainingPercent))
                            .frame(
                                width: geo.size.width
                                    * CGFloat(max(0, min(100, 100 - remainingPercent)))
                                    / 100,
                                height: 4
                            )
                    }
                }
                .frame(height: 4)
            }
            // Fold pace + reset into one compact secondary line — they're
            // both descriptors of the same lane, no need to burn two rows.
            Text(self.footerLabel)
                .font(.caption2)
                .foregroundStyle(.secondary)
        }
        .padding(8)
        .frame(maxWidth: .infinity, alignment: .leading)
        .menuCardBackground(opacity: 0.045, cornerRadius: 8)
    }

    /// One-line footer that merges pace + reset, using the dot separator
    /// consistent with the rest of the dashboard. Either half can be absent.
    private var footerLabel: String {
        let pace = self.detail.paceLabel.map { "Pace \($0.lowercased())" }
        return [pace, self.detail.resetDetail]
            .compactMap { $0 }
            .joined(separator: " · ")
    }

    /// Tint the progress bar by pace: comfortable green, heavy orange,
    /// critical red. Tracks the paceLabel(forRemainingPercent:) thresholds
    /// used in MenuProjectionBuilder.
    private static func paceTint(remainingPercent: Int) -> Color {
        switch remainingPercent {
        case ..<15: return .red
        case ..<35: return .orange
        default: return .green
        }
    }
}

struct HistoryBarStrip: View {
    let fractions: [Double]

    var body: some View {
        let labels = Self.dayLabels(count: self.fractions.count)
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                Text("Last \(self.fractions.count) days")
                    .font(.caption2.weight(.semibold))
                    .foregroundStyle(.secondary)
                    .textCase(.uppercase)
                    .tracking(0.4)
                Spacer()
            }
            HStack(alignment: .bottom, spacing: 6) {
                ForEach(Array(zip(self.fractions, labels).enumerated()), id: \.offset) { entry in
                    let (fraction, label) = entry.element
                    // Today is the last slot — highlight it with the accent tint.
                    let isToday = entry.offset == self.fractions.count - 1
                    VStack(spacing: 3) {
                        ZStack(alignment: .bottom) {
                            RoundedRectangle(cornerRadius: 2, style: .continuous)
                                .fill(Color.primary.opacity(0.06))
                                .frame(height: 28)
                            RoundedRectangle(cornerRadius: 2, style: .continuous)
                                .fill(isToday ? Color.accentColor : Color.primary.opacity(0.7))
                                .frame(height: max(2, 28 * fraction))
                        }
                        .frame(maxWidth: .infinity)
                        Text(label)
                            .font(.system(size: 9).monospacedDigit())
                            .foregroundStyle(.secondary)
                    }
                }
            }
        }
        .padding(8)
        .menuCardBackground(opacity: 0.03, cornerRadius: 8)
    }

    /// Produce day-of-week labels for the last N days ending today, so the
    /// rightmost label is today's weekday.
    private static func dayLabels(count: Int) -> [String] {
        guard count > 0 else { return [] }
        let formatter = DateFormatter()
        formatter.dateFormat = "EEE"
        let calendar = Calendar.current
        let today = Date()
        return (0..<count).map { offset in
            let date = calendar.date(byAdding: .day, value: -(count - 1 - offset), to: today) ?? today
            return formatter.string(from: date)
        }
    }
}

private struct MenuActionRow: View {
    let shell: AppShellModel?
    let overview: OverviewFeatureModel?
    let providerModel: (ProviderID) -> ProviderFeatureModel
    let helperPort: Int
    let tab: MergeMenuTab
    let onQuit: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Button(action: {
                Task {
                    if let provider = self.primaryRefreshProvider {
                        await self.providerModel(provider).refresh()
                    } else if let overview {
                        await overview.refreshAll()
                    }
                }
            }) {
                HStack {
                    Image(systemName: "arrow.clockwise")
                    Text(self.primaryRefreshTitle)
                    Spacer()
                    if self.primaryRefreshProvider == nil {
                        Text("⌘R")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
            }
            .buttonStyle(PrimaryDashboardButtonStyle())
            .keyboardShortcut("r", modifiers: .command)
            .disabled(self.isRefreshing)

            if self.tab.providerID != nil {
                Button(action: {
                    Task { await self.overview?.refreshAll() }
                }) {
                    SecondaryActionLabel(title: "Refresh All", systemImage: "arrow.clockwise")
                }
                .buttonStyle(SecondaryDashboardButtonStyle())
                .disabled(self.isRefreshing)
            }

            VStack(spacing: 6) {
                Button(action: {
                    if let url = URL(string: "http://127.0.0.1:\(self.helperPort)") {
                        NSWorkspace.shared.open(url)
                    }
                }) {
                    SecondaryActionLabel(title: "Open Dashboard", systemImage: "safari")
                }
                .buttonStyle(SecondaryDashboardButtonStyle())

                SettingsLink {
                    SecondaryActionLabel(title: "Open Settings", systemImage: "gearshape")
                }
                .buttonStyle(SecondaryDashboardButtonStyle())
            }

            Divider()
                .padding(.top, 2)

            Button(action: self.onQuit) {
                Text("Quit")
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
            .buttonStyle(.plain)
        }
    }

    private var primaryRefreshTitle: String {
        if self.isRefreshing {
            if let provider = self.primaryRefreshProvider {
                return "Refreshing \(provider.title)…"
            }
            let names = self.visibleProviders.map(\.title).joined(separator: " + ")
            return names.isEmpty ? "Refreshing…" : "Refreshing \(names)…"
        }
        if let provider = self.tab.providerID {
            return "Refresh \(provider.title)"
        }
        return "Refresh All"
    }

    private var primaryRefreshProvider: ProviderID? {
        self.tab.providerID
    }

    private var visibleProviders: [ProviderID] {
        self.shell?.visibleProviders ?? self.primaryRefreshProvider.map { [$0] } ?? []
    }
    private var isRefreshing: Bool {
        if let provider = self.primaryRefreshProvider {
            return self.providerModel(provider).isRefreshing
        }
        return self.overview?.projection.isRefreshing ?? false
    }
}

struct SessionActionGroup: View {
    let models: [ProviderFeatureModel]

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("Web Sessions")
                .font(.caption)
                .foregroundStyle(.secondary)
            ForEach(self.models, id: \.provider) { providerModel in
                Menu {
                    if providerModel.importCandidates.isEmpty {
                        Text("No browser stores found")
                    } else {
                        ForEach(providerModel.importCandidates) { candidate in
                            Button("Import \(candidate.title)") {
                                Task { await providerModel.importBrowserSession(candidate: candidate) }
                            }
                        }
                    }
                    if providerModel.importedSession != nil {
                        Divider()
                        Button("Reset Stored Session") {
                            Task { await providerModel.resetBrowserSession() }
                        }
                    }
                } label: {
                    SessionDisclosureRow(
                        title: "\(providerModel.provider.title) Web Session",
                        subtitle: self.sessionHealthLabel(for: providerModel)
                    )
                }
                .menuStyle(.borderlessButton)
            }
        }
        .disabled(self.models.contains { $0.isImportingSession })
    }

    private func sessionHealthLabel(for providerModel: ProviderFeatureModel) -> String {
        guard let session = providerModel.importedSession else {
            return "Missing"
        }
        if session.expired {
            return "Expired"
        }
        if session.loginRequired {
            return "Login required"
        }
        return "Connected"
    }
}

struct AdjunctSummaryCard: View {
    let adjunct: DashboardAdjunctSnapshot

    var body: some View {
        VStack(alignment: .leading, spacing: 3) {
            Text(self.adjunct.headline)
                .font(.caption)
            ForEach(self.adjunct.detailLines, id: \.self) { line in
                Text(line)
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
            if let statusText = self.adjunct.statusText {
                Text("Web extras: \(statusText)")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
        }
        .padding(8)
        .menuCardBackground(opacity: 0.06, cornerRadius: 8)
    }
}

private struct MergeTabSwitcher: View {
    let tabs: [MergeMenuTab]
    @Binding var selection: MergeMenuTab

    var body: some View {
        HStack(spacing: 4) {
            ForEach(self.tabs) { tab in
                Button {
                    self.selection = tab
                } label: {
                    Text(tab.title)
                        .font(.caption.weight(.semibold))
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 7)
                }
                .buttonStyle(.plain)
                .foregroundStyle(self.selection == tab ? Color.white : Color.secondary)
                .background(
                    RoundedRectangle(cornerRadius: 9, style: .continuous)
                        .fill(self.selection == tab ? Color.accentColor.opacity(0.9) : Color.clear)
                )
            }
        }
        .padding(3)
        .background(
            RoundedRectangle(cornerRadius: 11, style: .continuous)
                .fill(Color.primary.opacity(0.04))
        )
        .overlay(
            RoundedRectangle(cornerRadius: 11, style: .continuous)
                .stroke(Color.primary.opacity(0.08), lineWidth: 1)
        )
    }
}

struct OverviewProviderCard: View {
    @Bindable var model: ProviderFeatureModel
    let item: ProviderMenuProjection

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(alignment: .firstTextBaseline) {
                Text(self.item.title)
                    .font(.headline)
                StateBadge(state: self.item.visualState, label: self.item.stateLabel)
                Spacer()
            }
            TopMetricRow(
                title: self.metricTitle,
                value: self.metricValue,
                detail: self.metricDetail
            )
            Text(self.item.costLabel)
                .font(.caption)
                .foregroundStyle(.secondary)
            if let summaryNote = self.summaryNote {
                Text(summaryNote)
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.secondary)
            }
        }
        .padding(10)
        .menuCardBackground(opacity: 0.045)
        .overlay(
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .stroke(self.highlightColor, lineWidth: self.highlightLineWidth)
        )
    }

    private var metricTitle: String {
        self.item.laneDetails.first?.remainingPercent == nil ? "Availability" : "Session"
    }

    private var metricValue: String {
        if let remaining = self.item.laneDetails.first?.remainingPercent {
            return "\(remaining)%"
        }
        return self.unavailableValueLabel
    }

    private var metricDetail: String {
        guard let lane = self.item.laneDetails.first else {
            return "No live session data"
        }
        if let reset = lane.resetDetail, lane.remainingPercent != nil {
            return reset
        }
        return self.unavailableDetailLabel
    }

    private var highlightColor: Color {
        switch self.item.visualState {
        case .error:
            return .red.opacity(0.45)
        case .incident, .degraded, .stale:
            return .orange.opacity(0.45)
        default:
            return .clear
        }
    }

    private var highlightLineWidth: CGFloat {
        switch self.item.visualState {
        case .error, .incident, .degraded, .stale:
            return 1.5
        default:
            return 0
        }
    }

    private var unavailableValueLabel: String {
        let sourceLabel = self.item.sourceLabel.lowercased()
        if sourceLabel.contains("oauth") {
            return "OAuth quota unavailable"
        }
        if sourceLabel.contains("web") {
            return "Web quota unavailable"
        }
        if sourceLabel.contains("cli") {
            return "CLI quota unavailable"
        }
        return "Live quota unavailable"
    }

    private var unavailableDetailLabel: String {
        let sourceLabel = self.item.sourceLabel.lowercased()
        if self.item.provider == .claude,
           sourceLabel.contains("oauth"),
           let credentialDetail = self.model.missingCredentialDetail {
            return credentialDetail
        }
        if self.item.isShowingCachedData {
            return "Showing last known provider data"
        }
        return "Live \(self.item.title) session not available"
    }

    private var summaryNote: String? {
        self.item.authHeadline
    }

}

struct AuthStatusSection: View {
    @Bindable var model: ProviderFeatureModel
    let projection: ProviderMenuProjection

    var body: some View {
        let isHealthy = (self.projection.authDiagnosticCode ?? "")
            .hasPrefix("authenticated")
        if self.projection.authHeadline != nil
            || self.projection.authDetail != nil
            || !self.projection.authRecoveryActions.isEmpty
        {
            VStack(alignment: .leading, spacing: 6) {
                HStack(alignment: .firstTextBaseline) {
                    Text("Auth")
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(.secondary)
                    if isHealthy {
                        Image(systemName: "checkmark.circle.fill")
                            .font(.caption2)
                            .foregroundStyle(.green)
                    }
                    Spacer()
                    if let summary = self.projection.authSummaryLabel {
                        Text(summary)
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                    }
                }
                if let headline = self.projection.authHeadline {
                    Text(headline)
                        .font(.caption.weight(.semibold))
                }
                if let detail = self.projection.authDetail {
                    Text(detail)
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }
                // Hide the developer-facing diagnostic code + recovery-action
                // button stack when auth is healthy. The summary line and the
                // green check above already convey the state; the recovery
                // actions live one click away in Settings if the user wants
                // to reach them proactively.
                if !isHealthy {
                    if let diagnostic = self.projection.authDiagnosticCode {
                        Text("Diagnostic: \(diagnostic)")
                            .font(.caption2.monospaced())
                            .foregroundStyle(.secondary)
                    }
                    ForEach(self.model.authRecoveryActions) { action in
                        Button {
                            Task { await self.model.runAuthRecoveryAction(action) }
                        } label: {
                            Label(action.label, systemImage: "key.fill")
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                        .buttonStyle(SecondaryDashboardButtonStyle())
                    }
                }
            }
            .padding(8)
            .menuCardBackground(opacity: 0.04, cornerRadius: 8)
        }
    }
}

private struct GlobalIssueBanner: View {
    let message: String
    let detail: String?

    var body: some View {
        HStack(alignment: .top, spacing: 8) {
            Image(systemName: "exclamationmark.triangle.fill")
                .foregroundStyle(.orange)
            VStack(alignment: .leading, spacing: 2) {
                Text(self.message)
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.primary)
                if let detail {
                    Text(detail)
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }
            }
            Spacer(minLength: 0)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 8)
        .background(
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .fill(Color.orange.opacity(0.08))
        )
        .overlay(
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .stroke(Color.orange.opacity(0.18), lineWidth: 1)
        )
    }
}

struct MenuCardBackgroundModifier: ViewModifier {
    let opacity: Double
    let cornerRadius: CGFloat

    func body(content: Content) -> some View {
        content
            .background(
                RoundedRectangle(cornerRadius: self.cornerRadius, style: .continuous)
                    .fill(Color.primary.opacity(self.opacity))
            )
            .overlay(
                RoundedRectangle(cornerRadius: self.cornerRadius, style: .continuous)
                    .stroke(Color.primary.opacity(0.08), lineWidth: 1)
            )
    }
}

extension View {
    func menuCardBackground(opacity: Double, cornerRadius: CGFloat = 10) -> some View {
        self.modifier(MenuCardBackgroundModifier(opacity: opacity, cornerRadius: cornerRadius))
    }
}

private struct SecondaryActionLabel: View {
    let title: String
    let systemImage: String

    var body: some View {
        HStack(spacing: 8) {
            Image(systemName: self.systemImage)
                .frame(width: 14)
                .foregroundStyle(.secondary)
            Text(self.title)
            Spacer()
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }
}

private struct SessionDisclosureRow: View {
    let title: String
    let subtitle: String

    var body: some View {
        HStack(spacing: 10) {
            VStack(alignment: .leading, spacing: 2) {
                Text(self.title)
                    .font(.body.weight(.medium))
                HStack(spacing: 5) {
                    Circle()
                        .fill(self.statusColor)
                        .frame(width: 6, height: 6)
                    Text(self.subtitle)
                }
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
            Spacer()
            Image(systemName: "chevron.down")
                .font(.caption.weight(.semibold))
                .foregroundStyle(.secondary)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 7)
        .background(
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .fill(.regularMaterial)
        )
        .overlay(
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .stroke(Color.primary.opacity(0.08), lineWidth: 1)
        )
    }

    private var statusColor: Color {
        switch self.subtitle.lowercased() {
        case "connected":
            return .green
        case "login required", "expired":
            return .orange
        default:
            return .secondary.opacity(0.7)
        }
    }
}

struct PrimaryDashboardButtonStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(.body.weight(.semibold))
            .padding(.horizontal, 12)
            .padding(.vertical, 9)
            .foregroundStyle(Color.white)
            .background(
                RoundedRectangle(cornerRadius: 10, style: .continuous)
                    .fill(Color.accentColor.opacity(configuration.isPressed ? 0.72 : 0.84))
            )
    }
}

struct SecondaryDashboardButtonStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(.body)
            .padding(.horizontal, 10)
            .padding(.vertical, 7)
            .background(
                RoundedRectangle(cornerRadius: 10, style: .continuous)
                    .fill(.regularMaterial.opacity(configuration.isPressed ? 0.92 : 1))
            )
            .overlay(
                RoundedRectangle(cornerRadius: 10, style: .continuous)
                    .stroke(Color.primary.opacity(configuration.isPressed ? 0.12 : 0.07), lineWidth: 1)
            )
    }
}
