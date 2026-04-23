import Charts
import HeimdallDomain
import Observation
import SwiftUI

struct HeimdallMobileRootView: View {
    @Environment(\.scenePhase) private var scenePhase
    @State var model: MobileDashboardModel

    var body: some View {
        NavigationStack {
            Group {
                if self.model.isLoading && !self.model.hasSnapshot {
                    ProgressView("Loading synced snapshot…")
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else if self.model.hasSnapshot {
                    VStack(spacing: 0) {
                        if let warning = self.model.staleSnapshotWarning {
                            SyncWarningBanner(message: warning)
                        }

                        TabView {
                            OverviewTab(model: self.model)
                                .tabItem { Label("Overview", systemImage: "rectangle.grid.2x2") }
                            HistoryTab(model: self.model)
                                .tabItem { Label("History", systemImage: "chart.line.uptrend.xyaxis") }
                            FreshnessTab(model: self.model)
                                .tabItem { Label("Freshness", systemImage: "clock") }
                        }
                    }
                } else if let lastError = self.model.lastError {
                    ContentUnavailableView(
                        "Sync Unavailable",
                        systemImage: "icloud.slash",
                        description: Text(lastError)
                    )
                } else {
                    ContentUnavailableView(
                        "No Synced Data Yet",
                        systemImage: "iphone.slash",
                        description: Text("Open HeimdallBar on macOS and refresh once to publish a mobile snapshot.")
                    )
                }
            }
            .navigationTitle("Heimdall")
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("Reload") {
                        Task { await self.model.refresh(reason: .manual) }
                    }
                }
            }
        }
        .sheet(
            isPresented: Binding(
                get: { self.model.isAliasEditorPresented },
                set: { isPresented in
                    if isPresented {
                        self.model.presentAliasEditor()
                    } else {
                        self.model.dismissAliasEditor()
                    }
                }
            )
        ) {
            AliasEditorSheet(model: self.model)
        }
        .task {
            await self.model.refresh(reason: .startup)
        }
        .onOpenURL { url in
            Task { await self.model.acceptShareURL(url) }
        }
        .onChange(of: self.scenePhase) { _, newPhase in
            guard newPhase == .active else { return }
            Task { await self.model.refresh(reason: .foreground) }
        }
    }
}

private struct SyncWarningBanner: View {
    let message: String

    var body: some View {
        Label {
            Text("Showing last synced snapshot. \(self.message)")
        } icon: {
            Image(systemName: "exclamationmark.triangle.fill")
        }
        .font(.footnote)
        .foregroundStyle(.primary)
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal)
        .padding(.vertical, 10)
        .background(Color.orange.opacity(0.16))
    }
}

private struct OverviewTab: View {
    @Bindable var model: MobileDashboardModel

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                if let aggregate = self.model.scopedAggregate {
                    ScopeControlsCard(model: self.model)

                    if self.model.selectedAccountScope == .all && !self.model.accountSummaries.isEmpty {
                        CollapsibleCardSection(
                            title: "Accounts",
                            sectionID: "overview.accounts",
                            model: self.model
                        ) {
                            VStack(alignment: .leading, spacing: 12) {
                                ForEach(self.model.accountSummaries) { summary in
                                    AccountSummaryRow(summary: summary)
                                }
                                if let rolledUp = self.model.rolledUpAccountSummary {
                                    AccountSummaryRow(summary: rolledUp)
                                }
                            }
                        }
                    }

                    VStack(alignment: .leading, spacing: 8) {
                        Text("All Installations")
                            .font(.headline)
                        LabeledContent("Account scope", value: self.model.selectedScopeTitle)
                        LabeledContent("Today tokens", value: "\(aggregate.aggregateTotals.todayTokens)")
                        LabeledContent("Today cost", value: usd(aggregate.aggregateTotals.todayCostUSD))
                        LabeledContent("Last 90 days", value: "\(aggregate.aggregateTotals.last90DaysTokens) tokens")
                        LabeledContent("90 day cost", value: usd(aggregate.aggregateTotals.last90DaysCostUSD))
                        LabeledContent("Installations", value: "\(aggregate.installations.count)")
                    }
                    .cardStyle(compact: self.model.compressionPreference == .compact)

                    if self.model.visibleProviders.count > 1 {
                        ProviderPicker(model: self.model)
                    }

                    if let provider = self.model.selectedProviderSnapshot {
                        ProviderSummaryCard(
                            provider: provider,
                            scopeTitle: self.model.selectedScopeTitle,
                            currentLimitInstallationIDs: aggregate.aggregateProviderViews
                                .first(where: { $0.providerID == self.model.selectedProvider })?
                                .currentLimitInstallationIDs ?? [],
                            compact: self.model.compressionPreference == .compact
                        )
                    }

                    CollapsibleCardSection(
                        title: "Installations",
                        sectionID: "overview.installations",
                        model: self.model
                    ) {
                        VStack(alignment: .leading, spacing: 12) {
                            ForEach(self.model.visibleInstallations) { installation in
                                InstallationCard(
                                    installation: installation,
                                    compact: self.model.compressionPreference == .compact
                                )
                            }
                            if !self.model.rolledUpInstallations.isEmpty {
                                OtherInstallationsCard(installations: self.model.rolledUpInstallations)
                            }
                        }
                    }
                } else {
                    ScopedEmptyStateCard(model: self.model)
                }
            }
            .padding()
        }
        .refreshable {
            await self.model.refresh(reason: .manual)
        }
    }
}

