import HeimdallDomain
import SwiftUI

struct WindowAgentsView: View {
    @Bindable var model: AgentsFeatureModel

    var body: some View {
        VStack(alignment: .leading, spacing: 24) {
            WindowHeader(
                title: "Agents",
                subtitle: "Subagent and tool activity",
                issue: WindowHeaderIssuePresentation.make(message: self.model.globalIssueLabel),
                onRetry: { Task { await self.model.refreshAll() } },
                isRetrying: self.model.isRefreshing
            ) {
                EmptyView()
            }

            if self.model.hasAgentData {
                AgentsSummarySection(model: self.model)
                AgentsToolSection(model: self.model)
                AgentsSessionsSection(model: self.model)
            } else {
                AgentsEmptyBanner()
            }
        }
        .task { await self.model.refreshAll() }
    }
}

// MARK: - Empty state

private struct AgentsEmptyBanner: View {
    var body: some View {
        ContentUnavailableView(
            "No agent activity",
            systemImage: "person.3",
            description: Text("Subagent data appears once Claude Code runs parallel agents in your sessions.")
        )
        .frame(maxWidth: .infinity)
        .padding(.vertical, 48)
    }
}

// MARK: - Summary section

private struct AgentsSummarySection: View {
    let model: AgentsFeatureModel

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            WindowSectionHeader(
                title: "Subagent overview",
                subtitle: "Parallel sidechain activity over 30 days"
            )

            if let breakdown = self.model.subagentBreakdown {
                AgentsKpiRow(breakdown: breakdown)

                SubagentSummaryCard(breakdown: breakdown)
                    .padding(18)
                    .menuCardBackground(opacity: 0.04, cornerRadius: 16)
            }
        }
    }
}

// MARK: - KPI row

private struct AgentsKpiRow: View {
    let breakdown: ProviderSubagentBreakdown

    var body: some View {
        LazyVGrid(
            columns: [
                GridItem(.flexible()), GridItem(.flexible()),
                GridItem(.flexible()), GridItem(.flexible())
            ],
            spacing: 12
        ) {
            WindowOverviewKpiTile(label: "Agent turns", value: "\(self.breakdown.totalTurns)")
            WindowOverviewKpiTile(label: "Agent spend", value: FormatHelpers.formatUSD(self.breakdown.totalCostUSD))
            WindowOverviewKpiTile(label: "Sessions", value: "\(self.breakdown.sessionCount)")
            WindowOverviewKpiTile(label: "Agents", value: "\(self.breakdown.agentCount)")
        }
    }
}

// MARK: - Tool usage section

private struct AgentsToolSection: View {
    let model: AgentsFeatureModel

    var body: some View {
        if !self.model.byTool.isEmpty {
            VStack(alignment: .leading, spacing: 14) {
                WindowSectionHeader(
                    title: "Tool usage",
                    subtitle: "Tool invocations across all agent sessions"
                )

                ToolUsageTable(rows: self.model.byTool)
                    .padding(18)
                    .menuCardBackground(opacity: 0.04, cornerRadius: 16)
            }
        }
    }
}

// MARK: - Recent sessions section

private struct AgentsSessionsSection: View {
    let model: AgentsFeatureModel

    private var agentSessions: [ProviderSession] {
        self.model.recentSessions.prefix(10).map { $0 }
    }

    var body: some View {
        if !self.agentSessions.isEmpty {
            VStack(alignment: .leading, spacing: 14) {
                WindowSectionHeader(
                    title: "Recent sessions",
                    subtitle: "Latest 10 sessions involving agents"
                )

                VStack(spacing: 0) {
                    ForEach(self.agentSessions) { session in
                        AgentSessionRow(session: session)
                        if session.id != self.agentSessions.last?.id {
                            Divider().padding(.horizontal, 12)
                        }
                    }
                }
                .padding(4)
                .menuCardBackground(opacity: 0.04, cornerRadius: 16)
            }
        }
    }
}

private struct AgentSessionRow: View {
    let session: ProviderSession

    var body: some View {
        HStack(spacing: 8) {
            VStack(alignment: .leading, spacing: 2) {
                Text(self.session.displayName)
                    .font(.caption.weight(.medium))
                    .lineLimit(1)
                Text(self.session.startedAt)
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
            Spacer()
            VStack(alignment: .trailing, spacing: 2) {
                Text(FormatHelpers.formatUSD(self.session.costUSD))
                    .font(.caption.monospacedDigit())
                Text("\(self.session.turns) turns")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
    }
}
