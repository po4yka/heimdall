import HeimdallDomain
import Observation
import SwiftUI

struct HeimdallMobileRootView: View {
    @State var model: MobileDashboardModel

    var body: some View {
        NavigationStack {
            Group {
                if self.model.isLoading && !self.model.hasSnapshot {
                    ProgressView("Loading synced snapshot…")
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else if let snapshot = self.model.snapshot {
                    TabView {
                        OverviewTab(model: self.model, snapshot: snapshot)
                            .tabItem { Label("Overview", systemImage: "rectangle.grid.2x2") }
                        HistoryTab(model: self.model, snapshot: snapshot)
                            .tabItem { Label("History", systemImage: "chart.line.uptrend.xyaxis") }
                        FreshnessTab(snapshot: snapshot)
                            .tabItem { Label("Freshness", systemImage: "clock") }
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
                        Task { await self.model.load() }
                    }
                }
            }
        }
        .task {
            await self.model.load()
        }
    }
}

private struct OverviewTab: View {
    @Bindable var model: MobileDashboardModel
    let snapshot: MobileSnapshotEnvelope

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                if self.snapshot.providers.count > 1 {
                    ProviderPicker(model: self.model)
                }

                VStack(alignment: .leading, spacing: 8) {
                    Text("Totals")
                        .font(.headline)
                    LabeledContent("Today tokens", value: "\(self.snapshot.totals.todayTokens)")
                    LabeledContent("Today cost", value: usd(self.snapshot.totals.todayCostUSD))
                    LabeledContent("Last 90 days", value: "\(self.snapshot.totals.last90DaysTokens) tokens")
                    LabeledContent("90 day cost", value: usd(self.snapshot.totals.last90DaysCostUSD))
                    LabeledContent("Source device", value: self.snapshot.sourceDevice)
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
            }
            .padding()
        }
    }
}

private struct HistoryTab: View {
    @Bindable var model: MobileDashboardModel
    let snapshot: MobileSnapshotEnvelope

    var body: some View {
        List {
            if self.snapshot.providers.count > 1 {
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
    }
}

private struct FreshnessTab: View {
    let snapshot: MobileSnapshotEnvelope

    var body: some View {
        List {
            Section("Snapshot") {
                LabeledContent("Generated", value: displayTimestamp(self.snapshot.generatedAt))
                LabeledContent("Source device", value: self.snapshot.sourceDevice)
                LabeledContent(
                    "Stale providers",
                    value: self.snapshot.freshness.hasStaleProviders
                        ? self.snapshot.freshness.staleProviders.joined(separator: ", ")
                        : "None"
                )
            }

            Section("Provider refreshes") {
                ForEach(self.snapshot.providers) { provider in
                    VStack(alignment: .leading, spacing: 4) {
                        Text(provider.providerID?.title ?? provider.provider.capitalized)
                            .font(.headline)
                        Text(displayTimestamp(provider.lastRefresh))
                            .foregroundStyle(.secondary)
                        if provider.stale {
                            Text("Marked stale")
                                .foregroundStyle(.orange)
                        }
                    }
                }
            }
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