private struct HistoryTab: View {
    @Bindable var model: MobileDashboardModel

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                if let aggregate = self.model.scopedAggregate {
                    ScopeControlsCard(model: self.model)

                    if self.model.visibleProviders.count > 1 {
                        VStack(alignment: .leading, spacing: 12) {
                            Text("Provider")
                                .font(.headline)
                            ProviderPicker(model: self.model)
                        }
                        .cardStyle(compact: self.model.compressionPreference == .compact)
                    }

                    if let provider = self.model.selectedProviderSnapshot {
                        MobileProviderChartsSection(
                            provider: provider,
                            compact: self.model.compressionPreference == .compact
                        )
                    }

                    if let history = self.model.selectedHistorySeries {
                        VStack(alignment: .leading, spacing: 8) {
                            Text("Recent trend")
                                .font(.headline)
                            LabeledContent("Scope", value: self.model.selectedScopeTitle)
                            LabeledContent("90 day tokens", value: "\(history.totalTokens)")
                            LabeledContent("90 day cost", value: usd(history.totalCostUSD))
                        }
                        .cardStyle(compact: self.model.compressionPreference == .compact)

                        CollapsibleCardSection(
                            title: "Latest days",
                            sectionID: "history.days",
                            model: self.model
                        ) {
                            let showCompact = self.model.compressionPreference == .compact && self.model.isSectionCollapsed("history.day-window")
                            let points = showCompact ? Array(history.daily.suffix(7)) : history.daily
                            VStack(alignment: .leading, spacing: 12) {
                                if self.model.compressionPreference == .compact && history.daily.count > 7 {
                                    Text(showCompact ? "Showing last 7 days." : "Showing all synced days.")
                                        .font(.footnote)
                                        .foregroundStyle(.secondary)
                                }
                                ForEach(Array(points.reversed())) { point in
                                    VStack(alignment: .leading, spacing: 4) {
                                        Text(point.day)
                                            .font(.headline)
                                        Text("\(point.totalTokens) tokens · \(usd(point.costUSD))")
                                            .foregroundStyle(.secondary)
                                    }
                                }
                                if self.model.compressionPreference == .compact && history.daily.count > 7 {
                                    Button(showCompact ? "Show More" : "Show Less") {
                                        self.model.toggleSectionCollapsed("history.day-window")
                                    }
                                }
                            }
                        }
                    } else if aggregate.aggregateProviderViews.isEmpty {
                        ScopedMessageCard(message: self.model.scopedEmptyStateMessage)
                    } else {
                        ScopedMessageCard(message: "No provider history is available yet.")
                    }
                } else {
                    ScopedEmptyStateCard(model: self.model)
                }
            }
            .padding()
        }
        .refreshable {
            await self.model.refresh(reason: .manual)
        }
    }
}

private struct FreshnessTab: View {
    @Bindable var model: MobileDashboardModel

    var body: some View {
        List {
            Section {
                ScopeControlsInline(model: self.model)
            }

            Section("Cloud Sync") {
                LabeledContent("Status", value: self.model.cloudSyncStatusTitle)
                Text(self.model.cloudSyncStatusDetail)
                    .foregroundStyle(.secondary)
                LabeledContent(
                    "Last successful refresh",
                    value: self.model.lastSuccessfulRefreshAt.map(displayTimestamp) ?? "Never"
                )
                LabeledContent(
                    "Newest installation publish",
                    value: self.model.newestPublishedAt.map(displayTimestamp) ?? "Unavailable"
                )
                if let lastRefreshError = self.model.lastRefreshError {
                    Text("Last refresh error: \(lastRefreshError)")
                        .foregroundStyle(.orange)
                }
            }

            if let aggregate = self.model.scopedAggregate {
                Section("Snapshot") {
                    LabeledContent("Scope", value: self.model.selectedScopeTitle)
                    LabeledContent("Generated", value: displayTimestamp(aggregate.generatedAt))
                    LabeledContent("Installations", value: "\(aggregate.installations.count)")
                    LabeledContent(
                        "Stale installations",
                        value: !aggregate.staleInstallations.isEmpty
                            ? aggregate.staleInstallations.joined(separator: ", ")
                            : "None"
                    )
                    if let selectedProviderView = aggregate.aggregateProviderViews.first(where: { $0.providerID == self.model.selectedProvider }),
                       !selectedProviderView.currentLimitInstallationIDs.isEmpty {
                        LabeledContent(
                            "Live current-limit data",
                            value: selectedProviderView.currentLimitInstallationIDs.joined(separator: ", ")
                        )
                    }
                }

                Section {
                    if self.model.isSectionCollapsed("freshness.installations") {
                        ForEach(self.model.visibleInstallations) { installation in
                            InstallationFreshnessRow(installation: installation)
                        }
                        if !self.model.rolledUpInstallations.isEmpty {
                            Text("Other installations: \(self.model.rolledUpInstallations.count)")
                                .foregroundStyle(.secondary)
                        }
                    } else {
                        ForEach(aggregate.installations) { installation in
                            InstallationFreshnessRow(installation: installation)
                        }
                    }
                    if self.model.compressionPreference == .compact && aggregate.installations.count > 3 {
                        Button(self.model.isSectionCollapsed("freshness.installations") ? "Show More" : "Show Less") {
                            self.model.toggleSectionCollapsed("freshness.installations")
                        }
                    }
                } header: {
                    Text("Installation refreshes")
                }
            } else {
                Section {
                    Text(self.model.scopedEmptyStateMessage)
                        .foregroundStyle(.secondary)
                }
            }
        }
        .refreshable {
            await self.model.refresh(reason: .manual)
        }
    }
}

private struct ScopeControlsCard: View {
    @Bindable var model: MobileDashboardModel

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            ScopeControlsInline(model: self.model)
            Text(self.model.selectedScopeDescription)
                .font(.footnote)
                .foregroundStyle(.secondary)
        }
        .cardStyle(compact: self.model.compressionPreference == .compact)
    }
}

