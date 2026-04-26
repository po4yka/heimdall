import AppKit
import HeimdallDomain
import HeimdallServices
import SwiftUI

private enum AppShellLayout {
    static let overviewMaxWidth: CGFloat = 1_380
    static let sectionCardMinimumWidth: CGFloat = 420
    static let analyticsCardMinimumWidth: CGFloat = 320
    static let analyticsSplitMinimumWidth: CGFloat = 520
}

struct AppShellView: View {
    @Environment(\.scenePhase) private var scenePhase
    @Bindable var shell: AppShellModel
    @Bindable var overview: OverviewFeatureModel
    @Bindable var liveMonitor: LiveMonitorFeatureModel
    let helperPort: Int
    let providerModel: (ProviderID) -> ProviderFeatureModel

    var body: some View {
        NavigationSplitView {
            WindowSidebar(shell: self.shell)
            .navigationTitle("HeimdallBar")
        } detail: {
            ScrollView {
                VStack(alignment: .leading, spacing: 18) {
                    switch self.shell.navigationSelection {
                    case .overview:
                        WindowOverviewView(
                            overview: self.overview,
                            shell: self.shell,
                            providerModel: self.providerModel
                        )
                    case .liveMonitor:
                        WindowLiveMonitorView(model: self.liveMonitor, helperPort: self.helperPort)
                    case .provider(let provider):
                        WindowProviderView(model: self.providerModel(provider))
                    }
                }
                .padding(24)
                .frame(maxWidth: AppShellLayout.overviewMaxWidth, alignment: .leading)
                .frame(maxWidth: .infinity, alignment: .center)
            }
            .background(Color(nsColor: .windowBackgroundColor))
            .onAppear { self.syncLiveMonitorActivity() }
            .onChange(of: self.shell.navigationSelection) { _, _ in
                self.syncLiveMonitorActivity()
            }
            .onChange(of: self.scenePhase) { _, _ in
                self.syncLiveMonitorActivity()
            }
        }
        .toolbar {
            ToolbarItemGroup {
                Button {
                    Task {
                        switch self.shell.navigationSelection {
                        case .overview:
                            await self.overview.refreshAll()
                        case .liveMonitor:
                            await self.liveMonitor.refresh()
                        case .provider(let provider):
                            await self.providerModel(provider).refresh()
                        }
                    }
                }
                label: { Text("Refresh") }
                .help("Refresh the current view")
                .accessibilityLabel("Refresh the current view")
                .disabled(self.isBusy)

                SettingsLink {
                    Text("Settings")
                }
                .help("Open HeimdallBar settings")
                .accessibilityLabel("Open HeimdallBar settings")

                Button {
                    if let url = URL(string: "http://127.0.0.1:\(self.helperPort)") {
                        NSWorkspace.shared.open(url)
                    }
                } label: {
                    Text("Web Dashboard")
                }
                .help("Open the local Heimdall web dashboard")
                .accessibilityLabel("Open the local Heimdall web dashboard")
            }
        }
    }

    private var isBusy: Bool {
        switch self.shell.navigationSelection {
        case .overview:
            return self.overview.projection.isRefreshing
        case .liveMonitor:
            return self.liveMonitor.isRefreshing
        case .provider(let provider):
            return self.providerModel(provider).isBusy
        }
    }

    private func syncLiveMonitorActivity() {
        self.liveMonitor.updateActivity(
            isSelected: self.shell.navigationSelection == .liveMonitor,
            appIsActive: self.scenePhase == .active
        )
    }
}

private struct WindowSidebar: View {
    @Bindable var shell: AppShellModel

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 6) {
                ForEach(self.shell.navigationItems, id: \.id) { item in
                    Button {
                        self.shell.selectNavigation(item)
                    } label: {
                        SidebarNavigationRow(
                            item: item,
                            isSelected: self.shell.navigationSelection == item
                        )
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(12)
        }
        .frame(minWidth: 228, idealWidth: 244, maxHeight: .infinity, alignment: .topLeading)
        .background(Color(nsColor: .windowBackgroundColor))
    }
}

private struct WindowOverviewView: View {
    @Bindable var overview: OverviewFeatureModel
    @Bindable var shell: AppShellModel
    let providerModel: (ProviderID) -> ProviderFeatureModel

    var body: some View {
        let projection = self.overview.projection

        VStack(alignment: .leading, spacing: 24) {
            WindowHeader(
                title: "Overview",
                subtitle: projection.refreshedAtLabel,
                issue: WindowHeaderIssuePresentation.make(message: projection.globalIssueLabel),
                onRetry: {
                    Task { await self.overview.refreshAll() }
                },
                isRetrying: projection.isRefreshing
            )

            if let aggregate = self.overview.syncedAggregate {
                WindowOverviewCloudSyncSection(aggregate: aggregate, state: self.overview.cloudSyncState)
            }

            WindowOverviewProvidersSection(
                items: self.sortedItems(from: projection),
                providerModel: self.providerModel,
                openProvider: { provider in
                    self.shell.selectNavigation(.provider(provider))
                }
            )

            WindowOverviewActivitySection(projection: projection)
        }
    }

    private func sortedItems(from projection: OverviewMenuProjection) -> [ProviderMenuProjection] {
        projection.items.sorted {
            let lhs = $0.visualState.severityRank
            let rhs = $1.visualState.severityRank
            if lhs == rhs {
                return $0.title < $1.title
            }
            return lhs > rhs
        }
    }
}

private struct WindowOverviewCloudSyncSection: View {
    let aggregate: SyncedAggregateEnvelope
    let state: CloudSyncSpaceState

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            WindowSectionHeader(
                title: "Cloud Sync",
                subtitle: self.subtitle
            )

            LazyVGrid(columns: [GridItem(.adaptive(minimum: AppShellLayout.analyticsCardMinimumWidth), spacing: 16)], spacing: 16) {
                VStack(alignment: .leading, spacing: 8) {
                    Text("All Installations")
                        .font(.headline)
                    Text(Self.currency(self.aggregate.aggregateTotals.last90DaysCostUSD))
                        .font(.system(size: 30, weight: .semibold).monospacedDigit())
                    Text("\(self.aggregate.aggregateTotals.last90DaysTokens) tokens over 90 days")
                        .foregroundStyle(.secondary)
                    Text("\(self.aggregate.installations.count) synced installation(s)")
                        .foregroundStyle(.secondary)
                }
                .padding(18)
                .menuCardBackground(opacity: 0.04, cornerRadius: 16)

                ForEach(self.aggregate.installations) { installation in
                    VStack(alignment: .leading, spacing: 8) {
                        Text(installation.sourceDevice)
                            .font(.headline)
                        Text(displayTimestamp(installation.publishedAt))
                            .foregroundStyle(.secondary)
                        Text("\(installation.totals.last90DaysTokens) tokens")
                            .font(.system(.body, design: .monospaced))
                        if !installation.accountLabels.isEmpty {
                            Text(installation.accountLabels.joined(separator: ", "))
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                        if installation.isStale {
                            Text("Stale provider data")
                                .font(.caption.weight(.semibold))
                                .foregroundStyle(.warning)
                        }
                    }
                    .padding(18)
                    .menuCardBackground(opacity: 0.04, cornerRadius: 16)
                }
            }
        }
    }

    private var subtitle: String {
        switch self.state.status {
        case .inviteReady:
            return "Owner space ready. Share link copied from Settings."
        case .participantJoined:
            return "Joined shared usage across Macs."
        case .ownerReady:
            return "Owner space ready for sharing."
        case .sharingBlocked, .iCloudUnavailable:
            return self.state.statusMessage ?? "Cloud Sync is unavailable on this Mac."
        case .notConfigured:
            return "Refresh once, then create a share link in Settings."
        }
    }

    private static func currency(_ value: Double) -> String {
        FormatHelpers.formatUSD(value)
    }

    private func displayTimestamp(_ raw: String) -> String {
        liveMonitorAbbreviatedTimestamp(raw)
    }
}

private struct WindowOverviewProvidersSection: View {
    let items: [ProviderMenuProjection]
    let providerModel: (ProviderID) -> ProviderFeatureModel
    let openProvider: (ProviderID) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            WindowSectionHeader(
                title: "Providers",
                subtitle: "Provider state, freshness, and current session quota"
            )

            LazyVGrid(columns: [GridItem(.adaptive(minimum: AppShellLayout.sectionCardMinimumWidth), spacing: 16)], spacing: 16) {
                ForEach(self.items) { item in
                    WindowOverviewProviderCard(
                        model: self.providerModel(item.provider),
                        item: item,
                        openProvider: {
                            self.openProvider(item.provider)
                        }
                    )
                }
            }
        }
    }
}

private struct WindowOverviewActivitySection: View {
    let projection: OverviewMenuProjection

    var body: some View {
        let analytics = WindowOverviewDesktopAnalyticsModel.make(projection: self.projection)

        VStack(alignment: .leading, spacing: 14) {
            WindowSectionHeader(
                title: "Activity",
                subtitle: "Combined usage and recent spend"
            )

            LazyVGrid(columns: [GridItem(.adaptive(minimum: AppShellLayout.analyticsCardMinimumWidth), spacing: 16)], spacing: 16) {
                WindowOverviewTotalsCard(projection: self.projection)

                if !self.projection.historyFractions.isEmpty {
                    WindowOverviewHistoryCard(projection: self.projection)
                }

                if let weeklyProjection = analytics.weeklyProjection {
                    WindowOverviewWeeklyProjectionCard(model: weeklyProjection)
                }
            }

            if analytics.showsProviderComparison {
                ProviderComparisonChart(items: analytics.providerComparisonItems)
            }

            if analytics.showsHeatmap || analytics.showsModelMix {
                ViewThatFits(in: .horizontal) {
                    HStack(alignment: .top, spacing: 16) {
                        if analytics.showsHeatmap {
                            ActivityHeatmap(cells: analytics.heatmapCells)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                        if analytics.showsModelMix {
                            ModelDistributionDonut(
                                rows: analytics.modelRows,
                                dailyByModel: analytics.dailyModelRows
                            )
                            .frame(
                                minWidth: AppShellLayout.analyticsSplitMinimumWidth * 0.64,
                                maxWidth: AppShellLayout.analyticsSplitMinimumWidth * 0.82,
                                alignment: .leading
                            )
                        }
                    }

                    VStack(alignment: .leading, spacing: 16) {
                        if analytics.showsHeatmap {
                            ActivityHeatmap(cells: analytics.heatmapCells)
                        }
                        if analytics.showsModelMix {
                            ModelDistributionDonut(
                                rows: analytics.modelRows,
                                dailyByModel: analytics.dailyModelRows
                            )
                        }
                    }
                }
            }

            if analytics.showsRecentSessions {
                SessionsTable(sessions: analytics.recentSessions)
            }
        }
    }
}

private struct WindowLiveMonitorView: View {
    @Bindable var model: LiveMonitorFeatureModel
    let helperPort: Int
    @State private var isPreferencesPopoverPresented = false

    var body: some View {
        VStack(alignment: .leading, spacing: 24) {
            WindowHeader(
                title: "Live Monitor",
                subtitle: self.headerSubtitle,
                issue: WindowHeaderIssuePresentation.make(message: self.model.issue),
                onRetry: {
                    Task { await self.model.refresh() }
                },
                isRetrying: self.model.isRefreshing
            ) {
                Button {
                    self.isPreferencesPopoverPresented.toggle()
                } label: {
                    Image(systemName: "slider.horizontal.3")
                        .font(.body.weight(.medium))
                }
                .buttonStyle(SecondaryDashboardButtonStyle())
                .accessibilityLabel("Live Monitor preferences")
                .popover(isPresented: self.$isPreferencesPopoverPresented, arrowEdge: .top) {
                    LiveMonitorPreferencesPopover(model: self.model)
                }
            }

            if let envelope = self.model.envelope {
                VStack(alignment: .leading, spacing: 14) {
                    WindowSectionHeader(
                        title: "Provider Lanes",
                        subtitle: envelope.freshness.hasStaleProviders
                            ? "Stale: \(envelope.freshness.staleProviders.joined(separator: ", "))"
                            : "All visible providers are current"
                    )

                    LazyVGrid(columns: [GridItem(.adaptive(minimum: AppShellLayout.sectionCardMinimumWidth), spacing: 16)], spacing: 16) {
                        ForEach(self.model.providers) { provider in
                            WindowLiveMonitorProviderCard(provider: provider)
                        }
                    }
                }

                ForEach(self.model.detailProviders) { provider in
                    WindowLiveMonitorDetailSection(
                        provider: provider,
                        density: self.model.density,
                        hiddenPanels: self.model.hiddenPanels
                    )
                }
            } else {
                WindowLiveMonitorEmptyState(isRefreshing: self.model.isRefreshing)
            }

            HStack(spacing: 12) {
                Button("Open Web Monitor") {
                    if let url = URL(string: "http://127.0.0.1:\(self.helperPort)/monitor") {
                        NSWorkspace.shared.open(url)
                    }
                }
                .buttonStyle(.bordered)

                Button("Open Dashboard") {
                    if let url = URL(string: "http://127.0.0.1:\(self.helperPort)") {
                        NSWorkspace.shared.open(url)
                    }
                }
                .buttonStyle(.borderless)
            }
        }
    }

