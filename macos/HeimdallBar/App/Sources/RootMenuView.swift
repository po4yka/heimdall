import AppKit
import HeimdallBarShared
import SwiftUI

struct RootMenuView: View {
    @Bindable var model: AppModel

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            Picker("Provider", selection: self.$model.selectedMergeTab) {
                ForEach(self.model.visibleTabs) { tab in
                    Text(tab.title).tag(tab)
                }
            }
            .pickerStyle(.segmented)

            if self.model.selectedMergeTab == .overview {
                OverviewMenuCard(projection: self.model.overviewProjection())
            } else if let provider = self.model.selectedMergeTab.providerID {
                ProviderMenuCard(projection: self.model.projection(for: provider))
            }

            Divider()

            Button(self.model.refreshActionLabel(for: self.model.selectedMergeTab)) {
                Task { await self.model.refresh(force: true, provider: self.model.selectedMergeTab.providerID) }
            }
            Button("Refresh All") {
                Task { await self.model.refresh(force: true, provider: nil) }
            }
            Button("Open Dashboard") {
                if let url = URL(string: "http://127.0.0.1:\(self.model.config.helperPort)") {
                    NSWorkspace.shared.open(url)
                }
            }
            Button("Quit HeimdallBar") {
                NSApplication.shared.terminate(nil)
            }

            if let lastError = self.model.lastError {
                Text(lastError)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
        .padding(12)
        .frame(width: 360)
    }
}

struct ProviderMenuView: View {
    @Bindable var model: AppModel
    let provider: ProviderID

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            ProviderMenuCard(projection: self.model.projection(for: self.provider))

            Divider()

            Button("Refresh \(self.provider.title)") {
                Task { await self.model.refresh(force: true, provider: self.provider) }
            }
            Button("Open Dashboard") {
                if let url = URL(string: "http://127.0.0.1:\(self.model.config.helperPort)") {
                    NSWorkspace.shared.open(url)
                }
            }
            Button("Quit HeimdallBar") {
                NSApplication.shared.terminate(nil)
            }
        }
        .padding(12)
        .frame(width: 340)
    }
}

struct ProviderMenuCard: View {
    let projection: ProviderMenuProjection

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Text(self.projection.title)
                    .font(.headline)
                Spacer()
                if self.projection.stale {
                    Text("STALE")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }
            }
            Text(self.projection.sourceLabel)
                .font(.caption)
                .foregroundStyle(.secondary)
            if let identityLabel = self.projection.identityLabel {
                Text(identityLabel)
                    .font(.caption)
            }
            ForEach(self.projection.laneSummaries, id: \.self) { summary in
                Text(summary)
                    .font(.caption)
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
                    .foregroundStyle(.secondary)
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
            Text(self.projection.lastRefreshLabel)
                .font(.caption)
                .foregroundStyle(.secondary)
            if let error = self.projection.error {
                Text(error)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
    }
}

struct OverviewMenuCard: View {
    let projection: OverviewMenuProjection

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Overview")
                .font(.headline)
            ForEach(self.projection.items) { item in
                HStack(alignment: .top) {
                    VStack(alignment: .leading, spacing: 2) {
                        Text(item.title)
                        Text(item.laneSummaries.first ?? "Unavailable")
                            .foregroundStyle(.secondary)
                    }
                    .font(.caption)
                    Spacer()
                    Text(item.costLabel.replacingOccurrences(of: "Today: ", with: ""))
                        .font(.caption)
                }
            }
            Divider()
            Text(self.projection.combinedCostLabel)
                .font(.caption)
            Text(self.projection.refreshedAtLabel)
                .font(.caption2)
                .foregroundStyle(.secondary)
        }
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
        .background(Color.primary.opacity(0.06))
        .clipShape(RoundedRectangle(cornerRadius: 8))
    }
}