private struct ScopeControlsInline: View {
    @Bindable var model: MobileDashboardModel

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Menu {
                Button("All Accounts") {
                    self.model.selectAccountScope(.all)
                }
                if !self.model.aliasAccountOptions.isEmpty {
                    Section("Aliases") {
                        ForEach(self.model.aliasAccountOptions) { option in
                            Button(option.displayTitle) {
                                self.model.selectAccountScope(option.scope)
                            }
                        }
                    }
                }
                if !self.model.rawAccountOptions.isEmpty {
                    Section("Accounts") {
                        ForEach(self.model.rawAccountOptions) { option in
                            Button(option.displayTitle) {
                                self.model.selectAccountScope(option.scope)
                            }
                        }
                    }
                }
                Divider()
                Button("Manage Aliases") {
                    self.model.presentAliasEditor()
                }
            } label: {
                ScopeMenuLabel(
                    title: "Account",
                    value: self.model.selectedScopeTitle,
                    systemImage: "person.crop.circle"
                )
            }

            Menu {
                ForEach(MobileCompressionPreference.allCases) { preference in
                    Button(preference.title) {
                        self.model.setCompressionPreference(preference)
                    }
                }
            } label: {
                ScopeMenuLabel(
                    title: "Display",
                    value: self.model.compressionPreference.title,
                    systemImage: "rectangle.compress.vertical"
                )
            }
        }
    }
}

private struct ScopeMenuLabel: View {
    let title: String
    let value: String
    let systemImage: String

    var body: some View {
        HStack {
            Label(title, systemImage: self.systemImage)
            Spacer()
            Text(self.value)
                .foregroundStyle(.secondary)
            Image(systemName: "chevron.up.chevron.down")
                .font(.footnote)
                .foregroundStyle(.secondary)
        }
        .font(.subheadline)
    }
}

private struct ProviderPicker: View {
    @Bindable var model: MobileDashboardModel

    var body: some View {
        Picker(
            "Provider",
            selection: Binding(
                get: { self.model.selectedProvider },
                set: { self.model.selectProvider($0) }
            )
        ) {
            ForEach(self.model.visibleProviders) { provider in
                Text(provider.title).tag(provider)
            }
        }
        .pickerStyle(.segmented)
    }
}

private struct ProviderSummaryCard: View {
    let provider: ProviderSnapshot
    let scopeTitle: String
    let currentLimitInstallationIDs: [String]
    let compact: Bool

    var body: some View {
        VStack(alignment: .leading, spacing: compact ? 6 : 8) {
            Text(provider.providerID?.title ?? provider.provider.capitalized)
                .font(.headline)
            LabeledContent("Account scope", value: self.scopeTitle)
            LabeledContent("Source", value: provider.sourceUsed)
            LabeledContent("Available", value: provider.available ? "Yes" : "No")
            LabeledContent("Today tokens", value: "\(provider.costSummary.todayTokens)")
            LabeledContent("Today cost", value: usd(provider.costSummary.todayCostUSD))
            if let primary = provider.primary {
                LabeledContent("Primary lane", value: percent(primary.usedPercent))
            }
            if let identity = provider.identity?.accountEmail {
                LabeledContent("Account", value: identity)
            } else if let organization = provider.identity?.accountOrganization {
                LabeledContent("Organization", value: organization)
            }
            if !self.currentLimitInstallationIDs.isEmpty {
                LabeledContent(
                    "Live limit sources",
                    value: self.currentLimitInstallationIDs.joined(separator: ", ")
                )
            }
            if provider.stale {
                Text("Snapshot is currently marked stale.")
                    .foregroundStyle(.orange)
            }
        }
        .cardStyle(compact: compact)
    }
}

private struct AccountSummaryRow: View {
    let summary: MobileAccountSummary

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(self.summary.option.displayTitle)
                    .font(.headline)
                Spacer()
                Text(usd(self.summary.totals.last90DaysCostUSD))
                    .foregroundStyle(.secondary)
            }
            Text("\(self.summary.totals.last90DaysTokens) tokens in 90 days")
                .foregroundStyle(.secondary)
            if self.summary.hasCurrentLimitData {
                Text("Includes live current-limit data")
                    .font(.footnote)
            }
            if self.summary.isStale {
                Text("Contains stale installation data")
                    .font(.footnote)
                    .foregroundStyle(.orange)
            }
        }
    }
}

private struct InstallationCard: View {
    let installation: SyncedInstallationSnapshot
    let compact: Bool

    var body: some View {
        VStack(alignment: .leading, spacing: compact ? 6 : 8) {
            Text(installation.sourceDevice)
                .font(.headline)
            LabeledContent("Installation", value: installation.installationID)
            LabeledContent("Published", value: displayTimestamp(installation.publishedAt))
            LabeledContent("90 day tokens", value: "\(installation.totals.last90DaysTokens)")
            if !installation.accountLabels.isEmpty {
                LabeledContent("Accounts", value: installation.accountLabels.joined(separator: ", "))
            }
            if installation.isStale {
                Text("This installation has stale provider data.")
                    .foregroundStyle(.orange)
            }
        }
        .cardStyle(compact: compact)
    }
}

private struct OtherInstallationsCard: View {
    let installations: [SyncedInstallationSnapshot]

    var body: some View {
        let totals = self.installations.reduce(
            MobileSnapshotTotals(todayTokens: 0, todayCostUSD: 0, last90DaysTokens: 0, last90DaysCostUSD: 0)
        ) { partial, installation in
            partial.merging(installation.totals)
        }
        VStack(alignment: .leading, spacing: 6) {
            Text("Other Installations")
                .font(.headline)
            LabeledContent("Count", value: "\(self.installations.count)")
            LabeledContent("90 day tokens", value: "\(totals.last90DaysTokens)")
            LabeledContent("90 day cost", value: usd(totals.last90DaysCostUSD))
        }
        .cardStyle(compact: true)
    }
}

private struct InstallationFreshnessRow: View {
    let installation: SyncedInstallationSnapshot

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(installation.sourceDevice)
                .font(.headline)
            Text(displayTimestamp(installation.publishedAt))
                .foregroundStyle(.secondary)
            if installation.isStale {
                Text("Marked stale")
                    .foregroundStyle(.orange)
            }
        }
    }
}