    private var headerSubtitle: String {
        if let envelope = self.model.envelope {
            return "Updated \(Self.shortTime(envelope.generatedAt))"
        }
        if self.model.isRefreshing {
            return "Connecting to the local helper…"
        }
        return "Real-time provider rate windows, costs, and depletion forecasts. Refreshes every 10 seconds."
    }

    private static func shortTime(_ iso: String) -> String {
        liveMonitorShortTime(iso)
    }
}

private struct WindowLiveMonitorEmptyState: View {
    let isRefreshing: Bool

    private struct Highlight: Identifiable {
        let id: String
        let symbol: String
        let title: String
        let subtitle: String
    }

    private static let highlights: [Highlight] = [
        Highlight(
            id: "providers",
            symbol: "rectangle.stack",
            title: "Provider lanes",
            subtitle: "Today's spend, primary and secondary rate windows, and weekly projection per provider."
        ),
        Highlight(
            id: "active-block",
            symbol: "clock",
            title: "Active block",
            subtitle: "Tokens in flight, entries, end time, and projected quota burn."
        ),
        Highlight(
            id: "forecast",
            symbol: "chart.line.uptrend.xyaxis",
            title: "Depletion forecast",
            subtitle: "When current usage trends will exhaust your rate window."
        ),
        Highlight(
            id: "quotas",
            symbol: "gauge.with.dots.needle.bottom.50percent",
            title: "Suggested quotas",
            subtitle: "Recommended pacing to land safely inside the next reset."
        ),
        Highlight(
            id: "context",
            symbol: "doc.text.magnifyingglass",
            title: "Context window",
            subtitle: "Live input-token usage against the model's context limit."
        ),
        Highlight(
            id: "session",
            symbol: "bubble.left.and.bubble.right",
            title: "Recent session",
            subtitle: "Latest conversation: turns, duration, cost, and model."
        ),
    ]

