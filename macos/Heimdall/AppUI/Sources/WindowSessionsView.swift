import HeimdallDomain
import SwiftUI

struct WindowSessionsView: View {
    @Bindable var model: SessionsFeatureModel
    @Bindable var dashboardData: DashboardDataFeatureModel

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

            if let cp = self.dashboardData.contextPressure {
                ContextPressureSection(summary: cp)
            }

            SessionsRecentSection(model: self.model)

            SessionsModelSection(model: self.model)

            SessionsProjectSection(model: self.model)
        }
        .task {
            await self.model.refreshAll()
            await self.dashboardData.reload()
        }
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

// MARK: - Context pressure section

private struct ContextPressureSection: View {
    let summary: ContextPressureSummary

    private func pct(_ f: Float) -> String { String(format: "%.0f%%", f * 100) }

    private func bucketLabel(_ bucket: ContextPressureBucket) -> String {
        switch bucket {
        case .healthy: return "HEALTHY"
        case .warm: return "WARM"
        case .tight: return "TIGHT"
        case .overCompacted: return "COMPACTED"
        }
    }

    private func bucketColor(_ bucket: ContextPressureBucket) -> Color {
        switch bucket {
        case .healthy: return .secondary
        case .warm: return Color.primary.opacity(0.6)
        case .tight: return .red
        case .overCompacted: return .orange
        }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            WindowSectionHeader(
                title: "Context pressure",
                subtitle: "Peak input-token fraction of each session's context window"
            )

            LazyVGrid(
                columns: [
                    GridItem(.flexible()), GridItem(.flexible()),
                    GridItem(.flexible()), GridItem(.flexible()),
                    GridItem(.flexible()),
                ],
                spacing: 12
            ) {
                WindowOverviewKpiTile(label: "Healthy", value: "\(summary.healthyCount)")
                WindowOverviewKpiTile(label: "Warm", value: "\(summary.warmCount)")
                WindowOverviewKpiTile(label: "Tight", value: "\(summary.tightCount)")
                WindowOverviewKpiTile(label: "Compacted", value: "\(summary.overcompactedCount)")
                WindowOverviewKpiTile(label: "Avg peak", value: pct(summary.avgPeakFraction))
            }

            if !summary.rows.isEmpty {
                VStack(spacing: 0) {
                    HStack(spacing: 0) {
                        Text("SESSION").frame(maxWidth: .infinity, alignment: .leading)
                        Text("MODEL").frame(width: 120, alignment: .leading)
                        Text("TURNS").frame(width: 48, alignment: .trailing)
                        Text("PEAK").frame(width: 64, alignment: .trailing)
                        Text("STATUS").frame(width: 80, alignment: .trailing)
                    }
                    .font(.caption2.weight(.semibold).monospaced())
                    .foregroundStyle(.secondary)
                    .padding(.vertical, 4)

                    Divider().opacity(0.4)

                    ForEach(summary.rows.prefix(20)) { row in
                        HStack(spacing: 0) {
                            Text(row.sessionId.prefix(8))
                                .frame(maxWidth: .infinity, alignment: .leading)
                            Text(row.model.components(separatedBy: "-").prefix(3).joined(separator: "-"))
                                .frame(width: 120, alignment: .leading)
                                .lineLimit(1)
                            Text("\(row.turnCount)").frame(width: 48, alignment: .trailing)
                            Text(pct(row.peakFraction)).frame(width: 64, alignment: .trailing)
                            Text("[\(bucketLabel(row.bucket))]")
                                .foregroundStyle(bucketColor(row.bucket))
                                .frame(width: 80, alignment: .trailing)
                        }
                        .font(.caption2.monospaced())
                        .padding(.vertical, 3)
                        Divider().opacity(0.2)
                    }
                }
                .padding(18)
                .menuCardBackground(opacity: 0.04, cornerRadius: 16)
            }
        }
    }
}
