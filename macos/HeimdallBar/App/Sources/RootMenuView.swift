import AppKit
import HeimdallBarShared
import SwiftUI

struct RootMenuView: View {
    @Bindable var model: AppModel

    var body: some View {
        let overview = self.model.overviewProjection()

        VStack(alignment: .leading, spacing: 10) {
            MenuChromeHeader(
                title: "HeimdallBar",
                status: overview.refreshStatusLabel,
                isRefreshing: overview.isRefreshing,
                attentionLabel: self.attentionLabel(for: overview)
            )

            if self.model.visibleTabs.count > 1 {
                MergeTabSwitcher(
                    tabs: self.model.visibleTabs,
                    selection: self.$model.selectedMergeTab
                )
            }

            if self.model.selectedMergeTab == .overview {
                OverviewMenuCard(model: self.model, projection: overview)
            } else if let provider = self.model.selectedMergeTab.providerID {
                ProviderMenuCard(model: self.model, projection: self.model.projection(for: provider))
                SessionActionGroup(model: self.model, providers: [provider])
            }

            Divider()

            MenuActionRow(model: self.model, tab: self.model.selectedMergeTab)

            if self.model.selectedMergeTab == .overview {
                SessionActionGroup(model: self.model, providers: self.model.visibleProviders)
            }

            if let lastError = self.model.lastError {
                Text(lastError)
                    .font(.caption)
                    .foregroundStyle(.red)
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
    @Bindable var model: AppModel
    let provider: ProviderID

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            MenuChromeHeader(
                title: self.provider.title,
                status: self.model.projection(for: self.provider).refreshStatusLabel,
                isRefreshing: self.model.projection(for: self.provider).isRefreshing,
                attentionLabel: self.providerAttentionLabel
            )
            ProviderMenuCard(model: self.model, projection: self.model.projection(for: self.provider))
            SessionActionGroup(model: self.model, providers: [self.provider])

            Divider()

            MenuActionRow(model: self.model, tab: self.provider == .claude ? .claude : .codex)
        }
        .padding(12)
        .frame(width: 336)
    }

    private var providerAttentionLabel: String? {
        let projection = self.model.projection(for: self.provider)
        switch projection.visualState {
        case .healthy, .refreshing:
            return nil
        default:
            return "\(projection.title) is \(projection.stateLabel.lowercased())"
        }
    }
}

struct ProviderMenuCard: View {
    @Bindable var model: AppModel
    let projection: ProviderMenuProjection

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(alignment: .firstTextBaseline) {
                Text(self.projection.title)
                    .font(.system(size: 13, weight: .semibold))
                StateBadge(state: self.projection.visualState, label: self.projection.stateLabel)
                Spacer()
                Text(self.projection.refreshStatusLabel)
                    .font(.caption2.monospacedDigit())
                    .foregroundStyle(.secondary)
            }
            Text(self.primaryMetricText)
                .font(.system(size: 28, weight: .bold, design: .rounded))
                .foregroundStyle(.primary)
            Text(self.secondaryMetricText)
                .font(.caption)
                .foregroundStyle(.secondary)
            AuthStatusSection(model: self.model, provider: self.projection.provider, projection: self.projection)
            if self.projection.laneDetails.count > 1 {
                Divider()
                    .padding(.vertical, 1)
            }
            if let sourceExplanationLabel = self.projection.sourceExplanationLabel {
                Text(sourceExplanationLabel)
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
            ForEach(self.projection.warningLabels.prefix(2), id: \.self) { warning in
                Text(warning)
                    .font(.caption2)
                    .foregroundStyle(.orange)
            }
            if let identityLabel = self.projection.identityLabel {
                Text(identityLabel)
                    .font(.caption)
            }
            if self.projection.laneDetails.count > 1 {
                LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 8) {
                    ForEach(self.projection.laneDetails.dropFirst()) { detail in
                        LaneStatusCard(detail: detail)
                    }
                }
            }
            if !self.projection.historyFractions.isEmpty {
                HistoryBarStrip(fractions: self.projection.historyFractions)
            }
            Text(self.projection.costLabel)
                .font(.caption)
                .foregroundStyle(.secondary)
            if let creditsLabel = self.projection.creditsLabel {
                Text(creditsLabel)
                    .font(.caption)
            }
            if let statusLabel = self.projection.statusLabel {
                Text(statusLabel)
                    .font(.caption)
                    .foregroundStyle(.primary)
            }
            if let incidentLabel = self.projection.incidentLabel {
                Text(incidentLabel)
                    .font(.caption)
                    .foregroundStyle(.orange)
            }
            if !self.projection.claudeFactors.isEmpty {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Usage Factors")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    ForEach(self.projection.claudeFactors) { factor in
                        Text("\(factor.displayLabel): \(Int(factor.percent.rounded()))%")
                            .font(.caption2)
                    }
                }
            }
            if let adjunct = self.projection.adjunct {
                AdjunctSummaryCard(adjunct: adjunct)
            }
            if let error = self.projection.error {
                Text(error)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
        .padding(10)
        .menuCardBackground(opacity: self.cardBackgroundOpacity)
    }

    private var primaryMetricText: String {
        guard let primaryLane = self.projection.laneDetails.first else {
            return self.unavailableValueLabel
        }
        if let remaining = primaryLane.remainingPercent {
            return "\(remaining)% left"
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
           self.model.isClaudeOAuthCredentialsMissing() {
            return "Missing Claude OAuth credentials file"
        }
        return "Live \(self.projection.title) session not available"
    }

}