    var body: some View {
        VStack(alignment: .leading, spacing: 18) {
            VStack(spacing: 14) {
                ZStack {
                    Circle()
                        .fill(Color.accentInteractive.opacity(0.10))
                        .frame(width: 56, height: 56)
                    Image(systemName: "waveform.path.ecg")
                        .font(.system(size: 22, weight: .semibold))
                        .foregroundStyle(Color.accentInteractive)
                }

                VStack(spacing: 6) {
                    Text("Waiting for live data")
                        .font(.title3.weight(.semibold))
                    Text("Live Monitor streams provider rate-window usage, depletion forecasts, and quota suggestions for every connected AI provider. The helper will populate this view automatically as soon as the first refresh succeeds.")
                        .font(.callout)
                        .foregroundStyle(.secondary)
                        .multilineTextAlignment(.center)
                        .frame(maxWidth: 460)
                        .fixedSize(horizontal: false, vertical: true)
                }

                if self.isRefreshing {
                    HStack(spacing: 8) {
                        ProgressView()
                            .controlSize(.small)
                        Text("Refreshing…")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, 24)
            .padding(.horizontal, 16)
            .background(
                RoundedRectangle(cornerRadius: 14, style: .continuous)
                    .fill(Color.accentInteractive.opacity(0.05))
            )
            .overlay(
                RoundedRectangle(cornerRadius: 14, style: .continuous)
                    .stroke(Color.accentInteractive.opacity(0.18), lineWidth: 1)
            )

            VStack(alignment: .leading, spacing: 10) {
                Text("What you'll see when data arrives")
                    .font(.callout.weight(.semibold))
                    .foregroundStyle(.secondary)

                LazyVGrid(
                    columns: [GridItem(.adaptive(minimum: 240), spacing: 10)],
                    alignment: .leading,
                    spacing: 10
                ) {
                    ForEach(Self.highlights) { highlight in
                        HStack(alignment: .top, spacing: 10) {
                            Image(systemName: highlight.symbol)
                                .font(.callout.weight(.semibold))
                                .foregroundStyle(.secondary)
                                .frame(width: 20, alignment: .center)
                                .padding(.top, 1)
                            VStack(alignment: .leading, spacing: 2) {
                                Text(highlight.title)
                                    .font(.callout.weight(.semibold))
                                Text(highlight.subtitle)
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                                    .fixedSize(horizontal: false, vertical: true)
                            }
                            Spacer(minLength: 0)
                        }
                        .padding(12)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(
                            RoundedRectangle(cornerRadius: 10, style: .continuous)
                                .fill(Color.secondary.opacity(0.06))
                        )
                    }
                }
            }
        }
    }
}

private struct LiveMonitorPreferencesPopover: View {
    @Bindable var model: LiveMonitorFeatureModel

    var body: some View {
        VStack(alignment: .leading, spacing: 18) {
            VStack(alignment: .leading, spacing: 8) {
                Text("Provider focus")
                    .font(.callout.weight(.semibold))
                    .foregroundStyle(.secondary)
                Picker("Provider focus", selection: Binding(
                    get: { self.model.focus },
                    set: { self.model.setFocus($0) }
                )) {
                    ForEach(LiveMonitorFocus.allCases) { focus in
                        Text(focus.title).tag(focus)
                    }
                }
                .pickerStyle(.segmented)
                .labelsHidden()
            }

            VStack(alignment: .leading, spacing: 8) {
                Text("Detail density")
                    .font(.callout.weight(.semibold))
                    .foregroundStyle(.secondary)
                Picker("Detail density", selection: Binding(
                    get: { self.model.density },
                    set: { self.model.setDensity($0) }
                )) {
                    ForEach(LiveMonitorDensity.allCases) { density in
                        Text(density.title).tag(density)
                    }
                }
                .pickerStyle(.segmented)
                .labelsHidden()
            }

            Divider()

            VStack(alignment: .leading, spacing: 6) {
                Text("Show panels")
                    .font(.callout.weight(.semibold))
                    .foregroundStyle(.secondary)
                ForEach(LiveMonitorPanelID.allCases) { panel in
                    Toggle(panel.title, isOn: Binding(
                        get: { !self.model.isPanelHidden(panel) },
                        set: { self.model.setPanelVisibility(panel, isVisible: $0) }
                    ))
                    .toggleStyle(.switch)
                    .controlSize(.small)
                }
            }
        }
        .padding(18)
        .frame(minWidth: 280)
    }
}

private struct WindowLiveMonitorProviderCard: View {
    let provider: LiveMonitorProvider

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Text(self.provider.title)
                    .font(.title3.weight(.semibold))
                Text(self.provider.visualState.capitalized)
                    .font(.caption2.weight(.semibold))
                    .padding(.horizontal, 7)
                    .padding(.vertical, 3)
                    .background(
                        Capsule(style: .continuous)
                            .fill(self.stateColor.opacity(0.14))
                    )
                    .foregroundStyle(self.stateColor)
                Spacer(minLength: 0)
            }

            VStack(alignment: .leading, spacing: 4) {
                Text(Self.currency(self.provider.todayCostUSD))
                    .font(.system(size: 30, weight: .semibold).monospacedDigit())
                Text("Today cost")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            ForEach(Array([self.provider.primary, self.provider.secondary].enumerated()), id: \.offset) { index, window in
                if let window {
                    VStack(alignment: .leading, spacing: 6) {
                        HStack {
                            Text(index == 0 ? "Primary" : "Secondary")
                                .font(.caption.weight(.semibold))
                            Spacer()
                            Text("\(window.usedPercent.formatted(.number.precision(.fractionLength(1))))% used")
                                .font(.caption.monospacedDigit())
                                .foregroundStyle(.secondary)
                        }
                        ProgressView(value: min(max(window.usedPercent, 0), 100), total: 100)
                            .tint(Color.severity(usedPercent: window.usedPercent))
                        Text(window.resetsInMinutes.map { "Resets in \(Self.resetLabel(minutes: $0))" } ?? "No reset time available")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
            }

            HStack(spacing: 14) {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Weekly projection")
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(.secondary)
                    Text(self.provider.projectedWeeklySpendUSD.map(Self.currency) ?? "—")
                        .font(.body.monospacedDigit().weight(.semibold))
                }
                VStack(alignment: .leading, spacing: 4) {
                    Text("Freshness")
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(.secondary)
                    Text(self.provider.lastRefreshLabel)
                        .font(.body)
                }
            }

            VStack(alignment: .leading, spacing: 4) {
                Text(self.provider.sourceLabel)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                if let identityLabel = self.provider.identityLabel {
                    Text(identityLabel)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                if let warning = self.provider.warnings.first {
                    Text(warning)
                        .font(.caption)
                        .foregroundStyle(self.stateColor)
                }
            }
        }
        .padding(18)
        .menuCardBackground(opacity: 0.04, cornerRadius: 16)
    }

    private var stateColor: Color {
        switch self.provider.visualState {
        case "error": return .accentError
        case "incident", "degraded": return .warning
        case "stale": return .secondary
        default: return .primary
        }
    }

    private static func currency(_ value: Double) -> String {
        FormatHelpers.formatUSD(value)
    }

    private static func resetLabel(minutes: Int) -> String {
        if minutes >= 1_440 {
            return "\(minutes / 1_440)d \((minutes % 1_440) / 60)h"
        }
        if minutes >= 60 {
            return "\(minutes / 60)h \(minutes % 60)m"
        }
        return "\(minutes)m"
    }
}

private struct WindowLiveMonitorDetailSection: View {
    let provider: LiveMonitorProvider
    let density: LiveMonitorDensity
    let hiddenPanels: Set<LiveMonitorPanelID>

    var body: some View {
        VStack(alignment: .leading, spacing: self.sectionSpacing) {
            WindowSectionHeader(
                title: "\(self.provider.title) Details",
                subtitle: self.provider.lastRefreshLabel
            )

            LazyVGrid(columns: [GridItem(.adaptive(minimum: 260), spacing: self.gridSpacing)], spacing: self.gridSpacing) {
                if !self.hiddenPanels.contains(.activeBlock), let block = self.provider.activeBlock {
                    WindowLiveMonitorDetailCard(title: "Active Block", density: self.density) {
                        let totalTokens = block.tokens.total
                        Text(Self.compactNumber(totalTokens))
                            .font(.system(size: 28, weight: .semibold).monospacedDigit())
                        Text("\(block.entryCount) entries · ends \(Self.shortTime(block.end))")
                            .font(self.captionFont)
                            .foregroundStyle(.secondary)
                        if let quota = block.quota {
                            ProgressView(value: min(quota.projectedPercent, 1.0), total: 1.0)
                                .tint(Color.severity(code: quota.projectedSeverity))
                            Text("\(Int(quota.projectedPercent * 100))% projected · \(Self.compactNumber(quota.remainingTokens)) tokens left")
                                .font(self.captionFont)
                                .foregroundStyle(.secondary)
                        }
                    }
                }

                if !self.hiddenPanels.contains(.quotaSuggestions),
                   let suggestions = WindowQuotaSuggestionsModel.make(suggestions: self.provider.quotaSuggestions) {
                    WindowLiveMonitorDetailCard(title: "Suggested Quotas", density: self.density) {
                        WindowQuotaSuggestionRows(model: suggestions)
                    }
                }

                if !self.hiddenPanels.contains(.depletionForecast),
                   let forecast = WindowDepletionForecastModel.make(forecast: self.provider.depletionForecast) {
                    WindowLiveMonitorDetailCard(title: "Depletion Forecast", density: self.density) {
                        WindowDepletionForecastCardBody(model: forecast)
                    }
                }

                if let predictive = PredictiveInsightsSummaryModel.make(insights: self.provider.predictiveInsights) {
                    WindowLiveMonitorDetailCard(title: "Predictive Insights", density: self.density) {
                        WindowPredictiveInsightsCardBody(model: predictive, density: self.density)
                    }
                }

                if !self.hiddenPanels.contains(.contextWindow), let context = self.provider.contextWindow {
                    WindowLiveMonitorDetailCard(title: "Context Window", density: self.density) {
                        Text(Self.compactNumber(context.totalInputTokens))
                            .font(.system(size: 28, weight: .semibold).monospacedDigit())
                        Text("of \(Self.compactNumber(context.contextWindowSize)) · \(Int(context.pct * 100))%")
                            .font(self.captionFont)
                            .foregroundStyle(.secondary)
                        ProgressView(value: context.pct, total: 1.0)
                            .tint(Color.severity(code: context.severity))
                    }
                }

                if !self.hiddenPanels.contains(.recentSession), let session = self.provider.recentSession {
                    WindowLiveMonitorDetailCard(title: "Recent Session", density: self.density) {
                        Text(session.displayName)
                            .font(.headline)
                        Text("\(session.turns) turns · \(session.durationMinutes)m · \(Self.currency(session.costUSD))")
                            .font(self.captionFont)
                            .foregroundStyle(.secondary)
                        if let model = session.model {
                            Text(model)
                                .font(self.captionFont)
                                .foregroundStyle(.secondary)
                        }
                    }
                }
            }

            if !self.hiddenPanels.contains(.warnings), !self.provider.warnings.isEmpty {
                WindowLiveMonitorDetailCard(title: "Warnings", density: self.density) {
                    ForEach(self.provider.warnings, id: \.self) { warning in
                        Text("• \(warning)")
                            .font(.body)
                    }
                }
            }
        }
    }

    private var sectionSpacing: CGFloat { self.density == .compact ? 10 : 14 }
    private var gridSpacing: CGFloat { self.density == .compact ? 12 : 16 }
    private var captionFont: Font { self.density == .compact ? .caption2 : .caption }

    private static func shortTime(_ iso: String) -> String {
        liveMonitorShortTime(iso)
    }

    private static func compactNumber(_ value: Int) -> String {
        if abs(value) >= 1_000_000 {
            return String(format: "%.1fM", Double(value) / 1_000_000)
        }
        if abs(value) >= 1_000 {
            return String(format: "%.1fK", Double(value) / 1_000)
        }
        return "\(value)"
    }

    private static func currency(_ value: Double) -> String {
        FormatHelpers.formatUSD(value)
    }
}

private struct WindowLiveMonitorDetailCard<Content: View>: View {
    let title: String
    let density: LiveMonitorDensity
    let content: Content

    init(title: String, density: LiveMonitorDensity = .expanded, @ViewBuilder content: () -> Content) {
        self.title = title
        self.density = density
        self.content = content()
    }

    var body: some View {
        VStack(alignment: .leading, spacing: self.density == .compact ? 8 : 10) {
            Text(self.title)
                .font((self.density == .compact ? Font.caption2 : .caption).weight(.semibold))
                .foregroundStyle(.secondary)
            self.content
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(self.density == .compact ? 14 : 18)
        .menuCardBackground(opacity: 0.04, cornerRadius: 16)
    }
}

struct WindowOverviewDesktopAnalyticsModel {
    let providerComparisonItems: [ProviderMenuProjection]
    let heatmapCells: [ProviderHeatmapCell]
    let modelRows: [ProviderModelRow]
    let dailyModelRows: [ProviderDailyModelRow]
    let recentSessions: [ProviderSession]
    let weeklyProjection: WindowOverviewWeeklyProjectionModel?

    var showsProviderComparison: Bool {
        self.providerComparisonItems.filter { !$0.dailyCosts.isEmpty }.count >= 2
    }

    var showsHeatmap: Bool {
        self.heatmapCells.contains { $0.turns > 0 }
    }

    var showsModelMix: Bool {
        !self.modelRows.isEmpty
    }

    var showsRecentSessions: Bool {
        !self.recentSessions.isEmpty
    }

    static func make(projection: OverviewMenuProjection) -> Self {
        Self(
            providerComparisonItems: projection.items,
            heatmapCells: self.aggregateHeatmapCells(from: projection.items),
            modelRows: self.aggregateModelRows(from: projection.items),
            dailyModelRows: self.aggregateDailyModelRows(from: projection.items),
            recentSessions: self.aggregateRecentSessions(from: projection.items),
            weeklyProjection: WindowOverviewWeeklyProjectionModel.make(items: projection.items)
        )
    }

    private static func aggregateHeatmapCells(from items: [ProviderMenuProjection]) -> [ProviderHeatmapCell] {
        var grouped: [String: Int] = [:]
        for item in items {
            for cell in item.activityHeatmap {
                let key = "\(cell.dayOfWeek)-\(cell.hour)"
                grouped[key, default: 0] += cell.turns
            }
        }

        return grouped.compactMap { key, turns in
            let parts = key.split(separator: "-", maxSplits: 1).map(String.init)
            guard
                parts.count == 2,
                let day = Int(parts[0]),
                let hour = Int(parts[1])
            else {
                return nil
            }
            return ProviderHeatmapCell(dayOfWeek: day, hour: hour, turns: turns)
        }
        .sorted { lhs, rhs in
            if lhs.dayOfWeek == rhs.dayOfWeek {
                return lhs.hour < rhs.hour
            }
            return lhs.dayOfWeek < rhs.dayOfWeek
        }
    }

    private static func aggregateModelRows(from items: [ProviderMenuProjection]) -> [ProviderModelRow] {
        struct Aggregate {
            var costUSD: Double = 0
            var input: Int = 0
            var output: Int = 0
            var cacheRead: Int = 0
            var cacheCreation: Int = 0
            var reasoningOutput: Int = 0
            var turns: Int = 0
        }

        var grouped: [String: Aggregate] = [:]
        for item in items {
            for row in item.byModel {
                grouped[row.model, default: Aggregate()].costUSD += row.costUSD
                grouped[row.model, default: Aggregate()].input += row.input
                grouped[row.model, default: Aggregate()].output += row.output
                grouped[row.model, default: Aggregate()].cacheRead += row.cacheRead
                grouped[row.model, default: Aggregate()].cacheCreation += row.cacheCreation
                grouped[row.model, default: Aggregate()].reasoningOutput += row.reasoningOutput
                grouped[row.model, default: Aggregate()].turns += row.turns
            }
        }

        return grouped.map { model, aggregate in
            ProviderModelRow(
                model: model,
                costUSD: aggregate.costUSD,
                input: aggregate.input,
                output: aggregate.output,
                cacheRead: aggregate.cacheRead,
                cacheCreation: aggregate.cacheCreation,
                reasoningOutput: aggregate.reasoningOutput,
                turns: aggregate.turns
            )
        }
        .sorted { lhs, rhs in
            if lhs.costUSD == rhs.costUSD {
                return lhs.model < rhs.model
            }
            return lhs.costUSD > rhs.costUSD
        }
    }

    private static func aggregateDailyModelRows(
        from items: [ProviderMenuProjection]
    ) -> [ProviderDailyModelRow] {
        struct Aggregate {
            var costUSD: Double = 0
            var input: Int = 0
            var output: Int = 0
            var cacheRead: Int = 0
            var cacheCreation: Int = 0
            var reasoningOutput: Int = 0
            var turns: Int = 0
        }

        var grouped: [String: Aggregate] = [:]
        for item in items {
            for row in item.dailyByModel {
                let key = "\(row.day)|\(row.model)"
                var entry = grouped[key, default: Aggregate()]
                entry.costUSD += row.costUSD
                entry.input += row.input
                entry.output += row.output
                entry.cacheRead += row.cacheRead
                entry.cacheCreation += row.cacheCreation
                entry.reasoningOutput += row.reasoningOutput
                entry.turns += row.turns
                grouped[key] = entry
            }
        }

        return grouped.compactMap { key, aggregate -> ProviderDailyModelRow? in
            let parts = key.split(separator: "|", maxSplits: 1).map(String.init)
            guard parts.count == 2 else { return nil }
            return ProviderDailyModelRow(
                day: parts[0],
                model: parts[1],
                costUSD: aggregate.costUSD,
                input: aggregate.input,
                output: aggregate.output,
                cacheRead: aggregate.cacheRead,
                cacheCreation: aggregate.cacheCreation,
                reasoningOutput: aggregate.reasoningOutput,
                turns: aggregate.turns
            )
        }
        .sorted { lhs, rhs in
            if lhs.day == rhs.day {
                if lhs.costUSD == rhs.costUSD {
                    return lhs.model < rhs.model
                }
                return lhs.costUSD > rhs.costUSD
            }
            return lhs.day < rhs.day
        }
    }

    private static func aggregateRecentSessions(from items: [ProviderMenuProjection]) -> [ProviderSession] {
        items
            .flatMap { item in
                item.recentSessions.map { session in
                    ProviderSession(
                        sessionID: "\(item.provider.rawValue)-\(session.sessionID)",
                        displayName: "\(item.title) · \(session.displayName)",
                        startedAt: session.startedAt,
                        durationMinutes: session.durationMinutes,
                        turns: session.turns,
                        costUSD: session.costUSD,
                        model: session.model
                    )
                }
            }
            .sorted { lhs, rhs in
                if lhs.startedAt == rhs.startedAt {
                    return lhs.costUSD > rhs.costUSD
                }
                return lhs.startedAt > rhs.startedAt
            }
    }
}

private struct SidebarNavigationRow: View {
    let item: AppNavigationItem
    let isSelected: Bool

    var body: some View {
        HStack(spacing: 10) {
            RoundedRectangle(cornerRadius: 8, style: .continuous)
                .fill(self.isSelected ? Color.primary.opacity(0.08) : Color.primary.opacity(0.04))
                .frame(width: 28, height: 28)
                .overlay {
                    Image(systemName: self.item.systemImage)
                        .font(.system(size: 13, weight: .semibold))
                        .foregroundStyle(self.isSelected ? Color.primary : Color.secondary)
                        .symbolRenderingMode(.monochrome)
                }

            VStack(alignment: .leading, spacing: 1) {
                Text(self.item.title)
                    .font(.body.weight(self.isSelected ? .semibold : .medium))
                    .lineLimit(1)
                Text(self.item.subtitle)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }

            Spacer(minLength: 0)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 8)
        .background(
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .fill(self.isSelected ? Color.primary.opacity(0.05) : Color.clear)
        )
        .overlay(
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .stroke(
                    self.isSelected ? Color.primary.opacity(0.08) : Color.clear,
                    lineWidth: 1
                )
        )
        .contentShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
    }
}

private struct WindowSectionHeader: View {
    let title: String
    let subtitle: String

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(self.title)
                .font(.title3.weight(.semibold))
            Text(self.subtitle)
                .font(.callout)
                .foregroundStyle(.secondary)
        }
    }
}

private struct WindowOverviewHistoryCard: View {
    let projection: OverviewMenuProjection

    var body: some View {
        let summary = WindowOverviewHistorySummaryModel.make(fractions: self.projection.historyFractions)

        VStack(alignment: .leading, spacing: 12) {
            VStack(alignment: .leading, spacing: 4) {
                Text("Last 7 days")
                    .font(.headline)
                Text("Daily activity across providers.")
                    .font(.callout)
                    .foregroundStyle(.secondary)
            }

            HistoryBarChart(
                fractions: self.projection.historyFractions,
                showsHeader: false,
                inset: true
            )

            if let summary {
                HStack(spacing: 10) {
                    WindowOverviewHistoryStatRow(title: "Peak", value: summary.peakLabel)
                    WindowOverviewHistoryStatRow(title: "Today", value: summary.todayLabel)
                    WindowOverviewHistoryStatRow(title: "Active", value: summary.activeDaysLabel)
                }
            }
        }
        .padding(18)
        .frame(maxWidth: .infinity, alignment: .leading)
        .menuCardBackground(opacity: 0.04, cornerRadius: 16)
    }
}

struct WindowOverviewWeeklyProjectionModel: Equatable {
    let actualCostUSD: Double
    let projectedCostUSD: Double
    let elapsedFraction: Double
    let providerCount: Int
    let leadProviderTitle: String?

