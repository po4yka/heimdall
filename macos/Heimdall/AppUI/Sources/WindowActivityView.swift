import HeimdallDomain
import SwiftUI

struct WindowActivityView: View {
    @Bindable var model: ActivityFeatureModel

    var body: some View {
        VStack(alignment: .leading, spacing: 24) {
            WindowHeader(
                title: "Activity",
                subtitle: "Trends & charts",
                issue: WindowHeaderIssuePresentation.make(message: self.model.globalIssueLabel),
                onRetry: { Task { await self.model.refreshAll() } },
                isRetrying: self.model.isRefreshing
            ) {
                EmptyView()
            }

            ActivityKpisRow(model: self.model)

            ActivityDailySection(model: self.model)

            ActivityTokenSection(model: self.model)

            ActivityModelSection(model: self.model)

            ActivityHourlySection(model: self.model)

            ActivityHeatmapSection(model: self.model)

            ActivityProjectSection(model: self.model)

            if self.model.providerItems.count > 1 {
                ActivityProviderComparisonSection(model: self.model)
            }
        }
        .task { await self.model.refreshAll() }
    }
}

// MARK: - KPI row

private struct ActivityKpisRow: View {
    let model: ActivityFeatureModel

    private var totalCostUSD: Double {
        self.model.dailyCosts.reduce(0) { $0 + $1.costUSD }
    }

    private var totalTurns: Int {
        self.model.byModel.reduce(0) { $0 + $1.turns }
    }

    private var activeDay: String? {
        self.model.dailyCosts.max(by: { $0.costUSD < $1.costUSD })?.day
    }

    private var peakDayCost: Double {
        self.model.dailyCosts.map { $0.costUSD }.max() ?? 0
    }

    var body: some View {
        LazyVGrid(
            columns: [
                GridItem(.flexible()), GridItem(.flexible()),
                GridItem(.flexible()), GridItem(.flexible())
            ],
            spacing: 12
        ) {
            WindowOverviewKpiTile(label: "30d cost", value: FormatHelpers.formatUSD(self.totalCostUSD))
            WindowOverviewKpiTile(label: "Turns", value: "\(self.totalTurns)")
            WindowOverviewKpiTile(label: "Models", value: "\(self.model.byModel.count)")
            WindowOverviewKpiTile(label: "Projects", value: "\(self.model.byProject.count)")
        }
    }
}

// MARK: - Daily cost section

private struct ActivityDailySection: View {
    let model: ActivityFeatureModel

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            WindowSectionHeader(
                title: "Daily cost",
                subtitle: "Cost distribution over the last 30 days"
            )

            DailyCostChart(daily: self.model.dailyCosts)
                .padding(18)
                .menuCardBackground(opacity: 0.04, cornerRadius: 16)
        }
    }
}

// MARK: - Token breakdown section

private struct ActivityTokenSection: View {
    let model: ActivityFeatureModel

    var body: some View {
        if !self.model.historyBreakdowns.isEmpty {
            VStack(alignment: .leading, spacing: 14) {
                WindowSectionHeader(
                    title: "Token breakdown",
                    subtitle: "Input, output, cache read, and cache creation by day"
                )

                TokenStackChart(breakdowns: self.model.historyBreakdowns)
                    .padding(18)
                    .menuCardBackground(opacity: 0.04, cornerRadius: 16)
            }
        }
    }
}

// MARK: - Model distribution section

private struct ActivityModelSection: View {
    let model: ActivityFeatureModel

    var body: some View {
        if !self.model.byModel.isEmpty {
            VStack(alignment: .leading, spacing: 14) {
                WindowSectionHeader(
                    title: "Model distribution",
                    subtitle: "Cost share by model family"
                )

                ModelDistributionDonut(rows: self.model.byModel)
                    .padding(18)
                    .menuCardBackground(opacity: 0.04, cornerRadius: 16)
            }
        }
    }
}

// MARK: - Hourly activity section

private struct ActivityHourlySection: View {
    let model: ActivityFeatureModel

    var body: some View {
        if !self.model.hourlyActivity.isEmpty {
            VStack(alignment: .leading, spacing: 14) {
                WindowSectionHeader(
                    title: "Activity by hour",
                    subtitle: "Turns per hour of day across 30 days"
                )

                HourlyActivityChart(buckets: self.model.hourlyActivity)
                    .padding(18)
                    .menuCardBackground(opacity: 0.04, cornerRadius: 16)
            }
        }
    }
}

// MARK: - Heatmap section

private struct ActivityHeatmapSection: View {
    let model: ActivityFeatureModel

    var body: some View {
        if !self.model.activityHeatmap.isEmpty {
            VStack(alignment: .leading, spacing: 14) {
                WindowSectionHeader(
                    title: "Weekly pattern",
                    subtitle: "Activity by day of week and hour"
                )

                ActivityHeatmap(cells: self.model.activityHeatmap)
                    .padding(18)
                    .menuCardBackground(opacity: 0.04, cornerRadius: 16)
            }
        }
    }
}

// MARK: - Project breakdown section

private struct ActivityProjectSection: View {
    let model: ActivityFeatureModel

    var body: some View {
        if !self.model.byProject.isEmpty {
            VStack(alignment: .leading, spacing: 14) {
                WindowSectionHeader(
                    title: "Projects",
                    subtitle: "Cost by project"
                )

                ProjectCostTable(rows: self.model.byProject)
                    .padding(18)
                    .menuCardBackground(opacity: 0.04, cornerRadius: 16)
            }
        }
    }
}

// MARK: - Provider comparison section

private struct ActivityProviderComparisonSection: View {
    let model: ActivityFeatureModel

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            WindowSectionHeader(
                title: "Provider comparison",
                subtitle: "Daily spend stacked by provider"
            )

            ProviderComparisonChart(items: self.model.providerItems)
                .padding(18)
                .menuCardBackground(opacity: 0.04, cornerRadius: 16)
        }
    }
}
