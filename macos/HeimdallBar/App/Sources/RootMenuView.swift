import AppKit
import HeimdallBarShared
import SwiftUI

struct RootMenuView: View {
    @Bindable var model: AppModel

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            MenuChromeHeader(
                title: "HeimdallBar",
                status: self.model.overviewProjection().refreshStatusLabel,
                isRefreshing: self.model.overviewProjection().isRefreshing
            )

            if self.model.visibleTabs.count > 1 {
                MergeTabSwitcher(
                    tabs: self.model.visibleTabs,
                    selection: self.$model.selectedMergeTab
                )
            }

            let overview = self.model.overviewProjection()
            if self.model.selectedMergeTab == .overview {
                OverviewMenuCard(projection: overview)
            } else if let provider = self.model.selectedMergeTab.providerID {
                ProviderMenuCard(projection: self.model.projection(for: provider))
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
        .padding(12)
        .frame(width: 368)
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
                isRefreshing: self.model.projection(for: self.provider).isRefreshing
            )
            ProviderMenuCard(projection: self.model.projection(for: self.provider))
            SessionActionGroup(model: self.model, providers: [self.provider])

            Divider()

            MenuActionRow(model: self.model, tab: self.provider == .claude ? .claude : .codex)
        }
        .padding(12)
        .frame(width: 352)
    }
}

struct ProviderMenuCard: View {
    let projection: ProviderMenuProjection

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(alignment: .firstTextBaseline) {
                Text(self.projection.title)
                    .font(.headline)
                StateBadge(state: self.projection.visualState, label: self.projection.stateLabel)
                Spacer()
                Text(self.projection.refreshStatusLabel)
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
            Text(self.projection.sourceLabel)
                .font(.caption)
                .foregroundStyle(.secondary)
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
            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 8) {
                ForEach(self.projection.laneDetails) { detail in
                    LaneStatusCard(detail: detail)
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

    private var cardBackgroundOpacity: Double {
        switch self.projection.visualState {
        case .healthy:
            return 0.05
        case .refreshing:
            return 0.07
        case .stale:
            return 0.04
        case .degraded:
            return 0.09
        case .incident:
            return 0.12
        case .error:
            return 0.1
        }
    }
}

struct OverviewMenuCard: View {
    let projection: OverviewMenuProjection

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            ForEach(self.projection.items) { item in
                OverviewProviderCard(item: item)
            }
            if !self.projection.historyFractions.isEmpty {
                HistoryBarStrip(fractions: self.projection.historyFractions)
            }
            VStack(alignment: .leading, spacing: 4) {
                Text(self.projection.combinedCostLabel)
                    .font(.caption)
                    .foregroundStyle(.primary)
                Text(self.projection.activitySummaryLabel)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                if !self.projection.warningLabels.isEmpty {
                    Text("Some provider data is limited by source availability.")
                        .font(.caption2)
                        .foregroundStyle(.orange)
                }
            }
        }
        .padding(10)
        .menuCardBackground(opacity: 0.05)
    }
}

private struct MenuChromeHeader: View {
    let title: String
    let status: String
    let isRefreshing: Bool

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(self.title)
                    .font(.headline.weight(.semibold))
                Spacer()
                if self.isRefreshing {
                    ProgressView()
                        .controlSize(.small)
                }
            }
            Text(self.status)
                .font(.caption2)
                .foregroundStyle(.secondary)
        }
    }
}

private struct StateBadge: View {
    let state: ProviderVisualState
    let label: String

    var body: some View {
        Text(self.label.uppercased())
            .font(.caption2.monospaced())
            .padding(.horizontal, 6)
            .padding(.vertical, 3)
            .foregroundStyle(self.foregroundColor)
            .background(self.backgroundColor)
            .clipShape(Capsule())
    }

    private var foregroundColor: Color {
        switch self.state {
        case .incident, .error:
            return .white
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
            return Color.secondary.opacity(0.12)
        case .degraded:
            return Color.orange.opacity(0.18)
        case .incident:
            return Color.orange
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
                    .font(.caption)
                Spacer()
                if let remainingPercent = self.detail.remainingPercent {
                    Text("\(remainingPercent)%")
                        .font(.caption.monospacedDigit())
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
                    .fill(Color.primary.opacity(0.18))
                    .overlay(alignment: .bottom) {
                        RoundedRectangle(cornerRadius: 1)
                            .fill(Color.primary)
                            .frame(height: max(2, 24 * fraction))
                    }
                    .frame(width: 10, height: 24)
            }
        }
    }
}

private struct MenuActionRow: View {
    @Bindable var model: AppModel
    let tab: MergeMenuTab

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Button(self.primaryRefreshTitle) {
                Task { await self.model.refresh(force: true, provider: self.primaryRefreshProvider) }
            }
            .keyboardShortcut("r")

            if self.tab.providerID != nil {
                Button("Refresh All") {
                    Task { await self.model.refresh(force: true, provider: nil) }
                }
            }

            Button("Open Dashboard") {
                if let url = URL(string: "http://127.0.0.1:\(self.model.config.helperPort)") {
                    NSWorkspace.shared.open(url)
                }
            }

            SettingsLink {
                Text("Open Settings")
            }

            Button("Quit") {
                Task {
                    await self.model.prepareForExit()
                    NSApplication.shared.terminate(nil)
                }
            }
        }
    }

    private var primaryRefreshTitle: String {
        self.model.refreshActionLabel(for: self.tab)
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
                Menu("\(provider.title) Web Session") {
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
                }
            }
        }
        .disabled(self.model.isImportingSession)
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
        HStack(spacing: 6) {
            ForEach(self.tabs) { tab in
                Button {
                    self.selection = tab
                } label: {
                    Text(tab.title)
                        .font(.caption.weight(.medium))
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 6)
                }
                .buttonStyle(.plain)
                .foregroundStyle(self.selection == tab ? Color.primary : Color.secondary)
                .background(
                    RoundedRectangle(cornerRadius: 8)
                        .fill(self.selection == tab ? Color.primary.opacity(0.1) : Color.clear)
                )
                .overlay(
                    RoundedRectangle(cornerRadius: 8)
                        .stroke(Color.primary.opacity(self.selection == tab ? 0.16 : 0.08), lineWidth: 1)
                )
            }
        }
    }
}

private struct OverviewProviderCard: View {
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
            Text(self.item.laneDetails.first?.summary ?? "Session data unavailable")
                .font(.caption)
                .foregroundStyle(.primary)
            Text(self.item.costLabel)
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .padding(10)
        .menuCardBackground(opacity: 0.045)
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