private struct ScopedEmptyStateCard: View {
    @Bindable var model: MobileDashboardModel

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            ScopeControlsCard(model: self.model)
            ContentUnavailableView(
                self.model.scopedEmptyStateTitle,
                systemImage: "tray",
                description: Text(self.model.scopedEmptyStateMessage)
            )
            .frame(maxWidth: .infinity)
        }
    }
}

private struct ScopedMessageCard: View {
    let message: String

    var body: some View {
        Text(self.message)
            .foregroundStyle(.secondary)
            .cardStyle(compact: true)
    }
}

private struct CollapsibleCardSection<Content: View>: View {
    let title: String
    let sectionID: String
    @Bindable var model: MobileDashboardModel
    let content: Content

    init(
        title: String,
        sectionID: String,
        model: MobileDashboardModel,
        @ViewBuilder content: () -> Content
    ) {
        self.title = title
        self.sectionID = sectionID
        self.model = model
        self.content = content()
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Text(self.title)
                    .font(.headline)
                Spacer()
                Button(self.model.isSectionCollapsed(self.sectionID) ? "Show" : "Hide") {
                    self.model.toggleSectionCollapsed(self.sectionID)
                }
                .font(.footnote)
            }

            if !self.model.isSectionCollapsed(self.sectionID) {
                self.content
            } else {
                Text("Collapsed")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }
        }
        .cardStyle(compact: self.model.compressionPreference == .compact)
    }
}

private struct MobileProviderChartsSection: View {
    let provider: ProviderSnapshot
    let compact: Bool

    var body: some View {
        let costSummary = self.provider.costSummary
        let dailyBreakdownPoints = costSummary.daily.compactMap { point -> MobileTokenStackChart.DayBreakdown? in
            guard let breakdown = point.breakdown else {
                return nil
            }
            return MobileTokenStackChart.DayBreakdown(day: point.day, breakdown: breakdown)
        }

        VStack(alignment: .leading, spacing: 12) {
            if !costSummary.daily.isEmpty {
                MobileDailyCostChart(daily: costSummary.daily, compact: self.compact)
                MobileCumulativeSpendChart(daily: costSummary.daily, compact: self.compact)
            }

            if !dailyBreakdownPoints.isEmpty {
                MobileTokenStackChart(points: dailyBreakdownPoints, compact: self.compact)
            }

            if costSummary.todayBreakdown != nil || costSummary.last30DaysBreakdown != nil {
                MobileTokenMixSection(
                    todayBreakdown: costSummary.todayBreakdown,
                    trailingBreakdown: costSummary.last30DaysBreakdown,
                    compact: self.compact
                )
            }

            if !costSummary.byModel.isEmpty {
                MobileModelDistributionDonut(rows: costSummary.byModel, compact: self.compact)
            }

            if !costSummary.hourlyActivity.isEmpty {
                MobileHourlyActivityChart(buckets: costSummary.hourlyActivity, compact: self.compact)
            }

            if !costSummary.activityHeatmap.isEmpty {
                MobileActivityHeatmap(cells: costSummary.activityHeatmap, compact: self.compact)
            }
        }
    }
}

private struct MobileChartCard<Content: View>: View {
    let title: String
    let caption: String?
    let compact: Bool
    let content: Content

    init(
        title: String,
        caption: String? = nil,
        compact: Bool,
        @ViewBuilder content: () -> Content
    ) {
        self.title = title
        self.caption = caption
        self.compact = compact
        self.content = content()
    }

    var body: some View {
        VStack(alignment: .leading, spacing: self.compact ? 10 : 12) {
            VStack(alignment: .leading, spacing: 4) {
                Text(self.title)
                    .font(.headline)
                if let caption, !caption.isEmpty {
                    Text(caption)
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                }
            }

            self.content
        }
        .cardStyle(compact: self.compact)
    }
}

private enum MobileChartTheme {
    static let areaFill = Color.primary.opacity(0.08)
    static let lineStroke = Color.primary.opacity(0.82)
    static let secondaryLineStroke = Color.primary.opacity(0.42)
    static let barFill = Color.primary.opacity(0.56)
    static let faintBarFill = Color.primary.opacity(0.18)
    static let plotFill = Color.primary.opacity(0.03)
    static let plotStroke = Color.primary.opacity(0.08)
    static let categoryScale = MobileTokenCategory.allCases.map(\.tint)

    static func chartFrame(compact: Bool, heatmap: Bool = false) -> CGFloat {
        if heatmap {
            return compact ? 172 : 192
        }
        return compact ? 144 : 168
    }
}

private struct MobileDailyCostChart: View {
    let daily: [CostHistoryPoint]
    let compact: Bool

    var body: some View {
        let entries = self.daily.compactMap { MobileDateBucket(point: $0) }
        MobileChartCard(
            title: "Daily cost",
            caption: "Rolling daily spend for the selected provider and account scope.",
            compact: self.compact
        ) {
            if entries.isEmpty {
                MobileChartEmptyState(message: "No daily cost data yet.")
            } else {
                Chart {
                    ForEach(entries) { entry in
                        AreaMark(
                            x: .value("Day", entry.date),
                            y: .value("Cost", entry.costUSD)
                        )
                        .foregroundStyle(MobileChartTheme.areaFill)
                        .interpolationMethod(.monotone)
                    }
                    ForEach(entries) { entry in
                        LineMark(
                            x: .value("Day", entry.date),
                            y: .value("Cost", entry.costUSD)
                        )
                        .foregroundStyle(MobileChartTheme.lineStroke)
                        .interpolationMethod(.monotone)
                    }
                }
                .chartYAxis {
                    AxisMarks(position: .leading, values: .automatic(desiredCount: 3)) { value in
                        AxisValueLabel {
                            if let cost = value.as(Double.self) {
                                Text(MobileMetricFormat.compactCurrency(cost))
                                    .font(.caption2.monospacedDigit())
                                    .foregroundStyle(.secondary)
                            }
                        }
                    }
                }
                .chartXAxis {
                    AxisMarks(values: .stride(by: .day, count: max(entries.count / 4, 1))) { value in
                        AxisValueLabel {
                            if let date = value.as(Date.self) {
                                Text(MobileMetricFormat.shortDay(date))
                                    .font(.caption2.monospacedDigit())
                                    .foregroundStyle(.secondary)
                            }
                        }
                    }
                }
                .chartPlotStyle { plot in
                    plot.background(MobileChartPlotBackground())
                }
                .frame(height: MobileChartTheme.chartFrame(compact: self.compact))
            }
        }
    }
}