    var projectedLabel: String {
        WindowOverviewProviderCostInsightsModel.currencyLabel(self.projectedCostUSD)
    }

    var actualLabel: String {
        WindowOverviewProviderCostInsightsModel.currencyLabel(self.actualCostUSD)
    }

    var elapsedLabel: String {
        "\(Int((self.elapsedFraction * 100).rounded()))% of week elapsed"
    }

    var progressFraction: Double {
        guard self.projectedCostUSD > 0 else { return 0 }
        return max(0, min(1, self.actualCostUSD / self.projectedCostUSD))
    }

    var providerLabel: String {
        if let leadProviderTitle, !leadProviderTitle.isEmpty {
            return "Led by \(leadProviderTitle)"
        }
        return "\(self.providerCount) provider\(self.providerCount == 1 ? "" : "s") contributing"
    }

    static func make(items: [ProviderMenuProjection]) -> Self? {
        struct Entry {
            let title: String
            let projectedCostUSD: Double
            let elapsedFraction: Double
        }

        let entries = items.compactMap { item -> Entry? in
            guard
                let projected = item.weeklyProjectedCostUSD,
                projected > 0
            else {
                return nil
            }

            let weeklyLane = item.laneDetails.first {
                $0.title.localizedCaseInsensitiveContains("week")
            } ?? item.laneDetails.dropFirst().first
            let elapsed = weeklyLane.flatMap { detail in
                WindowOverviewQuotaWindowsModel.Lane.elapsedFraction(
                    resetMinutes: detail.resetMinutes,
                    windowMinutes: detail.windowMinutes
                )
            } ?? 0

            return Entry(
                title: item.title,
                projectedCostUSD: projected,
                elapsedFraction: elapsed
            )
        }

        guard !entries.isEmpty else { return nil }

        let projectedCostUSD = entries.reduce(0) { $0 + $1.projectedCostUSD }
        let actualCostUSD = entries.reduce(0) { $0 + ($1.projectedCostUSD * $1.elapsedFraction) }
        let weightedElapsed = projectedCostUSD > 0
            ? entries.reduce(0) { $0 + ($1.projectedCostUSD * $1.elapsedFraction) } / projectedCostUSD
            : 0
        let leadProviderTitle = entries.max { lhs, rhs in
            lhs.projectedCostUSD < rhs.projectedCostUSD
        }?.title

        return Self(
            actualCostUSD: actualCostUSD,
            projectedCostUSD: projectedCostUSD,
            elapsedFraction: max(0, min(1, weightedElapsed)),
            providerCount: entries.count,
            leadProviderTitle: leadProviderTitle
        )
    }
}

private struct WindowOverviewWeeklyProjectionCard: View {
    let model: WindowOverviewWeeklyProjectionModel

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            VStack(alignment: .leading, spacing: 4) {
                Text("Projected week")
                    .font(.headline)
                Text("Actual burn vs expected reset spend.")
                    .font(.callout)
                    .foregroundStyle(.secondary)
            }

            HStack(alignment: .firstTextBaseline, spacing: 10) {
                Text(self.model.projectedLabel)
                    .font(.system(size: 28, weight: .bold, design: .rounded).monospacedDigit())
                Spacer(minLength: 12)
                Text(self.model.elapsedLabel)
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.secondary)
            }

            VStack(alignment: .leading, spacing: 8) {
                HStack(alignment: .lastTextBaseline, spacing: 8) {
                    Text("Actual so far")
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(Color.primary.opacity(0.68))
                        .textCase(.uppercase)
                        .tracking(0.4)
                    Spacer(minLength: 8)
                    Text(self.model.actualLabel)
                        .font(.callout.monospacedDigit().weight(.semibold))
                }

                GeometryReader { geometry in
                    let width = max(geometry.size.width, 1)
                    let fillWidth = width * self.model.progressFraction
                    ZStack(alignment: .leading) {
                        Capsule(style: .continuous)
                            .fill(Color.primary.opacity(0.08))
                        Capsule(style: .continuous)
                            .fill(Color.primary.opacity(0.72))
                            .frame(width: max(fillWidth, 6))
                    }
                }
                .frame(height: 8)
            }

            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Text(self.model.providerLabel)
                    .font(.callout.weight(.medium))
                Spacer(minLength: 8)
                Text("\(self.model.providerCount) tracked")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
        .padding(18)
        .frame(maxWidth: .infinity, alignment: .leading)
        .menuCardBackground(opacity: 0.04, cornerRadius: 16)
    }
}

struct WindowOverviewHistorySummaryModel: Equatable {
    let peakLabel: String
    let todayLabel: String
    let activeDaysLabel: String

    static func make(fractions: [Double]) -> Self? {
        let entries = HistoryBarChart.entries(from: fractions)
        guard !entries.isEmpty else { return nil }
        let peak = entries.max { lhs, rhs in
            if lhs.fraction == rhs.fraction {
                return lhs.index > rhs.index
            }
            return lhs.fraction < rhs.fraction
        }
        let today = entries.last
        let activeDays = entries.filter { $0.fraction > 0.08 }.count

        guard let peak, let today else { return nil }
        return Self(
            peakLabel: "\(peak.label) \(Self.percentLabel(peak.fraction))",
            todayLabel: Self.percentLabel(today.fraction),
            activeDaysLabel: "\(activeDays)/\(entries.count)"
        )
    }

    private static func percentLabel(_ fraction: Double) -> String {
        "\(Int((fraction * 100).rounded()))%"
    }
}

private struct WindowOverviewHistoryStatRow: View {
    let title: String
    let value: String

    var body: some View {
        VStack(alignment: .leading, spacing: 3) {
            Text(self.title)
                .font(.caption2.weight(.semibold))
                .foregroundStyle(.secondary)
                .textCase(.uppercase)
                .tracking(0.4)
            Text(self.value)
                .font(.callout.monospacedDigit().weight(.semibold))
                .foregroundStyle(.primary)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, 12)
        .padding(.vertical, 10)
        .menuCardBackground(opacity: 0.03, cornerRadius: 10)
    }
}

struct WindowProviderMetricSummary: Equatable {
    let title: String
    let value: String
    let qualifier: String
    let detail: String

    static func make(
        item: ProviderMenuProjection,
        showUsedValues: Bool
    ) -> Self {
        guard let lane = item.laneDetails.first,
              let remainingPercent = lane.remainingPercent else {
            return Self(
                title: "Session availability",
                value: "Unavailable",
                qualifier: "Live quota",
                detail: self.unavailableDetail(for: item)
            )
        }

        let shownPercent = showUsedValues
            ? max(0, min(100, 100 - remainingPercent))
            : remainingPercent

        return Self(
            title: showUsedValues ? "Session usage" : "Session remaining",
            value: "\(shownPercent)%",
            qualifier: showUsedValues ? "Used" : "Remaining",
            detail: lane.resetDetail ?? lane.summary
        )
    }

    private static func unavailableDetail(for item: ProviderMenuProjection) -> String {
        let sourceLabel = item.sourceLabel.lowercased()
        if item.isShowingCachedData {
            return "Showing last known provider data"
        }
        if sourceLabel.contains("oauth") {
            return "OAuth session data is unavailable"
        }
        if sourceLabel.contains("web") {
            return "Web session data is unavailable"
        }
        if sourceLabel.contains("cli") {
            return "CLI session data is unavailable"
        }
        return "Live session data is unavailable"
    }
}

struct WindowOverviewProviderCostInsightsModel: Equatable {
    struct Stat: Equatable, Identifiable {
        let title: String
        let value: String
        let detail: String?

        var id: String { self.title }
    }

    let stats: [Stat]
    let mixLabel: String?

    static func make(item: ProviderMenuProjection) -> Self {
        var stats: [Stat] = []

        if let today = item.todayBreakdown, !today.isEmpty {
            stats.append(
                Stat(
                    title: "Today tokens",
                    value: Self.compactTokenCount(today.total),
                    detail: Self.costDetail(costUSD: item.todayCostUSD)
                )
            )
        }

        if let trailing = item.last30DaysBreakdown, !trailing.isEmpty {
            stats.append(
                Stat(
                    title: "30-day tokens",
                    value: Self.compactTokenCount(trailing.total),
                    detail: Self.costDetail(costUSD: item.last30DaysCostUSD)
                )
            )
        }

        if let hit = Self.cacheHitDetail(today: item.cacheHitRateToday, trailing: item.cacheHitRate30d) {
            stats.append(
                Stat(
                    title: "Cache hit rate",
                    value: hit.value,
                    detail: hit.detail
                )
            )
        }

        if let savings = item.cacheSavings30dUSD, savings > 0 {
            stats.append(
                Stat(
                    title: "Cache savings",
                    value: Self.currencyLabel(savings),
                    detail: "Last 30 days"
                )
            )
        }

        let mixLabel = Self.mixLabel(today: item.todayBreakdown, trailing: item.last30DaysBreakdown)
        return Self(stats: stats, mixLabel: mixLabel)
    }

    private static func costDetail(costUSD: Double?) -> String? {
        guard let costUSD else { return nil }
        return currencyLabel(costUSD)
    }

    private static func cacheHitDetail(today: Double?, trailing: Double?) -> (value: String, detail: String?)? {
        if let today {
            return (
                value: percentLabel(today),
                detail: trailing.map { "30-day avg \(percentLabel($0))" } ?? "Today"
            )
        }

        if let trailing {
            return (
                value: percentLabel(trailing),
                detail: "Last 30 days"
            )
        }

        return nil
    }

    private static func mixLabel(today: TokenBreakdown?, trailing: TokenBreakdown?) -> String? {
        let selected: (prefix: String, breakdown: TokenBreakdown)? = if let today, !today.isEmpty {
            ("Today mix", today)
        } else if let trailing, !trailing.isEmpty {
            ("30-day mix", trailing)
        } else {
            nil
        }

        guard let selected else { return nil }
        let breakdown = selected.breakdown
        let parts = [
            breakdown.input > 0 ? "\(compactTokenCount(breakdown.input)) in" : nil,
            breakdown.output > 0 ? "\(compactTokenCount(breakdown.output)) out" : nil,
            breakdown.cacheRead > 0 ? "\(compactTokenCount(breakdown.cacheRead)) cache read" : nil,
            breakdown.cacheCreation > 0 ? "\(compactTokenCount(breakdown.cacheCreation)) cache write" : nil,
            breakdown.reasoningOutput > 0 ? "\(compactTokenCount(breakdown.reasoningOutput)) reasoning" : nil,
        ]
        .compactMap { $0 }

        guard !parts.isEmpty else { return nil }
        return "\(selected.prefix): \(parts.joined(separator: " · "))"
    }

    static func compactTokenCount(_ count: Int) -> String {
        let value = Double(count)
        if value >= 1_000_000_000 {
            return String(format: "%.1fB", value / 1_000_000_000)
        }
        if value >= 1_000_000 {
            return String(format: "%.1fM", value / 1_000_000)
        }
        if value >= 1_000 {
            return String(format: "%.1fK", value / 1_000)
        }
        return "\(count)"
    }

    private static func percentLabel(_ value: Double) -> String {
        String(format: "%.1f%%", max(0, min(1, value)) * 100)
    }

    static func currencyLabel(_ value: Double) -> String {
        FormatHelpers.formatUSD(value)
    }
}

struct WindowQuotaSuggestionsModel: Equatable {
    struct Item: Equatable, Identifiable {
        let key: String
        let label: String
        let value: String
        let isRecommended: Bool

        var id: String { self.key }
    }

    let sampleCount: Int
    let sampleContextLabel: String
    let items: [Item]
    let note: String?

