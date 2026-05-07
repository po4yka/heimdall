import HeimdallDomain
import SwiftUI

struct WindowCostModelsView: View {
    @Bindable var model: CostModelsFeatureModel

    var body: some View {
        VStack(alignment: .leading, spacing: 24) {
            WindowHeader(
                title: "Cost & Models",
                subtitle: "Spend breakdown by model, tool, and version",
                issue: WindowHeaderIssuePresentation.make(message: self.model.globalIssueLabel),
                onRetry: { Task { await self.model.refreshAll() } },
                isRetrying: self.model.isRefreshing
            ) {
                EmptyView()
            }

            CostModelsKpiRow(model: self.model)

            CostModelsModelSection(model: self.model)

            CostModelsToolSection(model: self.model)

            CostModelsMcpSection(model: self.model)

            CostModelsVersionSection(model: self.model)
        }
        .task { await self.model.refreshAll() }
    }
}

// MARK: - KPI row

private struct CostModelsKpiRow: View {
    let model: CostModelsFeatureModel

    private var totalCostUSD: Double {
        self.model.byModel.reduce(0) { $0 + $1.costUSD }
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
            WindowOverviewKpiTile(label: "Models", value: "\(self.model.byModel.count)")
            WindowOverviewKpiTile(label: "Tools", value: "\(self.model.byTool.count)")
            WindowOverviewKpiTile(label: "MCP servers", value: "\(self.model.byMcp.count)")
        }
    }
}

// MARK: - Model sections

private struct CostModelsModelSection: View {
    let model: CostModelsFeatureModel

    var body: some View {
        if !self.model.byModel.isEmpty {
            VStack(alignment: .leading, spacing: 14) {
                WindowSectionHeader(
                    title: "Cost by model",
                    subtitle: "Input, output, and cache token costs per model"
                )

                ModelCostTable(rows: self.model.byModel)
                    .padding(18)
                    .menuCardBackground(opacity: 0.04, cornerRadius: 16)

                ModelDistributionDonut(rows: self.model.byModel)
                    .padding(18)
                    .menuCardBackground(opacity: 0.04, cornerRadius: 16)
            }
        }
    }
}

// MARK: - Tool section

private struct CostModelsToolSection: View {
    let model: CostModelsFeatureModel

    var body: some View {
        if !self.model.byTool.isEmpty {
            VStack(alignment: .leading, spacing: 14) {
                WindowSectionHeader(
                    title: "Tool usage",
                    subtitle: "Top tools by invocation count"
                )

                ToolUsageTable(rows: self.model.byTool)
                    .padding(18)
                    .menuCardBackground(opacity: 0.04, cornerRadius: 16)
            }
        }
    }
}

// MARK: - MCP section

private struct CostModelsMcpSection: View {
    let model: CostModelsFeatureModel

    var body: some View {
        if !self.model.byMcp.isEmpty {
            VStack(alignment: .leading, spacing: 14) {
                WindowSectionHeader(
                    title: "MCP servers",
                    subtitle: "Server activity over 30 days"
                )

                McpSummaryTable(rows: self.model.byMcp)
                    .padding(18)
                    .menuCardBackground(opacity: 0.04, cornerRadius: 16)
            }
        }
    }
}

// MARK: - Version section

private struct CostModelsVersionSection: View {
    let model: CostModelsFeatureModel

    var body: some View {
        if !self.model.versionBreakdown.isEmpty {
            VStack(alignment: .leading, spacing: 14) {
                WindowSectionHeader(
                    title: "CLI versions",
                    subtitle: "Version distribution by cost"
                )

                VersionDistributionDonut(rows: self.model.versionBreakdown)
                    .padding(18)
                    .menuCardBackground(opacity: 0.04, cornerRadius: 16)
            }
        }
    }
}