private struct MobileCumulativeSpendChart: View {
    let daily: [CostHistoryPoint]
    let compact: Bool

    var body: some View {
        let entries = MobileCumulativeSpendChart.entries(from: self.daily)
        MobileChartCard(
            title: "Cumulative spend",
            caption: "Running total across the visible daily window.",
            compact: self.compact
        ) {
            if entries.isEmpty {
                MobileChartEmptyState(message: "No cumulative spend data yet.")
            } else {
                Chart {
                    ForEach(entries) { entry in
                        AreaMark(
                            x: .value("Day", entry.date),
                            y: .value("Cumulative", entry.cumulativeCostUSD)
                        )
                        .foregroundStyle(MobileChartTheme.areaFill)
                        .interpolationMethod(.monotone)
                    }
                    ForEach(entries) { entry in
                        LineMark(
                            x: .value("Day", entry.date),
                            y: .value("Cumulative", entry.cumulativeCostUSD)
                        )
                        .foregroundStyle(MobileChartTheme.lineStroke)
                        .interpolationMethod(.monotone)
                    }
                    if let total = entries.last?.cumulativeCostUSD {
                        let pace = total / Double(entries.count)
                        ForEach(Array(entries.enumerated()), id: \.offset) { index, entry in
                            LineMark(
                                x: .value("Day", entry.date),
                                y: .value("Pace", pace * Double(index + 1))
                            )
                            .foregroundStyle(MobileChartTheme.secondaryLineStroke)
                            .lineStyle(StrokeStyle(lineWidth: 1, dash: [4, 3]))
                            .interpolationMethod(.monotone)
                        }
                    }
                }
                .chartYAxis {
                    AxisMarks(position: .leading, values: .automatic(desiredCount: 3)) { value in
                        AxisValueLabel {
                            if let cost = value.as(Double.self) {
                                Text(MobileMetricFormat.compactCurrency(cost))
                                    .font(.caption2.monospacedDigit())
                                    .foregroundStyle(.secondary)
                            }
                        }
                    }
                }
                .chartXAxis {
                    AxisMarks(values: .stride(by: .day, count: max(entries.count / 4, 1))) { value in
                        AxisValueLabel {
                            if let date = value.as(Date.self) {
                                Text(MobileMetricFormat.shortDay(date))
                                    .font(.caption2.monospacedDigit())
                                    .foregroundStyle(.secondary)
                            }
                        }
                    }
                }
                .chartPlotStyle { plot in
                    plot.background(MobileChartPlotBackground())
                }
                .frame(height: MobileChartTheme.chartFrame(compact: self.compact))
            }
        }
    }

    private static func entries(from daily: [CostHistoryPoint]) -> [Entry] {
        var running = 0.0
        return daily.compactMap { point in
            guard let parsed = MobileDateBucket(point: point) else {
                return nil
            }
            running += parsed.costUSD
            return Entry(date: parsed.date, cumulativeCostUSD: running)
        }
    }

    private struct Entry: Identifiable {
        let date: Date
        let cumulativeCostUSD: Double

        var id: Date { self.date }
    }
}

private struct MobileTokenStackChart: View {
    struct DayBreakdown: Identifiable {
        let day: String
        let breakdown: TokenBreakdown

        var id: String { self.day }
    }

    let points: [DayBreakdown]
    let compact: Bool

    var body: some View {
        let entries = self.entries
        MobileChartCard(
            title: "Usage history",
            caption: "Daily token totals with category mix.",
            compact: self.compact
        ) {
            if entries.isEmpty {
                MobileChartEmptyState(message: "No token breakdown history yet.")
            } else {
                Chart(entries) { entry in
                    BarMark(
                        x: .value("Day", entry.dayIndex),
                        y: .value("Tokens", entry.tokens)
                    )
                    .foregroundStyle(by: .value("Category", entry.category.label))
                }
                .chartForegroundStyleScale(
                    domain: MobileTokenCategory.allCases.map(\.label),
                    range: MobileChartTheme.categoryScale
                )
                .chartLegend(position: .bottom, spacing: 8)
                .chartYAxis {
                    AxisMarks(position: .leading, values: .automatic(desiredCount: 3)) { value in
                        AxisValueLabel {
                            if let tokens = value.as(Int.self) {
                                Text(MobileMetricFormat.compactTokens(tokens))
                                    .font(.caption2.monospacedDigit())
                                    .foregroundStyle(.secondary)
                            }
                        }
                    }
                }
                .chartXAxis {
                    AxisMarks(values: Array(Set(entries.map(\.dayIndex))).sorted()) { value in
                        AxisValueLabel {
                            if let index = value.as(Int.self),
                               self.points.indices.contains(index) {
                                Text(MobileMetricFormat.shortDay(self.points[index].day))
                                    .font(.caption2.monospacedDigit())
                                    .foregroundStyle(.secondary)
                            }
                        }
                    }
                }
                .chartPlotStyle { plot in
                    plot.background(MobileChartPlotBackground())
                }
                .frame(height: MobileChartTheme.chartFrame(compact: self.compact))
            }
        }
    }

    private var entries: [Entry] {
        self.points.enumerated().flatMap { offset, point in
            MobileTokenCategory.allCases.compactMap { category in
                let tokens = category.tokens(in: point.breakdown)
                guard tokens > 0 else {
                    return nil
                }
                return Entry(dayIndex: offset, category: category, tokens: tokens)
            }
        }
    }