    static func make(suggestions: QuotaSuggestions?) -> Self? {
        guard let suggestions, !suggestions.levels.isEmpty else { return nil }
        return Self(
            sampleCount: suggestions.sampleCount,
            sampleContextLabel: self.sampleContextLabel(for: suggestions),
            items: suggestions.levels.map { level in
                Item(
                    key: level.key,
                    label: level.label,
                    value: Self.compactTokenCount(level.limitTokens),
                    isRecommended: level.key == suggestions.recommendedKey
                )
            },
            note: suggestions.note
        )
    }

    private static func sampleContextLabel(for suggestions: QuotaSuggestions) -> String {
        if let sampleLabel = suggestions.sampleLabel?.trimmingCharacters(in: .whitespacesAndNewlines),
           !sampleLabel.isEmpty {
            return sampleLabel
        }

        var fragments = ["\(suggestions.sampleCount) completed blocks"]
        if let populationCount = suggestions.populationCount {
            fragments.append("from \(populationCount)")
        }
        if let sampleStrategy = suggestions.sampleStrategy?.trimmingCharacters(in: .whitespacesAndNewlines),
           !sampleStrategy.isEmpty {
            fragments.append(sampleStrategy.replacingOccurrences(of: "_", with: " "))
        }
        return fragments.joined(separator: " · ")
    }

    private static func compactTokenCount(_ value: Int) -> String {
        let absolute = abs(value)
        if absolute >= 1_000_000 {
            return String(format: "%.1fM", Double(value) / 1_000_000)
        }
        if absolute >= 1_000 {
            return String(format: "%.1fK", Double(value) / 1_000)
        }
        return "\(value)"
    }
}

struct PredictiveInsightsSummaryModel: Equatable {
    struct RollingHourBurn: Equatable {
        let tokensPerMinuteLabel: String
        let costPerHourLabel: String
        let coverageLabel: String
        let tierLabel: String?
    }

    struct HistoricalEnvelope: Equatable {
        let sampleLabel: String
        let tokensRangeLabel: String
        let costRangeLabel: String
        let turnsRangeLabel: String
        let averagesLabel: String
    }

    struct LimitHitAnalysis: Equatable {
        let riskLevel: String
        let summaryLabel: String
        let hitRateLabel: String
        let thresholdLabel: String?
        let activityLabel: String?
    }

    let rollingHourBurn: RollingHourBurn?
    let historicalEnvelope: HistoricalEnvelope?
    let limitHitAnalysis: LimitHitAnalysis?

    var hasContent: Bool {
        self.rollingHourBurn != nil || self.historicalEnvelope != nil || self.limitHitAnalysis != nil
    }

    static func make(insights: LivePredictiveInsights?) -> Self? {
        guard let insights else { return nil }
        let model = Self(
            rollingHourBurn: insights.rollingHourBurn.map { burn in
                RollingHourBurn(
                    tokensPerMinuteLabel: "\(self.compactValue(burn.tokensPerMin)) tok/min",
                    costPerHourLabel: "\(self.currencyLabel(nanos: burn.costPerHourNanos))/h",
                    coverageLabel: "Coverage \(self.compactValue(burn.coverageMinutes))m",
                    tierLabel: burn.tier?.uppercased()
                )
            },
            historicalEnvelope: insights.historicalEnvelope.map { envelope in
                HistoricalEnvelope(
                    sampleLabel: "\(envelope.sampleCount) historical samples",
                    tokensRangeLabel: "\(self.compactValue(envelope.tokens.p50))-\(self.compactValue(envelope.tokens.p95)) tok",
                    costRangeLabel: "\(self.currencyLabel(usd: envelope.costUSD.p50))-\(self.currencyLabel(usd: envelope.costUSD.p95))",
                    turnsRangeLabel: "\(self.compactValue(envelope.turns.p50))-\(self.compactValue(envelope.turns.p95)) turns",
                    averagesLabel: "Avg \(self.compactValue(envelope.tokens.average)) tok · \(self.currencyLabel(usd: envelope.costUSD.average)) · \(self.compactValue(envelope.turns.average)) turns"
                )
            },
            limitHitAnalysis: insights.limitHitAnalysis.map { analysis in
                LimitHitAnalysis(
                    riskLevel: analysis.riskLevel,
                    summaryLabel: analysis.summaryLabel,
                    hitRateLabel: "\(analysis.hitCount)/\(analysis.sampleCount) hits · \(self.percentLabel(analysis.hitRate))",
                    thresholdLabel: self.thresholdLabel(for: analysis),
                    activityLabel: self.activityLabel(for: analysis)
                )
            }
        )
        return model.hasContent ? model : nil
    }

    private static func thresholdLabel(for analysis: LivePredictiveLimitHitAnalysis) -> String? {
        var fragments: [String] = []
        if let thresholdTokens = analysis.thresholdTokens {
            fragments.append("\(self.compactValue(Double(thresholdTokens))) tok")
        }
        if let thresholdPercent = analysis.thresholdPercent {
            fragments.append(self.percentLabel(thresholdPercent))
        }
        guard !fragments.isEmpty else { return nil }
        return "Threshold " + fragments.joined(separator: " · ")
    }

    private static func activityLabel(for analysis: LivePredictiveLimitHitAnalysis) -> String? {
        switch (analysis.activeCurrentHit ?? false, analysis.activeProjectedHit ?? false) {
        case (true, true):
            return "Active now + projected"
        case (true, false):
            return "Active now"
        case (false, true):
            return "Projected to hit"
        default:
            return nil
        }
    }

    fileprivate static func compactValue(_ value: Double) -> String {
        let absolute = abs(value)
        if absolute >= 1_000_000 {
            return String(format: "%.1fM", value / 1_000_000)
        }
        if absolute >= 1_000 {
            return String(format: "%.1fK", value / 1_000)
        }
        if value.rounded() == value {
            return String(format: "%.0f", value)
        }
        return String(format: "%.1f", value)
    }

    fileprivate static func currencyLabel(nanos: Int) -> String {
        self.currencyLabel(usd: Double(nanos) / 1_000_000_000)
    }

    fileprivate static func currencyLabel(usd: Double) -> String {
        switch abs(usd) {
        case 100...:
            return String(format: "$%.0f", usd)
        case 10...:
            return String(format: "$%.1f", usd)
        case 1...:
            return String(format: "$%.2f", usd)
        default:
            return String(format: "$%.3f", usd)
        }
    }

    fileprivate static func percentLabel(_ value: Double) -> String {
        let normalized = value <= 1.0 ? value * 100 : value
        return "\(Int(normalized.rounded()))%"
    }
}

private struct WindowQuotaSuggestionRows: View {
    let model: WindowQuotaSuggestionsModel

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text(self.model.sampleContextLabel)
                .font(.caption)
                .foregroundStyle(.secondary)

            VStack(spacing: 8) {
                ForEach(self.model.items) { item in
                    HStack(alignment: .firstTextBaseline, spacing: 8) {
                        Text(item.label)
                            .font(.caption.weight(.semibold))
                            .foregroundStyle(.secondary)
                        if item.isRecommended {
                            Text("RECOMMENDED")
                                .font(.caption2.weight(.semibold))
                                .foregroundStyle(Color.primary.opacity(0.72))
                        }
                        Spacer(minLength: 8)
                        Text(item.value)
                            .font(.callout.monospacedDigit().weight(.semibold))
                    }
                }
            }

            if let note = self.model.note {
                Text(note)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
    }
}

struct WindowDepletionForecastModel: Equatable {
    struct Signal: Equatable, Identifiable {
        let id: String
        let title: String
        let valueLabel: String
        let timingLabel: String?
        let remainingLabel: String?
        let paceLabel: String?
    }

    let primary: Signal
    let secondary: [Signal]
    let summaryLabel: String
    let severity: String
    let note: String?

    static func make(forecast: DepletionForecast?) -> Self? {
        guard let forecast else { return nil }
        return Self(
            primary: self.signal(from: forecast.primarySignal),
            secondary: forecast.secondarySignals.map(self.signal(from:)),
            summaryLabel: forecast.summaryLabel,
            severity: forecast.severity,
            note: forecast.note
        )
    }

    fileprivate static func signal(from signal: DepletionForecastSignal) -> Signal {
        let percent = signal.projectedPercent ?? signal.usedPercent
        let valueLabel = "\(Int(percent.rounded()))% \(signal.projectedPercent != nil ? "projected" : "used")"
        let timingLabel: String?
        if let resetsInMinutes = signal.resetsInMinutes {
            timingLabel = "Resets in \(Self.resetLabel(minutes: resetsInMinutes))"
        } else if let endTime = signal.endTime {
            timingLabel = "Ends \(Self.shortTime(endTime))"
        } else {
            timingLabel = nil
        }
        let remainingLabel: String?
        if let remainingTokens = signal.remainingTokens {
            remainingLabel = "\(Self.compactNumber(remainingTokens)) tokens left"
        } else if let remainingPercent = signal.remainingPercent {
            remainingLabel = "\(Int(remainingPercent.rounded()))% remaining"
        } else {
            remainingLabel = nil
        }

        return Signal(
            id: signal.id,
            title: signal.title,
            valueLabel: valueLabel,
            timingLabel: timingLabel,
            remainingLabel: remainingLabel,
            paceLabel: signal.paceLabel
        )
    }

    private static func compactNumber(_ value: Int) -> String {
        let absolute = abs(value)
        if absolute >= 1_000_000 {
            return String(format: "%.1fM", Double(value) / 1_000_000)
        }
        if absolute >= 1_000 {
            return String(format: "%.1fK", Double(value) / 1_000)
        }
        return "\(value)"
    }

    private static func resetLabel(minutes: Int) -> String {
        if minutes >= 1_440 {
            return "\(minutes / 1_440)d \((minutes % 1_440) / 60)h"
        }
        if minutes >= 60 {
            return "\(minutes / 60)h \(minutes % 60)m"
        }
        return "\(minutes)m"
    }

    private static func shortTime(_ iso: String) -> String {
        liveMonitorShortTime(iso)
    }
}

private struct WindowDepletionForecastCardBody: View {
    let model: WindowDepletionForecastModel

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text(self.model.primary.title)
                .font(.headline)
            Text(self.model.summaryLabel)
                .font(.caption)
                .foregroundStyle(.secondary)
                .fixedSize(horizontal: false, vertical: true)

            VStack(alignment: .leading, spacing: 8) {
                HStack(alignment: .firstTextBaseline, spacing: 8) {
                    Text(self.model.primary.valueLabel)
                        .font(.callout.monospacedDigit().weight(.semibold))
                    if let paceLabel = self.model.primary.paceLabel {
                        Text(paceLabel)
                            .font(.caption2.weight(.semibold))
                            .textCase(.uppercase)
                            .foregroundStyle(self.toneColor.opacity(0.9))
                    }
                }

                if let timingLabel = self.model.primary.timingLabel {
                    Text(timingLabel)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                if let remainingLabel = self.model.primary.remainingLabel {
                    Text(remainingLabel)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }

            if !self.model.secondary.isEmpty {
                VStack(alignment: .leading, spacing: 8) {
                    Text("SUPPORTING SIGNALS")
                        .font(.caption2.weight(.semibold))
                        .foregroundStyle(Color.primary.opacity(0.72))
                        .tracking(0.4)

                    ForEach(self.model.secondary) { signal in
                        VStack(alignment: .leading, spacing: 2) {
                            HStack(alignment: .firstTextBaseline, spacing: 8) {
                                Text(signal.title)
                                    .font(.caption.weight(.semibold))
                                    .foregroundStyle(.secondary)
                                Spacer(minLength: 8)
                                Text(signal.valueLabel)
                                    .font(.caption.monospacedDigit())
                            }
                            Text([signal.timingLabel, signal.remainingLabel].compactMap { $0 }.joined(separator: " · "))
                                .font(.caption2)
                                .foregroundStyle(Color.primary.opacity(0.66))
                                .fixedSize(horizontal: false, vertical: true)
                        }
                    }
                }
            }

            if let note = self.model.note {
                Text(note)
                    .font(.caption2)
                    .foregroundStyle(Color.primary.opacity(0.66))
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
    }

    private var toneColor: Color {
        switch self.model.severity {
        case "danger":
            return .accentError
        case "warn":
            return .warning
        default:
            return .primary
        }
    }
}

private struct WindowOverviewDepletionForecastStrip: View {
    let model: WindowDepletionForecastModel

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Text("Depletion forecast")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(Color.primary.opacity(0.72))
                    .textCase(.uppercase)
                    .tracking(0.4)
                Spacer(minLength: 8)
                Text(self.model.primary.valueLabel)
                    .font(.caption.monospacedDigit())
                    .foregroundStyle(.secondary)
            }

            WindowDepletionForecastCardBody(model: self.model)
                .padding(.horizontal, 10)
                .padding(.vertical, 8)
                .menuCardBackground(opacity: 0.03, cornerRadius: 10)
        }
    }
}

