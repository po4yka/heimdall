import HeimdallDomain
import SwiftUI

struct WindowSessionsView: View {
    @Bindable var model: SessionsFeatureModel

    var body: some View {
        VStack(alignment: .leading, spacing: 24) {
            WindowHeader(
                title: "Sessions",
                subtitle: "Recent session history by cost and model",
                issue: WindowHeaderIssuePresentation.make(message: self.model.globalIssueLabel),
                onRetry: { Task { await self.model.refreshAll() } },
                isRetrying: self.model.isRefreshing
            ) {
                EmptyView()
            }

            SessionsKpiRow(model: self.model)

            SessionsRecentSection(model: self.model)

            SessionsModelSection(model: self.model)

            SessionsProjectSection(model: self.model)
        }
        .task { await self.model.refreshAll() }
    }
}

// MARK: - KPI row

private struct SessionsKpiRow: View {
    let model: SessionsFeatureModel

    private var totalCostUSD: Double {
        self.model.recentSessions.reduce(0) { $0 + $1.costUSD }
    }

    private var totalTurns: Int {
        self.model.recentSessions.reduce(0) { $0 + $1.turns }
    }

    var body: some View {
        LazyVGrid(
            columns: [
                GridItem(.flexible()), GridItem(.flexible()),
                GridItem(.flexible()), GridItem(.flexible())
            ],
            spacing: 12
        ) {
            WindowOverviewKpiTile(label: "Total spend", value: FormatHelpers.formatUSD(self.totalCostUSD))
            WindowOverviewKpiTile(label: "Sessions", value: "\(self.model.recentSessions.count)")
            WindowOverviewKpiTile(label: "Turns", value: "\(self.totalTurns)")
            WindowOverviewKpiTile(label: "Models", value: "\(self.model.byModel.count)")
        }
    }
}

// MARK: - Recent sessions section

private struct SessionsRecentSection: View {
    let model: SessionsFeatureModel

    var body: some View {
        if !self.model.recentSessions.isEmpty {
            VStack(alignment: .leading, spacing: 14) {
                WindowSectionHeader(
                    title: "Recent sessions",
                    subtitle: "Most recent sessions across all providers"
                )

                SessionsTable(sessions: self.model.recentSessions)
                    .padding(18)
                    .menuCardBackground(opacity: 0.04, cornerRadius: 16)
            }
        }
    }
}

// MARK: - Model breakdown section

private struct SessionsModelSection: View {
    let model: SessionsFeatureModel

    var body: some View {
        if !self.model.byModel.isEmpty {
            VStack(alignment: .leading, spacing: 14) {
                WindowSectionHeader(
                    title: "Cost by model",
                    subtitle: "Spend per model across all sessions"
                )

                ModelCostTable(rows: self.model.byModel)
                    .padding(18)
                    .menuCardBackground(opacity: 0.04, cornerRadius: 16)
            }
        }
    }
}

// MARK: - Project breakdown section

private struct SessionsProjectSection: View {
    let model: SessionsFeatureModel

    var body: some View {
        if !self.model.byProject.isEmpty {
            VStack(alignment: .leading, spacing: 14) {
                WindowSectionHeader(
                    title: "Cost by project",
                    subtitle: "Top projects by session spend"
                )

                ProjectCostTable(rows: self.model.byProject)
                    .padding(18)
                    .menuCardBackground(opacity: 0.04, cornerRadius: 16)
            }
        }
    }
}