struct OverviewMenuCard: View {
    @Bindable var model: AppModel
    let projection: OverviewMenuProjection

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            ForEach(self.sortedItems) { item in
                OverviewProviderCard(model: self.model, item: item)
            }
            if !self.projection.historyFractions.isEmpty {
                VStack(alignment: .leading, spacing: 6) {
                    Text("Last 7 days")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                    HistoryBarStrip(fractions: self.projection.historyFractions)
                }
            }
            VStack(alignment: .leading, spacing: 6) {
                Text("Overview")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.secondary)
                Text(self.projection.combinedCostLabel)
                    .font(.system(size: 15, weight: .semibold))
                    .foregroundStyle(.primary)
                Text(self.projection.activitySummaryLabel)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                if !self.projection.warningLabels.isEmpty {
                    Label("Source limits affect some provider data", systemImage: "exclamationmark.triangle.fill")
                        .font(.caption2.weight(.medium))
                        .foregroundStyle(.orange)
                }
            }
            .padding(10)
            .menuCardBackground(opacity: 0.02, cornerRadius: 12)
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
                    Text("\(remainingPercent)%")
                        .font(.caption.monospacedDigit().weight(.semibold))
                }
            }
            if let paceLabel = self.detail.paceLabel {
                Text("Pace \(paceLabel.lowercased())")
                    .font(.caption2)
            }
            if let resetDetail = self.detail.resetDetail {
                Text(resetDetail)
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
        }
        .padding(8)
        .frame(maxWidth: .infinity, alignment: .leading)
        .menuCardBackground(opacity: 0.045, cornerRadius: 8)
    }
}

struct HistoryBarStrip: View {
    let fractions: [Double]

    var body: some View {
        HStack(alignment: .bottom, spacing: 3) {
            ForEach(Array(self.fractions.enumerated()), id: \.offset) { entry in
                let fraction = entry.element
                RoundedRectangle(cornerRadius: 1)
                    .fill(Color.primary.opacity(0.06))
                    .overlay(alignment: .bottom) {
                        RoundedRectangle(cornerRadius: 1)
                            .fill(Color.primary)
                            .frame(height: max(2, 16 * fraction))
                    }
                    .frame(width: 6, height: 16)
            }
        }
    }
}