struct WindowOverviewQuotaWindowsModel: Equatable {
    struct Lane: Equatable, Identifiable {
        let title: String
        let remainingPercent: Int
        let resetDetail: String
        let paceLabel: String?
        let elapsedFraction: Double?

        var id: String { self.title }
        var remainingLabel: String { "\(self.remainingPercent)%" }
        var remainingFraction: Double { Double(self.remainingPercent) / 100.0 }

        fileprivate static func make(detail: LaneDetailProjection) -> Self? {
            guard let remainingPercent = detail.remainingPercent else { return nil }
            return Self(
                title: detail.title,
                remainingPercent: remainingPercent,
                resetDetail: detail.resetDetail ?? detail.summary,
                paceLabel: detail.paceLabel,
                elapsedFraction: Self.elapsedFraction(
                    resetMinutes: detail.resetMinutes,
                    windowMinutes: detail.windowMinutes
                )
            )
        }

        fileprivate static func elapsedFraction(resetMinutes: Int?, windowMinutes: Int?) -> Double? {
            guard
                let resetMinutes,
                let windowMinutes,
                windowMinutes > 0
            else { return nil }
            return max(0, min(1, 1 - (Double(resetMinutes) / Double(windowMinutes))))
        }
    }

    let lanes: [Lane]

    var primary: Lane? { self.lanes.first }
    var secondary: Lane? { self.lanes.dropFirst().first }
    var chartLanes: [Lane] { self.lanes.filter { $0.elapsedFraction != nil } }

    static func make(item: ProviderMenuProjection) -> Self {
        Self(lanes: item.laneDetails.prefix(2).compactMap(Lane.make))
    }
}

private struct WindowOverviewProviderCostInsights: View {
    let model: WindowOverviewProviderCostInsightsModel

    private let columns = [
        GridItem(.flexible(minimum: 120), spacing: 8),
        GridItem(.flexible(minimum: 120), spacing: 8),
    ]

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            if !self.model.stats.isEmpty {
                LazyVGrid(columns: self.columns, spacing: 8) {
                    ForEach(self.model.stats) { stat in
                        VStack(alignment: .leading, spacing: 3) {
                            Text(stat.title)
                                .font(.caption2.weight(.semibold))
                                .foregroundStyle(.secondary)
                                .textCase(.uppercase)
                                .tracking(0.4)
                            Text(stat.value)
                                .font(.callout.monospacedDigit().weight(.semibold))
                            if let detail = stat.detail {
                                Text(detail)
                                    .font(.caption2)
                                    .foregroundStyle(Color.primary.opacity(0.66))
                                    .lineLimit(1)
                            }
                        }
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(.horizontal, 10)
                        .padding(.vertical, 8)
                        .menuCardBackground(opacity: 0.03, cornerRadius: 10)
                    }
                }
            }

            if let mixLabel = self.model.mixLabel {
                Text(mixLabel)
                    .font(.caption)
                    .foregroundStyle(Color.primary.opacity(0.7))
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
    }
}

private struct WindowOverviewQuotaWindowsStrip: View {
    let model: WindowOverviewQuotaWindowsModel

    private let columns = [
        GridItem(.flexible(minimum: 120), spacing: 8),
        GridItem(.flexible(minimum: 120), spacing: 8),
    ]

    var body: some View {
        if !self.model.lanes.isEmpty {
            VStack(alignment: .leading, spacing: 8) {
                HStack(alignment: .firstTextBaseline, spacing: 8) {
                    Text("Limits")
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(Color.primary.opacity(0.72))
                        .textCase(.uppercase)
                        .tracking(0.4)
                    Spacer(minLength: 8)
                    Text("Session + weekly quota")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }

                LazyVGrid(columns: self.columns, spacing: 8) {
                    ForEach(self.model.lanes) { lane in
                        WindowOverviewQuotaLaneCard(lane: lane)
                    }
                }
            }
        }
    }
}

private struct WindowOverviewQuotaLaneCard: View {
    let lane: WindowOverviewQuotaWindowsModel.Lane

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Text(self.lane.title)
                    .font(.caption2.weight(.semibold))
                    .foregroundStyle(.secondary)
                    .textCase(.uppercase)
                    .tracking(0.4)
                Spacer(minLength: 8)
                if let paceLabel = self.lane.paceLabel {
                    Text(paceLabel)
                        .font(.caption2.weight(.semibold))
                        .textCase(.uppercase)
                        .foregroundStyle(self.toneColor.opacity(0.9))
                }
            }

            HStack(alignment: .lastTextBaseline, spacing: 8) {
                Text(self.lane.remainingLabel)
                    .font(.system(size: 22, weight: .bold, design: .rounded).monospacedDigit())
                    .foregroundStyle(.primary)
                Text("remaining")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.secondary)
            }

            GeometryReader { geometry in
                let width = max(geometry.size.width, 1)
                let fillWidth = width * self.lane.remainingFraction
                ZStack(alignment: .leading) {
                    Capsule(style: .continuous)
                        .fill(Color.primary.opacity(0.08))
                    Capsule(style: .continuous)
                        .fill(self.toneColor)
                        .frame(width: max(fillWidth, 6))
                }
            }
            .frame(height: 8)

            Text(self.lane.resetDetail)
                .font(.caption)
                .foregroundStyle(Color.primary.opacity(0.72))
                .lineLimit(1)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, 12)
        .padding(.vertical, 10)
        .menuCardBackground(opacity: 0.03, cornerRadius: 12)
    }

    private var toneColor: Color {
        switch self.lane.remainingPercent {
        case ..<15:
            return Color.accentError.opacity(0.78)
        case ..<35:
            return Color.warning.opacity(0.76)
        default:
            return Color.primary.opacity(0.72)
        }
    }
}

private struct WindowOverviewQuotaBurnChart: View {
    let model: WindowOverviewQuotaWindowsModel

    var body: some View {
        if !self.model.chartLanes.isEmpty {
            VStack(alignment: .leading, spacing: 10) {
                HStack(alignment: .firstTextBaseline, spacing: 8) {
                    Text("Burn to reset")
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(Color.primary.opacity(0.72))
                        .textCase(.uppercase)
                        .tracking(0.4)
                    Spacer(minLength: 8)
                    Text("Remaining vs time left")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }

                VStack(spacing: 10) {
                    ForEach(self.model.chartLanes) { lane in
                        WindowOverviewQuotaBurnRow(lane: lane)
                    }
                }
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 10)
            .menuCardBackground(opacity: 0.03, cornerRadius: 10)
        }
    }
}

private struct WindowOverviewQuotaBurnRow: View {
    let lane: WindowOverviewQuotaWindowsModel.Lane

    var body: some View {
        TimelineView(.animation(minimumInterval: 1.0 / 18.0, paused: false)) { timeline in
            VStack(alignment: .leading, spacing: 6) {
                HStack(alignment: .firstTextBaseline, spacing: 8) {
                    Text(self.lane.title)
                        .font(.caption2.weight(.semibold))
                        .foregroundStyle(.secondary)
                        .textCase(.uppercase)
                        .tracking(0.4)
                    Spacer(minLength: 8)
                    Text(self.lane.remainingLabel)
                        .font(.caption.monospacedDigit().weight(.semibold))
                        .foregroundStyle(self.toneColor)
                }

                GeometryReader { geometry in
                    let width = max(geometry.size.width, 1)
                    let height = max(geometry.size.height, 1)
                    let elapsed = self.lane.elapsedFraction ?? 0.0
                    let remaining = self.lane.remainingFraction
                    let point = CGPoint(
                        x: width * elapsed,
                        y: height * (1 - remaining)
                    )
                    let pulse = self.pulseScale(for: timeline.date)

                    ZStack(alignment: .topLeading) {
                        RoundedRectangle(cornerRadius: 8, style: .continuous)
                            .fill(Color.primary.opacity(0.025))

                        Path { path in
                            path.move(to: CGPoint(x: 0, y: 0))
                            path.addLine(to: CGPoint(x: width, y: height))
                        }
                        .stroke(Color.primary.opacity(0.14), style: StrokeStyle(lineWidth: 1, dash: [4, 4]))

                        Path { path in
                            path.move(to: CGPoint(x: 0, y: height))
                            path.addLine(to: CGPoint(x: 0, y: 0))
                            path.addLine(to: point)
                            path.addLine(to: CGPoint(x: width, y: height))
                            path.closeSubpath()
                        }
                        .fill(self.toneColor.opacity(0.08))

                        Path { path in
                            path.move(to: CGPoint(x: 0, y: 0))
                            path.addLine(to: point)
                            path.addLine(to: CGPoint(x: width, y: height))
                        }
                        .stroke(self.toneColor, style: StrokeStyle(lineWidth: 2, lineCap: .round, lineJoin: .round))

                        Path { path in
                            path.move(to: CGPoint(x: point.x, y: 0))
                            path.addLine(to: CGPoint(x: point.x, y: height))
                        }
                        .stroke(self.toneColor.opacity(0.22), style: StrokeStyle(lineWidth: 1, dash: [3, 3]))

                        Circle()
                            .stroke(self.toneColor.opacity(0.18), lineWidth: 8)
                            .frame(width: 14, height: 14)
                            .scaleEffect(pulse)
                            .position(point)

                        Circle()
                            .fill(self.toneColor)
                            .frame(width: 8, height: 8)
                            .position(point)
                    }
                }
                .frame(height: 34)

                HStack(alignment: .firstTextBaseline, spacing: 8) {
                    Text(self.lane.resetDetail)
                        .font(.caption2)
                        .foregroundStyle(Color.primary.opacity(0.68))
                    Spacer(minLength: 8)
                    Text(self.burnLabel)
                        .font(.caption2)
                        .foregroundStyle(Color.primary.opacity(0.54))
                    if let paceLabel = self.lane.paceLabel {
                        Text("· \(paceLabel.lowercased())")
                            .font(.caption2)
                            .foregroundStyle(Color.primary.opacity(0.54))
                    }
                }
            }
        }
    }

    private var toneColor: Color {
        switch self.lane.remainingPercent {
        case ..<15:
            return Color.accentError.opacity(0.78)
        case ..<35:
            return Color.warning.opacity(0.74)
        default:
            return Color.primary.opacity(0.78)
        }
    }

    private func pulseScale(for date: Date) -> CGFloat {
        let phase = (sin(date.timeIntervalSinceReferenceDate * 3.4) + 1) / 2
        return 0.88 + CGFloat(phase) * 0.42
    }

    private var burnLabel: String {
        guard let elapsed = self.lane.elapsedFraction else { return "Time window unknown" }
        return "\(Int((elapsed * 100).rounded()))% of window elapsed"
    }
}

private struct WindowOverviewProviderCard: View {
    @Bindable var model: ProviderFeatureModel
    let item: ProviderMenuProjection
    let openProvider: () -> Void

    private var metric: WindowProviderMetricSummary {
        WindowProviderMetricSummary.make(
            item: self.item,
            showUsedValues: self.model.showUsedValues
        )
    }

    private var note: WindowOverviewProviderNote? {
        WindowOverviewProviderNote.make(item: self.item)
    }

    private var costInsights: WindowOverviewProviderCostInsightsModel {
        WindowOverviewProviderCostInsightsModel.make(item: self.item)
    }

    private var quotaWindows: WindowOverviewQuotaWindowsModel {
        WindowOverviewQuotaWindowsModel.make(item: self.item)
    }

    private var quotaSuggestions: WindowQuotaSuggestionsModel? {
        WindowQuotaSuggestionsModel.make(suggestions: self.item.quotaSuggestions)
    }

    private var depletionForecast: WindowDepletionForecastModel? {
        WindowDepletionForecastModel.make(forecast: self.item.depletionForecast)
    }

