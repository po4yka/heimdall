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
                } else if let aggregate = self.model.aggregate {
                    VStack(spacing: 0) {
                        if let warning = self.model.staleSnapshotWarning {
                            SyncWarningBanner(message: warning)
                        }

                        TabView {
                            OverviewTab(model: self.model, aggregate: aggregate)
                                .tabItem { Label("Overview", systemImage: "rectangle.grid.2x2") }
                            HistoryTab(model: self.model, aggregate: aggregate)
                                .tabItem { Label("History", systemImage: "chart.line.uptrend.xyaxis") }
                            FreshnessTab(model: self.model, aggregate: aggregate)
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
    let aggregate: SyncedAggregateEnvelope

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                if self.aggregate.aggregateProviderViews.count > 1 {
                    ProviderPicker(model: self.model)
                }

                VStack(alignment: .leading, spacing: 8) {
                    Text("All Installations")
                        .font(.headline)
                    LabeledContent("Today tokens", value: "\(self.aggregate.aggregateTotals.todayTokens)")
                    LabeledContent("Today cost", value: usd(self.aggregate.aggregateTotals.todayCostUSD))
                    LabeledContent("Last 90 days", value: "\(self.aggregate.aggregateTotals.last90DaysTokens) tokens")
                    LabeledContent("90 day cost", value: usd(self.aggregate.aggregateTotals.last90DaysCostUSD))
                    LabeledContent("Installations", value: "\(self.aggregate.installations.count)")
                }
                .cardStyle()

                if let provider = self.model.selectedProviderSnapshot {
                    VStack(alignment: .leading, spacing: 8) {
                        Text(provider.providerID?.title ?? provider.provider.capitalized)
                            .font(.headline)
                        LabeledContent("Source", value: provider.sourceUsed)
                        LabeledContent("Available", value: provider.available ? "Yes" : "No")
                        LabeledContent("Today tokens", value: "\(provider.costSummary.todayTokens)")
                        LabeledContent("Today cost", value: usd(provider.costSummary.todayCostUSD))
                        if let primary = provider.primary {
                            LabeledContent("Primary lane", value: percent(primary.usedPercent))
                        }
                        if let identity = provider.identity?.accountEmail {
                            LabeledContent("Account", value: identity)
                        }
                        if provider.stale {
                            Text("Snapshot is currently marked stale.")
                                .foregroundStyle(.orange)
                        }
                    }
                    .cardStyle()
                }

                ForEach(self.aggregate.installations) { installation in
                    VStack(alignment: .leading, spacing: 8) {
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
                    .cardStyle()
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
    let aggregate: SyncedAggregateEnvelope

    var body: some View {
        List {
            if self.aggregate.aggregateProviderViews.count > 1 {
                Section {
                    ProviderPicker(model: self.model)
                }
            }

            if let history = self.model.selectedHistorySeries {
                Section("Recent trend") {
                    LabeledContent("90 day tokens", value: "\(history.totalTokens)")
                    LabeledContent("90 day cost", value: usd(history.totalCostUSD))
                }

                Section("Latest days") {
                    ForEach(Array(history.daily.suffix(14).reversed())) { point in
                        VStack(alignment: .leading, spacing: 4) {
                            Text(point.day)
                                .font(.headline)
                            Text("\(point.totalTokens) tokens · \(usd(point.costUSD))")
                                .foregroundStyle(.secondary)
                        }
                    }
                }
            } else {
                Section {
                    Text("No provider history is available yet.")
                        .foregroundStyle(.secondary)
                }
            }
        }
        .refreshable {
            await self.model.refresh(reason: .manual)
        }
    }
}

private struct FreshnessTab: View {
    @Bindable var model: MobileDashboardModel
    let aggregate: SyncedAggregateEnvelope

    var body: some View {
        List {
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

            Section("Snapshot") {
                LabeledContent("Generated", value: displayTimestamp(self.aggregate.generatedAt))
                LabeledContent("Installations", value: "\(self.aggregate.installations.count)")
                LabeledContent(
                    "Stale installations",
                    value: !self.aggregate.staleInstallations.isEmpty
                        ? self.aggregate.staleInstallations.joined(separator: ", ")
                        : "None"
                )
            }

            Section("Installation refreshes") {
                ForEach(self.aggregate.installations) { installation in
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
        }
        .refreshable {
            await self.model.refresh(reason: .manual)
        }
    }
}

private struct ProviderPicker: View {
    @Bindable var model: MobileDashboardModel

    var body: some View {
        Picker("Provider", selection: self.$model.selectedProvider) {
            ForEach(self.model.providerSnapshots.compactMap(\.providerID)) { provider in
                Text(provider.title).tag(provider)
            }
        }
        .pickerStyle(.segmented)
    }
}

private extension View {
    func cardStyle() -> some View {
        self
            .padding()
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