    private struct Entry: Identifiable {
        let dayIndex: Int
        let category: MobileTokenCategory
        let tokens: Int

        var id: String { "\(self.dayIndex)-\(self.category.label)" }
    }
}

private struct MobileTokenMixSection: View {
    let todayBreakdown: TokenBreakdown?
    let trailingBreakdown: TokenBreakdown?
    let compact: Bool

    var body: some View {
        MobileChartCard(
            title: "Token mix",
            caption: "Current and trailing mix across input, output, cache, and reasoning lanes.",
            compact: self.compact
        ) {
            ViewThatFits(in: .horizontal) {
                HStack(alignment: .top, spacing: 12) {
                    if let todayBreakdown {
                        MobileTokenBreakdownDonut(title: "Today", breakdown: todayBreakdown, compact: self.compact)
                    }
                    if let trailingBreakdown {
                        MobileTokenBreakdownDonut(title: "30 days", breakdown: trailingBreakdown, compact: self.compact)
                    }
                }
                VStack(alignment: .leading, spacing: 12) {
                    if let todayBreakdown {
                        MobileTokenBreakdownDonut(title: "Today", breakdown: todayBreakdown, compact: self.compact)
                    }
                    if let trailingBreakdown {
                        MobileTokenBreakdownDonut(title: "30 days", breakdown: trailingBreakdown, compact: self.compact)
                    }
                }
            }
        }
    }
}

private struct MobileTokenBreakdownDonut: View {
    let title: String
    let breakdown: TokenBreakdown
    let compact: Bool

    var body: some View {
        let entries = MobileTokenCategory.allCases.compactMap { category -> Entry? in
            let tokens = category.tokens(in: self.breakdown)
            guard tokens > 0 else {
                return nil
            }
            return Entry(category: category, tokens: tokens)
        }

        VStack(alignment: .leading, spacing: 10) {
            Text(self.title)
                .font(.subheadline.weight(.semibold))

            if entries.isEmpty {
                MobileChartEmptyState(message: "No token mix yet.")
            } else {
                HStack(alignment: .center, spacing: 12) {
                    ZStack {
                        Chart(entries) { entry in
                            SectorMark(
                                angle: .value("Tokens", entry.tokens),
                                innerRadius: .ratio(0.65),
                                outerRadius: .ratio(0.98)
                            )
                            .foregroundStyle(by: .value("Category", entry.category.label))
                        }
                        .chartForegroundStyleScale(
                            domain: MobileTokenCategory.allCases.map(\.label),
                            range: MobileChartTheme.categoryScale
                        )
                        .chartLegend(.hidden)
                        .frame(width: self.compact ? 112 : 128, height: self.compact ? 112 : 128)

                        VStack(spacing: 2) {
                            Text(MobileMetricFormat.compactTokens(self.breakdown.total))
                                .font(.caption.monospacedDigit().weight(.semibold))
                            Text("total")
                                .font(.caption2)
                                .foregroundStyle(.secondary)
                        }
                    }

                    VStack(alignment: .leading, spacing: 6) {
                        ForEach(entries) { entry in
                            HStack(spacing: 6) {
                                RoundedRectangle(cornerRadius: 2, style: .continuous)
                                    .fill(entry.category.tint)
                                    .frame(width: 10, height: 10)
                                Text(entry.category.label)
                                    .font(.caption)
                                Spacer()
                                Text(MobileMetricFormat.percent(entry.tokens, of: self.breakdown.total))
                                    .font(.caption.monospacedDigit())
                                    .foregroundStyle(.secondary)
                            }
                        }
                    }
                }
            }
        }
    }

    private struct Entry: Identifiable {
        let category: MobileTokenCategory
        let tokens: Int

        var id: String { self.category.label }
    }
}

private struct MobileModelDistributionDonut: View {
    let rows: [ProviderModelRow]
    let compact: Bool

    var body: some View {
        let families = self.families
        let palette = self.palette
        MobileChartCard(
            title: "Model mix",
            caption: "30-day spend share by model family.",
            compact: self.compact
        ) {
            if families.isEmpty {
                MobileChartEmptyState(message: "No model distribution yet.")
            } else {
                ViewThatFits(in: .horizontal) {
                    HStack(alignment: .center, spacing: 12) {
                        self.donut(families: families, palette: palette)
                        self.legend(families: families, palette: palette)
                    }
                    VStack(alignment: .leading, spacing: 12) {
                        self.donut(families: families, palette: palette)
                        self.legend(families: families, palette: palette)
                    }
                }
            }
        }
    }

    private var families: [FamilyEntry] {
        var grouped: [String: Double] = [:]
        for row in self.rows {
            grouped[Self.modelFamilyLabel(row.model), default: 0] += row.costUSD
        }
        return grouped
            .map { FamilyEntry(label: $0.key, costUSD: $0.value) }
            .sorted { $0.costUSD > $1.costUSD }
            .prefix(8)
            .map { $0 }
    }

    private var palette: [String: Color] {
        let colors: [Color] = [
            .accentColor,
            Color.primary.opacity(0.88),
            Color.primary.opacity(0.58),
            Color.primary.opacity(0.32),
            Color.primary.opacity(0.16),
        ]
        return Dictionary(uniqueKeysWithValues: self.families.enumerated().map { index, family in
            (family.label, colors[index % colors.count])
        })
    }

    private func donut(families: [FamilyEntry], palette: [String: Color]) -> some View {
        Chart(families) { family in
            SectorMark(
                angle: .value("Cost", family.costUSD),
                innerRadius: .ratio(0.62),
                outerRadius: .ratio(0.98)
            )
            .foregroundStyle(by: .value("Family", family.label))
        }
        .chartForegroundStyleScale(
            domain: families.map(\.label),
            range: families.map { palette[$0.label] ?? Color.primary.opacity(0.16) }
        )
        .chartLegend(.hidden)
        .frame(width: self.compact ? 120 : 136, height: self.compact ? 120 : 136)
    }