    private var predictiveInsights: PredictiveInsightsSummaryModel? {
        PredictiveInsightsSummaryModel.make(insights: self.item.predictiveInsights)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack(alignment: .top, spacing: 12) {
                VStack(alignment: .leading, spacing: 8) {
                    HStack(alignment: .firstTextBaseline, spacing: 8) {
                        Text(self.item.title)
                            .font(.title3.weight(.semibold))
                        StateBadge(state: self.item.visualState, label: self.item.stateLabel)
                    }

                    Text(self.item.costLabel)
                        .font(.callout)
                        .foregroundStyle(.secondary)

                    Text(self.item.lastRefreshLabel)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }

                Spacer(minLength: 12)
                Button("Open", action: self.openProvider)
                    .buttonStyle(.bordered)
                    .controlSize(.small)
            }

            VStack(alignment: .leading, spacing: 8) {
                if self.quotaWindows.lanes.isEmpty {
                    Text(self.metric.title)
                        .font(.footnote.weight(.semibold))
                        .foregroundStyle(Color.primary.opacity(0.72))
                    Text(self.metric.value)
                        .font(.callout.weight(.semibold))
                    Text(self.metric.detail)
                        .font(.caption)
                        .foregroundStyle(Color.primary.opacity(0.68))
                        .fixedSize(horizontal: false, vertical: true)
                }

                if !self.quotaWindows.lanes.isEmpty {
                    WindowOverviewQuotaWindowsStrip(model: self.quotaWindows)
                    WindowOverviewQuotaBurnChart(model: self.quotaWindows)
                }

                if let depletionForecast = self.depletionForecast {
                    WindowOverviewDepletionForecastStrip(model: depletionForecast)
                }

                if let quotaSuggestions = self.quotaSuggestions {
                    WindowOverviewQuotaSuggestionStrip(model: quotaSuggestions)
                }

                if let predictiveInsights = self.predictiveInsights {
                    WindowOverviewPredictiveInsightsStrip(model: predictiveInsights)
                }

                if !self.costInsights.stats.isEmpty || self.costInsights.mixLabel != nil {
                    WindowOverviewProviderCostInsights(model: self.costInsights)
                }

                if let note {
                    WindowCardNote(
                        text: note.text,
                        tone: note.tone
                    )
                }
            }
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .leading)
        .menuCardBackground(opacity: 0.04, cornerRadius: 16)
        .overlay(
            RoundedRectangle(cornerRadius: 16, style: .continuous)
                .stroke(self.borderColor, lineWidth: self.borderLineWidth)
        )
    }

    private var borderColor: Color {
        switch self.item.visualState {
        case .error:
            return Color.accentError.opacity(0.42)
        case .incident:
            return Color.accentError.opacity(0.34)
        case .degraded, .stale:
            return Color.warning.opacity(0.38)
        case .refreshing:
            return Color.primary.opacity(0.18)
        case .healthy:
            return Color.primary.opacity(0.12)
        }
    }

    private var borderLineWidth: CGFloat {
        switch self.item.visualState {
        case .healthy:
            return 1
        default:
            return 1.5
        }
    }
}

private struct WindowOverviewQuotaSuggestionStrip: View {
    let model: WindowQuotaSuggestionsModel

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Text("Suggested quotas")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(Color.primary.opacity(0.72))
                    .textCase(.uppercase)
                    .tracking(0.4)
                Spacer(minLength: 8)
                Text("\(self.model.sampleCount) blocks")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }

            LazyVGrid(columns: [GridItem(.flexible(minimum: 90), spacing: 8)], spacing: 8) {
                ForEach(self.model.items) { item in
                    VStack(alignment: .leading, spacing: 4) {
                        HStack(alignment: .firstTextBaseline, spacing: 6) {
                            Text(item.label)
                                .font(.caption2.weight(.semibold))
                                .foregroundStyle(.secondary)
                            if item.isRecommended {
                                Text("REC")
                                    .font(.caption2.weight(.semibold))
                                    .foregroundStyle(Color.primary.opacity(0.72))
                            }
                        }

                        Text(item.value)
                            .font(.callout.monospacedDigit().weight(.semibold))
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(.horizontal, 10)
                    .padding(.vertical, 8)
                    .menuCardBackground(opacity: 0.03, cornerRadius: 10)
                }
            }

            if let note = self.model.note {
                Text(note)
                    .font(.caption2)
                    .foregroundStyle(Color.primary.opacity(0.66))
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
    }
}

private struct WindowOverviewPredictiveInsightsStrip: View {
    let model: PredictiveInsightsSummaryModel

    private let columns = [
        GridItem(.flexible(minimum: 120), spacing: 8),
        GridItem(.flexible(minimum: 120), spacing: 8),
    ]

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Text("Predictive insights")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(Color.primary.opacity(0.72))
                    .textCase(.uppercase)
                    .tracking(0.4)
                Spacer(minLength: 8)
                if let analysis = self.model.limitHitAnalysis {
                    Text(analysis.riskLevel)
                        .font(.caption2.weight(.semibold))
                        .textCase(.uppercase)
                        .foregroundStyle(self.riskColor(for: analysis.riskLevel))
                }
            }

            LazyVGrid(columns: self.columns, spacing: 8) {
                if let burn = self.model.rollingHourBurn {
                    PredictiveInsightsMetricCard(
                        title: "Rolling Hour",
                        value: burn.tokensPerMinuteLabel,
                        secondaryValue: burn.costPerHourLabel,
                        detail: [burn.coverageLabel, burn.tierLabel].compactMap { $0 }.joined(separator: " · "),
                        accent: .primary
                    )
                }

                if let envelope = self.model.historicalEnvelope {
                    PredictiveInsightsMetricCard(
                        title: "Envelope",
                        value: envelope.tokensRangeLabel,
                        secondaryValue: envelope.costRangeLabel,
                        detail: "\(envelope.sampleLabel) · \(envelope.turnsRangeLabel)",
                        accent: .primary
                    )
                }

                if let analysis = self.model.limitHitAnalysis {
                    PredictiveInsightsMetricCard(
                        title: "Limit Hits",
                        value: analysis.hitRateLabel,
                        secondaryValue: analysis.thresholdLabel,
                        detail: [analysis.summaryLabel, analysis.activityLabel].compactMap { $0 }.joined(separator: " · "),
                        accent: self.riskColor(for: analysis.riskLevel)
                    )
                }
            }
        }
    }

    private func riskColor(for riskLevel: String) -> Color {
        switch riskLevel.lowercased() {
        case "critical", "high":
            return .accentError
        case "warn", "warning", "medium", "moderate":
            return .warning
        default:
            return Color.primary.opacity(0.8)
        }
    }
}

private struct WindowPredictiveInsightsCardBody: View {
    let model: PredictiveInsightsSummaryModel
    let density: LiveMonitorDensity

    private var spacing: CGFloat {
        self.density == .compact ? 8 : 10
    }

    var body: some View {
        VStack(alignment: .leading, spacing: self.spacing) {
            if let burn = self.model.rollingHourBurn {
                PredictiveInsightsDetailSection(
                    title: "Rolling hour burn",
                    value: burn.tokensPerMinuteLabel,
                    secondaryValue: burn.costPerHourLabel,
                    detail: [burn.coverageLabel, burn.tierLabel].compactMap { $0 }.joined(separator: " · "),
                    density: self.density
                )
            }

            if let envelope = self.model.historicalEnvelope {
                PredictiveInsightsDetailSection(
                    title: "Historical envelope",
                    value: envelope.tokensRangeLabel,
                    secondaryValue: envelope.costRangeLabel,
                    detail: "\(envelope.sampleLabel) · \(envelope.turnsRangeLabel)\n\(envelope.averagesLabel)",
                    density: self.density
                )
            }

            if let analysis = self.model.limitHitAnalysis {
                PredictiveInsightsDetailSection(
                    title: "Limit hit analysis",
                    value: analysis.hitRateLabel,
                    secondaryValue: analysis.thresholdLabel,
                    detail: [analysis.summaryLabel, analysis.activityLabel].compactMap { $0 }.joined(separator: " · "),
                    density: self.density,
                    accent: self.riskColor(for: analysis.riskLevel)
                )
            }
        }
    }

    private func riskColor(for riskLevel: String) -> Color {
        switch riskLevel.lowercased() {
        case "critical", "high":
            return .accentError
        case "warn", "warning", "medium", "moderate":
            return .warning
        default:
            return Color.primary.opacity(0.8)
        }
    }
}

private struct PredictiveInsightsMetricCard: View {
    let title: String
    let value: String
    let secondaryValue: String?
    let detail: String
    let accent: Color

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(self.title)
                .font(.caption2.weight(.semibold))
                .foregroundStyle(.secondary)
                .textCase(.uppercase)
                .tracking(0.4)

            Text(self.value)
                .font(.callout.monospacedDigit().weight(.semibold))

            if let secondaryValue, !secondaryValue.isEmpty {
                Text(secondaryValue)
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(self.accent.opacity(0.9))
            }

            Text(self.detail)
                .font(.caption2)
                .foregroundStyle(Color.primary.opacity(0.68))
                .fixedSize(horizontal: false, vertical: true)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, 10)
        .padding(.vertical, 8)
        .menuCardBackground(opacity: 0.03, cornerRadius: 10)
    }
}

private struct PredictiveInsightsDetailSection: View {
    let title: String
    let value: String
    let secondaryValue: String?
    let detail: String
    let density: LiveMonitorDensity
    var accent: Color = Color.primary.opacity(0.82)

    var body: some View {
        VStack(alignment: .leading, spacing: self.density == .compact ? 5 : 6) {
            Text(self.title)
                .font((self.density == .compact ? Font.caption2 : .caption2).weight(.semibold))
                .foregroundStyle(.secondary)
                .textCase(.uppercase)
                .tracking(0.4)

            Text(self.value)
                .font((self.density == .compact ? Font.callout : .headline).monospacedDigit().weight(.semibold))

            if let secondaryValue, !secondaryValue.isEmpty {
                Text(secondaryValue)
                    .font(self.density == .compact ? .caption2.weight(.semibold) : .caption.weight(.semibold))
                    .foregroundStyle(self.accent)
            }

            Text(self.detail)
                .font(self.density == .compact ? .caption2 : .caption)
                .foregroundStyle(Color.primary.opacity(0.68))
                .fixedSize(horizontal: false, vertical: true)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, self.density == .compact ? 10 : 12)
        .padding(.vertical, self.density == .compact ? 8 : 10)
        .background(
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .fill(Color.primary.opacity(0.035))
        )
    }
}

private struct WindowOverviewTotalsCard: View {
    let projection: OverviewMenuProjection

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack(alignment: .firstTextBaseline, spacing: 12) {
                Text("Today")
                    .font(.headline)
                Spacer(minLength: 12)
                Text(self.combinedValue)
                    .font(.system(size: 30, weight: .bold, design: .rounded).monospacedDigit())
            }

            if let topProvider = self.topProvider {
                HStack(alignment: .firstTextBaseline, spacing: 8) {
                    Text("Highest spend: \(topProvider.title)")
                        .font(.callout.weight(.medium))
                    StateBadge(state: topProvider.visualState, label: topProvider.stateLabel)
                }
            } else {
                Text("Waiting for provider activity")
                    .font(.callout)
                    .foregroundStyle(.secondary)
            }

            Text(self.projection.refreshedAtLabel)
                .font(.caption)
                .foregroundStyle(.secondary)

            Text(self.projection.activitySummaryLabel)
                .font(.callout)
                .foregroundStyle(.secondary)
                .fixedSize(horizontal: false, vertical: true)

        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .leading)
        .menuCardBackground(opacity: 0.04, cornerRadius: 16)
    }

    private var combinedValue: String {
        Self.currencyLabel(self.projection.combinedTodayCostUSD)
    }

    private var topProvider: ProviderMenuProjection? {
        guard self.projection.combinedTodayCostUSD > 0 else {
            return nil
        }
        return self.projection.items.max { lhs, rhs in
            Self.todayCost(lhs) < Self.todayCost(rhs)
        }
    }

    private static func todayCost(_ item: ProviderMenuProjection) -> Double {
        item.todayCostUSD ?? 0
    }

    private static func currencyLabel(_ value: Double) -> String {
        FormatHelpers.formatUSD(value)
    }
}

