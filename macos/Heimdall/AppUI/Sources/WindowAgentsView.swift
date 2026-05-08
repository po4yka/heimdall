import HeimdallDomain
import SwiftUI

struct WindowAgentsView: View {
    @Bindable var model: AgentsFeatureModel
    @Bindable var dashboardData: DashboardDataFeatureModel
    @State private var showingRegistry = false

    var body: some View {
        VStack(alignment: .leading, spacing: 24) {
            WindowHeader(
                title: "Agents",
                subtitle: "Subagent and tool activity",
                issue: WindowHeaderIssuePresentation.make(message: self.model.globalIssueLabel),
                onRetry: { Task { await self.model.refreshAll() } },
                isRetrying: self.model.isRefreshing
            ) {
                Button("Agent registry") { self.showingRegistry = true }
                    .sheet(isPresented: self.$showingRegistry) {
                        AgentRegistrySheet(model: self.model)
                    }
            }

            if self.model.hasAgentData {
                AgentsSummarySection(model: self.model)
                AgentsToolSection(model: self.model)
                AgentsSessionsSection(model: self.model)
            } else {
                AgentsEmptyBanner()
            }

            if let tree = self.dashboardData.agentTree, !tree.sessions.isEmpty {
                AgentTreeSection(summary: tree)
            }
        }
        .task {
            await self.model.refreshAll()
            await self.dashboardData.reload()
        }
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

// MARK: - Agent tree section

private struct AgentTreeSection: View {
    let summary: AgentTreeSummary

    private func fmtCost(_ nanos: Int64) -> String {
        let cents = Double(nanos) / 1e7
        if cents >= 100 { return String(format: "$%.2f", cents / 100) }
        return String(format: "%.2f¢", cents)
    }

    private func fmtTokens(_ n: UInt64) -> String {
        if n >= 1_000_000 { return String(format: "%.1fM", Double(n) / 1_000_000) }
        if n >= 1_000 { return String(format: "%.0fK", Double(n) / 1_000) }
        return "\(n)"
    }

    private var totalSubagentCost: Int64 {
        summary.sessions.reduce(0) { acc, s in
            acc + s.root.children.reduce(0) { $0 + $1.estimatedCostNanos }
        }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            WindowSectionHeader(
                title: "Subagent cost attribution",
                subtitle: "Per-session breakdown of root vs. subagent spend"
            )

            LazyVGrid(
                columns: [
                    GridItem(.flexible()), GridItem(.flexible()),
                    GridItem(.flexible()),
                ],
                spacing: 12
            ) {
                WindowOverviewKpiTile(label: "Sessions", value: "\(summary.sessions.count)")
                WindowOverviewKpiTile(label: "Subagent cost", value: fmtCost(totalSubagentCost))
                if let top = summary.topSubagentRoles.first {
                    WindowOverviewKpiTile(label: "Top role", value: top.role.isEmpty ? "—" : top.role)
                }
            }

            VStack(spacing: 6) {
                ForEach(summary.sessions.prefix(20)) { session in
                    AgentTreeSessionCard(session: session, fmtCost: fmtCost, fmtTokens: fmtTokens)
                }
            }
        }
    }
}

private struct AgentTreeSessionCard: View {
    let session: SessionAgentTree
    let fmtCost: (Int64) -> String
    let fmtTokens: (UInt64) -> String
    @State private var expanded = false

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Button {
                withAnimation(.easeInOut(duration: 0.15)) { expanded.toggle() }
            } label: {
                HStack {
                    Text(session.sessionId.prefix(8))
                        .font(.caption2.monospaced())
                        .foregroundStyle(.secondary)
                    if let proj = session.project {
                        Text(String(proj.suffix(28)))
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                    }
                    Text("\(session.subagentCount) subagent\(session.subagentCount != 1 ? "s" : "")")
                        .font(.caption2.monospaced())
                        .foregroundStyle(.secondary)
                    Spacer()
                    Text(fmtCost(session.totalCostNanos))
                        .font(.caption2.monospaced())
                    Image(systemName: expanded ? "chevron.up" : "chevron.down")
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                }
                .padding(.vertical, 6)
                .padding(.horizontal, 10)
            }
            .buttonStyle(.plain)

            if expanded {
                Divider().opacity(0.4)
                VStack(spacing: 0) {
                    AgentNodeRow(node: session.root, isRoot: true, fmtCost: fmtCost, fmtTokens: fmtTokens)
                    ForEach(session.root.children) { child in
                        AgentNodeRow(node: child, isRoot: false, fmtCost: fmtCost, fmtTokens: fmtTokens)
                    }
                }
                .padding(.horizontal, 10)
                .padding(.bottom, 6)
            }
        }
        .menuCardBackground(opacity: 0.04, cornerRadius: 10)
    }
}

private struct AgentNodeRow: View {
    let node: AgentTreeNode
    let isRoot: Bool
    let fmtCost: (Int64) -> String
    let fmtTokens: (UInt64) -> String

    var label: String {
        if let id = node.agentId {
            if let role = node.role { return "\(role) (\(id.prefix(8)))" }
            return String(id.prefix(12))
        }
        return "root"
    }

    var body: some View {
        HStack {
            HStack(spacing: 4) {
                if !isRoot {
                    Text("└").font(.caption2.monospaced()).foregroundStyle(.tertiary)
                }
                Text(label)
                    .font(.caption2.monospaced())
                    .foregroundStyle(isRoot ? .primary : .secondary)
                if node.role != nil && isRoot {
                    Text("root").font(.system(size: 9).monospaced()).foregroundStyle(.tertiary)
                }
            }
            Spacer()
            Text("\(fmtTokens(node.inputTokens + node.outputTokens)) tok")
                .font(.caption2.monospaced())
                .foregroundStyle(.secondary)
            Text(fmtCost(node.estimatedCostNanos))
                .font(.caption2.monospaced())
                .frame(minWidth: 40, alignment: .trailing)
        }
        .padding(.leading, isRoot ? 0 : 16)
        .padding(.vertical, 3)
    }
}