    private func legend(families: [FamilyEntry], palette: [String: Color]) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            ForEach(families) { family in
                HStack(spacing: 6) {
                    RoundedRectangle(cornerRadius: 2, style: .continuous)
                        .fill(palette[family.label] ?? Color.primary.opacity(0.16))
                        .frame(width: 10, height: 10)
                    Text(family.label)
                        .font(.caption)
                        .lineLimit(1)
                    Spacer()
                    Text(MobileMetricFormat.compactCurrency(family.costUSD))
                        .font(.caption.monospacedDigit())
                        .foregroundStyle(.secondary)
                }
            }
        }
    }

    private static func modelFamilyLabel(_ model: String) -> String {
        let lower = model.lowercased()
        if lower.hasPrefix("claude-opus") { return "Opus" }
        if lower.hasPrefix("claude-sonnet") { return "Sonnet" }
        if lower.hasPrefix("claude-haiku") { return "Haiku" }
        if lower.hasPrefix("gpt-5") { return "GPT-5" }
        if lower.hasPrefix("gpt-") { return "GPT" }
        let first = model.split(separator: "-").first.map(String.init) ?? model
        return first.prefix(1).uppercased() + first.dropFirst()
    }

    private struct FamilyEntry: Identifiable {
        let label: String
        let costUSD: Double

        var id: String { self.label }
    }
}

private struct MobileHourlyActivityChart: View {
    let buckets: [ProviderHourlyBucket]
    let compact: Bool

    var body: some View {
        MobileChartCard(
            title: "Activity by hour",
            caption: "30-day turn volume grouped by local hour.",
            compact: self.compact
        ) {
            if self.buckets.isEmpty || !self.buckets.contains(where: { $0.turns > 0 }) {
                MobileChartEmptyState(message: "No hourly activity yet.")
            } else {
                Chart(self.buckets) { bucket in
                    BarMark(
                        x: .value("Hour", bucket.hour),
                        y: .value("Turns", bucket.turns)
                    )
                    .foregroundStyle(bucket.turns > 0 ? MobileChartTheme.barFill : MobileChartTheme.faintBarFill)
                }
                .chartXScale(domain: 0...23)
                .chartYAxis {
                    AxisMarks(position: .leading, values: .automatic(desiredCount: 3)) { value in
                        AxisValueLabel {
                            if let turns = value.as(Int.self) {
                                Text("\(turns)")
                                    .font(.caption2.monospacedDigit())
                                    .foregroundStyle(.secondary)
                            }
                        }
                    }
                }
                .chartXAxis {
                    AxisMarks(values: [0, 6, 12, 18, 23]) { value in
                        AxisValueLabel {
                            if let hour = value.as(Int.self) {
                                Text(String(format: "%02d:00", hour))
                                    .font(.caption2.monospacedDigit())
                                    .foregroundStyle(.secondary)
                            }
                        }
                    }
                }
                .chartPlotStyle { plot in
                    plot.background(MobileChartPlotBackground())
                }
                .frame(height: MobileChartTheme.chartFrame(compact: self.compact))
            }
        }
    }
}

private struct MobileActivityHeatmap: View {
    let cells: [ProviderHeatmapCell]
    let compact: Bool

    private static let dayLabels = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]

    var body: some View {
        let grid = self.grid
        let maxTurns = grid.flatMap { $0 }.max() ?? 0
        MobileChartCard(
            title: "Activity heatmap",
            caption: "7 × 24 activity map over the last 30 days.",
            compact: self.compact
        ) {
            if maxTurns == 0 {
                MobileChartEmptyState(message: "No heatmap activity yet.")
            } else {
                VStack(alignment: .leading, spacing: 6) {
                    ForEach(0..<7, id: \.self) { day in
                        HStack(alignment: .center, spacing: 6) {
                            Text(Self.dayLabels[day])
                                .font(.caption2.monospacedDigit())
                                .foregroundStyle(.secondary)
                                .frame(width: 28, alignment: .leading)
                            HStack(spacing: 2) {
                                ForEach(0..<24, id: \.self) { hour in
                                    RoundedRectangle(cornerRadius: 2, style: .continuous)
                                        .fill(Color.primary.opacity(self.opacity(for: grid[day][hour], maxTurns: maxTurns)))
                                        .frame(maxWidth: .infinity, minHeight: self.compact ? 10 : 12)
                                }
                            }
                        }
                    }
                    HStack(spacing: 6) {
                        Text("Hour")
                            .font(.caption2.monospacedDigit())
                            .foregroundStyle(.secondary)
                            .frame(width: 28, alignment: .leading)
                        HStack(spacing: 2) {
                            ForEach(0..<24, id: \.self) { hour in
                                Text([0, 6, 12, 18, 23].contains(hour) ? "\(hour)" : "")
                                    .font(.caption2.monospacedDigit())
                                    .foregroundStyle(.secondary)
                                    .frame(maxWidth: .infinity, alignment: .center)
                            }
                        }
                    }
                    .padding(.top, 2)
                }
                .frame(height: MobileChartTheme.chartFrame(compact: self.compact, heatmap: true))
            }
        }
    }

    private var grid: [[Int]] {
        var rows = Array(repeating: Array(repeating: 0, count: 24), count: 7)
        for cell in self.cells where (0..<7).contains(cell.dayOfWeek) && (0..<24).contains(cell.hour) {
            rows[cell.dayOfWeek][cell.hour] = cell.turns
        }
        return rows
    }

    private func opacity(for turns: Int, maxTurns: Int) -> Double {
        guard turns > 0, maxTurns > 0 else {
            return 0.04
        }
        let normalized = Double(turns) / Double(maxTurns)
        return 0.12 + (normalized * 0.74)
    }
}

private struct MobileChartEmptyState: View {
    let message: String

    var body: some View {
        Text(self.message)
            .font(.footnote)
            .foregroundStyle(.secondary)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.vertical, 6)
    }
}