private struct WindowProviderView: View {
    @Bindable var model: ProviderFeatureModel

    var body: some View {
        VStack(alignment: .leading, spacing: 18) {
            WindowHeader(
                title: self.model.provider.title,
                subtitle: WindowProviderHeaderSubtitle.make(
                    projection: self.model.projection,
                    issue: self.model.issue
                ),
                issue: WindowHeaderIssuePresentation.make(
                    issue: self.model.issue,
                    fallbackMessage: self.model.projection.globalIssueLabel
                ),
                onRetry: {
                    Task { await self.model.refresh() }
                },
                isRetrying: self.model.isBusy
            )

            ProviderMenuCard(providerModel: self.model)

            if let forecast = WindowDepletionForecastModel.make(forecast: self.model.projection.depletionForecast) {
                WindowOverviewDepletionForecastStrip(model: forecast)
            }

            ProviderSessionDetails(model: self.model)
        }
    }
}

enum WindowProviderHeaderSubtitle {
    static func make(
        projection: ProviderMenuProjection,
        issue: AppIssue?
    ) -> String {
        let status = issue?.kind == .helperStartup ? "Waiting for local server" : projection.refreshStatusLabel
        guard let projected = projection.weeklyProjectedCostUSD, projected > 0 else {
            return status
        }
        return "\(status) · Projected this week: \(self.currencyLabel(projected))"
    }

    private static func currencyLabel(_ value: Double) -> String {
        let formatter = NumberFormatter()
        formatter.numberStyle = .currency
        formatter.locale = Locale(identifier: "en_US")
        formatter.currencyCode = "USD"
        formatter.currencySymbol = "$"
        formatter.minimumFractionDigits = 2
        formatter.maximumFractionDigits = 2
        formatter.positiveFormat = "¤#,##0.00"
        formatter.negativeFormat = "-¤#,##0.00"
        return formatter.string(from: NSNumber(value: value)) ?? String(format: "$%.2f", value)
    }
}

private struct WindowHeader<Trailing: View>: View {
    let title: String
    let subtitle: String
    let issue: WindowHeaderIssuePresentation?
    var onRetry: (() -> Void)? = nil
    var isRetrying: Bool = false
    let trailing: Trailing

    init(
        title: String,
        subtitle: String,
        issue: WindowHeaderIssuePresentation? = nil,
        onRetry: (() -> Void)? = nil,
        isRetrying: Bool = false,
        @ViewBuilder trailing: () -> Trailing
    ) {
        self.title = title
        self.subtitle = subtitle
        self.issue = issue
        self.onRetry = onRetry
        self.isRetrying = isRetrying
        self.trailing = trailing()
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(alignment: .firstTextBaseline) {
                Text(self.title)
                    .font(.system(size: 24, weight: .semibold))
                Spacer(minLength: 12)
                self.trailing
            }
            Text(self.subtitle)
                .font(.callout)
                .foregroundStyle(.secondary)
            if let issue {
                WindowIssueBanner(
                    issue: issue,
                    onRetry: onRetry,
                    isRetrying: self.isRetrying
                )
            }
        }
    }
}

extension WindowHeader where Trailing == EmptyView {
    init(
        title: String,
        subtitle: String,
        issue: WindowHeaderIssuePresentation? = nil,
        onRetry: (() -> Void)? = nil,
        isRetrying: Bool = false
    ) {
        self.init(
            title: title,
            subtitle: subtitle,
            issue: issue,
            onRetry: onRetry,
            isRetrying: isRetrying,
            trailing: { EmptyView() }
        )
    }
}

enum WindowHeaderIssueTone: Equatable {
    case pending
    case warning
    case critical

    var tint: Color {
        switch self {
        case .pending:
            return .accentInteractive
        case .warning:
            return .warning
        case .critical:
            return .accentError
        }
    }

    var background: Color {
        switch self {
        case .pending:
            return Color.accentInteractive.opacity(0.08)
        case .warning:
            return Color.warning.opacity(0.10)
        case .critical:
            return Color.accentError.opacity(0.10)
        }
    }

    var border: Color {
        switch self {
        case .pending:
            return Color.accentInteractive.opacity(0.22)
        case .warning:
            return Color.warning.opacity(0.28)
        case .critical:
            return Color.accentError.opacity(0.28)
        }
    }
}

struct WindowHeaderIssuePresentation: Equatable {
    let tone: WindowHeaderIssueTone
    let symbolName: String
    let badge: String
    let title: String
    let detail: String?
    let actionTitle: String
    let progressTitle: String

    static func make(
        issue: AppIssue?,
        fallbackMessage: String? = nil
    ) -> Self? {
        if let issue {
            switch issue.kind {
            case .helperStartup:
                return Self(
                    tone: .pending,
                    symbolName: "clock.fill",
                    badge: "Local server",
                    title: "Starting local server",
                    detail: "The embedded Heimdall helper is booting. Live data will appear automatically once it responds.",
                    actionTitle: "Check again",
                    progressTitle: "Checking…"
                )
            case .authRecovery:
                return Self(
                    tone: .warning,
                    symbolName: "person.crop.circle.badge.exclamationmark",
                    badge: "Authentication",
                    title: "Action required",
                    detail: issue.message,
                    actionTitle: "Retry",
                    progressTitle: "Retrying…"
                )
            default:
                return Self.make(message: issue.message)
            }
        }

        return Self.make(message: fallbackMessage)
    }

    static func make(message: String?) -> Self? {
        guard let message else { return nil }
        let normalized = message.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !normalized.isEmpty else { return nil }

        let lowercased = normalized.lowercased()
        if lowercased.contains("still starting") {
            return Self(
                tone: .pending,
                symbolName: "clock.fill",
                badge: "Local server",
                title: "Starting local server",
                detail: "The embedded Heimdall helper is booting. Live data will appear automatically once it responds.",
                actionTitle: "Check again",
                progressTitle: "Checking…"
            )
        }
        if lowercased.contains("cannot reach the local heimdall server")
            || lowercased.contains("failed to connect")
            || lowercased.contains("connection refused")
        {
            return Self(
                tone: .warning,
                symbolName: "bolt.horizontal.circle.fill",
                badge: "Local server",
                title: "Can’t reach Heimdall",
                detail: "Retry now, or wait for the next automatic refresh after the helper becomes reachable.",
                actionTitle: "Retry",
                progressTitle: "Retrying…"
            )
        }
        if lowercased.contains("did not respond in time") || lowercased.contains("timed out") {
            return Self(
                tone: .warning,
                symbolName: "hourglass.circle.fill",
                badge: "Local server",
                title: "Heimdall is taking too long to respond",
                detail: "The helper is running but this refresh timed out. Retry now or wait for the next poll.",
                actionTitle: "Retry",
                progressTitle: "Retrying…"
            )
        }

        return Self(
            tone: .warning,
            symbolName: "exclamationmark.triangle.fill",
            badge: "Refresh issue",
            title: normalized,
            detail: nil,
            actionTitle: "Retry",
            progressTitle: "Retrying…"
        )
    }
}

struct WindowIssueBanner: View {
    let issue: WindowHeaderIssuePresentation
    var onRetry: (() -> Void)? = nil
    var isRetrying: Bool = false

    var body: some View {
        HStack(alignment: .center, spacing: 12) {
            ZStack {
                Circle()
                    .fill(issue.tone.tint.opacity(0.14))
                Image(systemName: issue.symbolName)
                    .font(.system(size: 13, weight: .semibold))
                    .foregroundStyle(issue.tone.tint)
            }
            .frame(width: 30, height: 30)

            VStack(alignment: .leading, spacing: 6) {
                Text(issue.badge)
                    .font(.caption2.weight(.semibold))
                    .foregroundStyle(issue.tone.tint)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(
                        Capsule()
                            .fill(issue.tone.tint.opacity(0.10))
                    )

                VStack(alignment: .leading, spacing: 2) {
                    Text(issue.title)
                        .font(.callout.weight(.semibold))
                        .foregroundStyle(.primary)
                        .fixedSize(horizontal: false, vertical: true)
                    if let detail = issue.detail {
                        Text(detail)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                            .fixedSize(horizontal: false, vertical: true)
                    }
                }
            }

            Spacer(minLength: 12)

            if let onRetry {
                Button(action: onRetry) {
                    HStack(spacing: 6) {
                        if self.isRetrying {
                            ProgressView()
                                .controlSize(.small)
                        } else {
                            Image(systemName: "arrow.clockwise")
                                .font(.caption.weight(.semibold))
                        }
                        Text(self.isRetrying ? issue.progressTitle : issue.actionTitle)
                    }
                }
                .buttonStyle(SecondaryDashboardButtonStyle())
                .disabled(self.isRetrying)
            }
        }
        .padding(12)
        .background(
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .fill(issue.tone.background)
        )
        .overlay(
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .stroke(issue.tone.border, lineWidth: 1)
        )
    }
}

enum WindowCardTone: Equatable {
    case neutral
    case warning
    case critical
}

struct WindowOverviewProviderNote: Equatable {
    let text: String
    let tone: WindowCardTone

    static func make(item: ProviderMenuProjection) -> Self? {
        if let incidentLabel = item.incidentLabel {
            return Self(text: incidentLabel, tone: .critical)
        }
        if let warning = item.warningLabels.first {
            return Self(text: warning, tone: .warning)
        }
        if let authHeadline = item.authHeadline {
            return Self(text: authHeadline, tone: .neutral)
        }
        return nil
    }
}

private struct WindowCardNote: View {

    let text: String
    let tone: WindowCardTone

    var body: some View {
        HStack(alignment: .firstTextBaseline, spacing: 6) {
            Image(systemName: self.iconName)
                .font(.caption.weight(.semibold))
            Text(self.text)
                .font(.caption)
                .fixedSize(horizontal: false, vertical: true)
        }
        .padding(.horizontal, 8)
        .padding(.vertical, 6)
        .foregroundStyle(self.color)
        .background(
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .fill(self.backgroundColor)
        )
        .overlay(
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .stroke(self.borderColor, lineWidth: 1)
        )
    }

    private var iconName: String {
        switch self.tone {
        case .neutral:
            return "info.circle.fill"
        case .warning:
            return "exclamationmark.triangle.fill"
        case .critical:
            return "exclamationmark.octagon.fill"
        }
    }

    private var color: Color {
        switch self.tone {
        case .neutral:
            return Color.primary.opacity(0.78)
        case .warning:
            return .warning
        case .critical:
            return .accentError
        }
    }

    private var backgroundColor: Color {
        switch self.tone {
        case .neutral:
            return Color.primary.opacity(0.06)
        case .warning:
            return Color.warning.opacity(0.12)
        case .critical:
            return Color.accentError.opacity(0.12)
        }
    }

    private var borderColor: Color {
        switch self.tone {
        case .neutral:
            return Color.primary.opacity(0.14)
        case .warning:
            return Color.warning.opacity(0.24)
        case .critical:
            return Color.accentError.opacity(0.24)
        }
    }
}

private struct ProviderSessionDetails: View {
    @Bindable var model: ProviderFeatureModel

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Web Session")
                .font(.headline)
            if let session = self.model.importedSession {
                Text(session.expired ? "Stored session is expired." : (session.loginRequired ? "Stored session is missing an active auth cookie." : "Stored session is ready."))
                Text("Source: \(session.browserSource.title) · \(session.profileName)")
                    .foregroundStyle(.secondary)
                Button("Reset Session") {
                    Task { await self.model.resetBrowserSession() }
                }
                .disabled(self.model.isImportingSession)
            } else {
                Text("No imported browser session stored.")
                    .foregroundStyle(.secondary)
            }

            ForEach(self.model.importCandidates) { candidate in
                Button("Import from \(candidate.title)") {
                    Task { await self.model.importBrowserSession(candidate: candidate) }
                }
                .disabled(self.model.isImportingSession)
            }
        }
        .padding(14)
        .background(
            RoundedRectangle(cornerRadius: 14)
                .fill(Color.primary.opacity(0.05))
        )
        .overlay(
            RoundedRectangle(cornerRadius: 14)
                .stroke(Color.primary.opacity(0.12), lineWidth: 1)
        )
    }
}
