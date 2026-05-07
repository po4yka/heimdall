import HeimdallDomain
import SwiftUI

struct AgentRegistrySheet: View {
    let model: AgentsFeatureModel
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            SheetHeader(title: "Agent registry", onDismiss: { self.dismiss() }) {
                EmptyView()
            }

            Divider()

            if !self.model.hasAgentData {
                ContentUnavailableView(
                    "No agent data",
                    systemImage: "person.3",
                    description: Text("Subagent data appears once Claude Code runs parallel agents in your sessions.")
                )
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ScrollView {
                    VStack(alignment: .leading, spacing: 24) {
                        if let breakdown = self.model.subagentBreakdown {
                            AgentRegistrySummarySection(breakdown: breakdown)
                        }
                        AgentRegistrySessionsSection(sessions: self.model.recentSessions)
                    }
                    .padding(20)
                }
            }
        }
        .frame(minWidth: 480, idealWidth: 560, minHeight: 380, idealHeight: 480)
    }
}

// MARK: - Summary section

private struct AgentRegistrySummarySection: View {
    let breakdown: ProviderSubagentBreakdown

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Overview")
                .font(.subheadline.weight(.semibold))

            LazyVGrid(
                columns: [GridItem(.flexible()), GridItem(.flexible())],
                spacing: 10
            ) {
                AgentRegistryStat(label: "Total turns", value: "\(self.breakdown.totalTurns)")
                AgentRegistryStat(label: "Agent spend",
                                  value: FormatHelpers.formatUSD(self.breakdown.totalCostUSD))
                AgentRegistryStat(label: "Sessions", value: "\(self.breakdown.sessionCount)")
                AgentRegistryStat(label: "Agents", value: "\(self.breakdown.agentCount)")
            }
        }
    }
}

private struct AgentRegistryStat: View {
    let label: String
    let value: String

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(self.value)
                .font(.title3.weight(.semibold).monospacedDigit())
            Text(self.label)
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .padding(10)
        .frame(maxWidth: .infinity, alignment: .leading)
        .menuCardBackground(opacity: 0.04, cornerRadius: 10)
    }
}

// MARK: - Sessions section

private struct AgentRegistrySessionsSection: View {
    let sessions: [ProviderSession]

    private var agentSessions: [ProviderSession] {
        Array(self.sessions.prefix(15))
    }

    var body: some View {
        if !self.agentSessions.isEmpty {
            VStack(alignment: .leading, spacing: 10) {
                Text("Recent sessions")
                    .font(.subheadline.weight(.semibold))

                VStack(spacing: 0) {
                    ForEach(self.agentSessions) { session in
                        AgentRegistrySessionRow(session: session)
                        if session.id != self.agentSessions.last?.id {
                            Divider().opacity(0.5)
                        }
                    }
                }
                .menuCardBackground(opacity: 0.04, cornerRadius: 10)
            }
        }
    }
}

private struct AgentRegistrySessionRow: View {
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