private struct MobileChartPlotBackground: View {
    var body: some View {
        RoundedRectangle(cornerRadius: 10, style: .continuous)
            .fill(MobileChartTheme.plotFill)
            .overlay {
                RoundedRectangle(cornerRadius: 10, style: .continuous)
                    .stroke(MobileChartTheme.plotStroke, lineWidth: 1)
            }
    }
}

private struct MobileDateBucket: Identifiable {
    let rawDay: String
    let date: Date
    let costUSD: Double

    var id: String { self.rawDay }

    init?(point: CostHistoryPoint) {
        guard let date = MobileMetricFormat.dayFormatter.date(from: point.day) else {
            return nil
        }
        self.rawDay = point.day
        self.date = date
        self.costUSD = point.costUSD
    }
}

private enum MobileTokenCategory: CaseIterable {
    case input
    case output
    case cacheRead
    case cacheCreation
    case reasoning

    var label: String {
        switch self {
        case .input:
            return "Input"
        case .output:
            return "Output"
        case .cacheRead:
            return "Cache Read"
        case .cacheCreation:
            return "Cache Create"
        case .reasoning:
            return "Reasoning"
        }
    }

    var tint: Color {
        switch self {
        case .input:
            return .accentColor
        case .output:
            return Color.primary.opacity(0.82)
        case .cacheRead:
            return Color.primary.opacity(0.58)
        case .cacheCreation:
            return Color.primary.opacity(0.36)
        case .reasoning:
            return Color.primary.opacity(0.18)
        }
    }

    func tokens(in breakdown: TokenBreakdown) -> Int {
        switch self {
        case .input:
            return breakdown.input
        case .output:
            return breakdown.output
        case .cacheRead:
            return breakdown.cacheRead
        case .cacheCreation:
            return breakdown.cacheCreation
        case .reasoning:
            return breakdown.reasoningOutput
        }
    }
}

private enum MobileMetricFormat {
    static let dayFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.calendar = Calendar(identifier: .gregorian)
        formatter.locale = Locale(identifier: "en_US_POSIX")
        formatter.dateFormat = "yyyy-MM-dd"
        return formatter
    }()

    static let shortDayFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.setLocalizedDateFormatFromTemplate("MMM d")
        return formatter
    }()

    static func shortDay(_ date: Date) -> String {
        Self.shortDayFormatter.string(from: date)
    }

    static func shortDay(_ rawDay: String) -> String {
        guard let date = Self.dayFormatter.date(from: rawDay) else {
            return rawDay
        }
        return Self.shortDay(date)
    }

    static func compactCurrency(_ value: Double) -> String {
        if value >= 1_000 {
            return String(format: "$%.0f", value)
        }
        if value >= 10 {
            return String(format: "$%.1f", value)
        }
        return String(format: "$%.2f", value)
    }

    static func compactTokens(_ count: Int) -> String {
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

    static func percent(_ value: Int, of total: Int) -> String {
        guard total > 0 else {
            return "0%"
        }
        let share = (Double(value) / Double(total)) * 100
        if share < 1 {
            return "<1%"
        }
        if share >= 10 {
            return "\(Int(share.rounded()))%"
        }
        return String(format: "%.1f%%", share)
    }
}

private struct AliasEditorSheet: View {
    @Environment(\.dismiss) private var dismiss
    let model: MobileDashboardModel
    @State private var draftAliases: [MobileAccountAlias]

    init(model: MobileDashboardModel) {
        self.model = model
        self._draftAliases = State(initialValue: model.aliases)
    }

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    Button("Add Alias") {
                        self.draftAliases.append(
                            MobileAccountAlias(title: "New Group", sourceLabelKeys: [])
                        )
                    }
                }

                ForEach(Array(self.draftAliases.enumerated()), id: \.element.id) { index, alias in
                    Section("Alias \(index + 1)") {
                        TextField(
                            "Alias title",
                            text: Binding(
                                get: { self.draftAliases[index].title },
                                set: { self.draftAliases[index].title = $0 }
                            )
                        )

                        ForEach(self.model.accountSources) { source in
                            Toggle(
                                source.displayTitle,
                                isOn: Binding(
                                    get: {
                                        self.draftAliases[index].sourceLabelKeys.contains(source.key)
                                    },
                                    set: { isOn in
                                        var keys = Set(self.draftAliases[index].sourceLabelKeys)
                                        if isOn {
                                            keys.insert(source.key)
                                        } else {
                                            keys.remove(source.key)
                                        }
                                        self.draftAliases[index].sourceLabelKeys = Array(keys).sorted()
                                    }
                                )
                            )
                        }

                        Button("Delete Alias", role: .destructive) {
                            self.draftAliases.removeAll { $0.id == alias.id }
                        }
                    }
                }
            }
            .navigationTitle("Account Aliases")
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Button("Cancel") {
                        self.model.dismissAliasEditor()
                        self.dismiss()
                    }
                }
                ToolbarItem(placement: .topBarTrailing) {
                    Button("Save") {
                        let normalized = self.draftAliases.filter { !$0.sourceLabelKeys.isEmpty }
                        self.model.replaceAliases(normalized)
                        self.model.dismissAliasEditor()
                        self.dismiss()
                    }
                }
            }
        }
    }
}

private extension View {
    func cardStyle(compact: Bool = false) -> some View {
        self
            .padding(compact ? 12 : 16)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(Color(uiColor: .secondarySystemBackground), in: RoundedRectangle(cornerRadius: 16))
    }
}

private func usd(_ value: Double) -> String {
    value.formatted(.currency(code: "USD"))
}

private func percent(_ value: Double) -> String {
    "\(Int(value.rounded()))%"
}

private func displayTimestamp(_ raw: String) -> String {
    let formatter = ISO8601DateFormatter()
    guard let date = formatter.date(from: raw) else { return raw }
    return date.formatted(date: .abbreviated, time: .shortened)
}