private struct MenuActionRow: View {
    @Bindable var model: AppModel
    let tab: MergeMenuTab

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Button(action: {
                Task { await self.model.refresh(force: true, provider: self.primaryRefreshProvider) }
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
            .disabled(self.model.isRefreshing)

            if self.tab.providerID != nil {
                Button(action: {
                    Task { await self.model.refresh(force: true, provider: nil) }
                }) {
                    SecondaryActionLabel(title: "Refresh All", systemImage: "arrow.clockwise")
                }
                .buttonStyle(SecondaryDashboardButtonStyle())
                .disabled(self.model.isRefreshing)
            }

            VStack(spacing: 6) {
                Button(action: {
                    if let url = URL(string: "http://127.0.0.1:\(self.model.config.helperPort)") {
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

            Button(action: {
                Task {
                    await self.model.prepareForExit()
                    NSApplication.shared.terminate(nil)
                }
            }) {
                Text("Quit")
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
            .buttonStyle(.plain)
        }
    }

    private var primaryRefreshTitle: String {
        if self.model.isRefreshing {
            if let provider = self.primaryRefreshProvider {
                return "Refreshing \(provider.title)…"
            }
            let names = self.model.visibleProviders.map(\.title).joined(separator: " + ")
            return names.isEmpty ? "Refreshing…" : "Refreshing \(names)…"
        }
        return self.model.refreshActionLabel(for: self.tab)
    }

    private var primaryRefreshProvider: ProviderID? {
        self.tab.providerID
    }
}

private struct SessionActionGroup: View {
    @Bindable var model: AppModel
    let providers: [ProviderID]

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("Web Sessions")
                .font(.caption)
                .foregroundStyle(.secondary)
            ForEach(self.providers, id: \.self) { provider in
                Menu {
                    let candidates = self.model.importCandidates(for: provider)
                    if candidates.isEmpty {
                        Text("No browser stores found")
                    } else {
                        ForEach(candidates) { candidate in
                            Button("Import \(candidate.title)") {
                                Task { await self.model.importBrowserSession(provider: provider, candidate: candidate) }
                            }
                        }
                    }
                    if self.model.importedSession(for: provider) != nil {
                        Divider()
                        Button("Reset Stored Session") {
                            Task { await self.model.resetBrowserSession(provider: provider) }
                        }
                    }
                } label: {
                    SessionDisclosureRow(
                        title: "\(provider.title) Web Session",
                        subtitle: self.sessionHealthLabel(for: provider)
                    )
                }
                .menuStyle(.borderlessButton)
            }
        }
        .disabled(self.model.isImportingSession)
    }

    private func sessionHealthLabel(for provider: ProviderID) -> String {
        guard let session = self.model.importedSession(for: provider) else {
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

private struct OverviewProviderCard: View {
    @Bindable var model: AppModel
    let item: ProviderMenuProjection

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(alignment: .firstTextBaseline) {
                Text(self.item.title)
                    .font(.headline)
                StateBadge(state: self.item.visualState, label: self.item.stateLabel)
                Spacer()
                Text(self.item.refreshStatusLabel)
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
            TopMetricRow(
                title: self.metricTitle,
                value: self.metricValue,
                detail: self.metricDetail
            )
            Text(self.item.costLabel)
                .font(.caption)
                .foregroundStyle(.secondary)
            if let authHeadline = self.item.authHeadline {
                Text(authHeadline)
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
        self.item.laneDetails.first?.remainingPercent == nil ? "Needs attention" : "Session"
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
           self.model.isClaudeOAuthCredentialsMissing() {
            return "Missing Claude OAuth credentials file"
        }
        return "Live \(self.item.title) session not available"
    }

}

private struct AuthStatusSection: View {
    @Bindable var model: AppModel
    let provider: ProviderID
    let projection: ProviderMenuProjection

    var body: some View {
        if self.projection.authHeadline != nil
            || self.projection.authDetail != nil
            || !self.projection.authRecoveryActions.isEmpty
        {
            VStack(alignment: .leading, spacing: 6) {
                HStack(alignment: .firstTextBaseline) {
                    Text("Auth")
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(.secondary)
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
                if let diagnostic = self.projection.authDiagnosticCode {
                    Text("Diagnostic: \(diagnostic)")
                        .font(.caption2.monospaced())
                        .foregroundStyle(.secondary)
                }
                ForEach(self.projection.authRecoveryActions.prefix(2)) { action in
                    Button {
                        Task { await self.model.runAuthRecoveryAction(action, for: self.provider) }
                    } label: {
                        Label(action.label, systemImage: "key.fill")
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                    .buttonStyle(SecondaryDashboardButtonStyle())
                }
            }
            .padding(8)
            .menuCardBackground(opacity: 0.04, cornerRadius: 8)
        }
    }
}

private struct MenuCardBackgroundModifier: ViewModifier {
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

private extension View {
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

private struct PrimaryDashboardButtonStyle: ButtonStyle {
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

private struct SecondaryDashboardButtonStyle: ButtonStyle {
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
