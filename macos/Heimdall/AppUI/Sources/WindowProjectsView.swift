import HeimdallDomain
import SwiftUI

struct WindowProjectsView: View {
    @Bindable var model: ProjectsFeatureModel

    var body: some View {
        VStack(alignment: .leading, spacing: 24) {
            WindowHeader(
                title: "Projects",
                subtitle: "Spend and activity by project",
                issue: WindowHeaderIssuePresentation.make(message: self.model.globalIssueLabel),
                onRetry: { Task { await self.model.refreshAll() } },
                isRetrying: self.model.isRefreshing
            ) {
                EmptyView()
            }

            ProjectsKpiRow(model: self.model)

            ProjectsRegistrySection(model: self.model)
        }
        .task { await self.model.refreshAll() }
    }
}

// MARK: - KPI row

private struct ProjectsKpiRow: View {
    let model: ProjectsFeatureModel

    var body: some View {
        LazyVGrid(
            columns: [
                GridItem(.flexible()), GridItem(.flexible()),
                GridItem(.flexible()), GridItem(.flexible())
            ],
            spacing: 12
        ) {
            WindowOverviewKpiTile(label: "Total spend", value: FormatHelpers.formatUSD(self.model.totalCostUSD))
            WindowOverviewKpiTile(label: "Projects", value: "\(self.model.byProject.count)")
            WindowOverviewKpiTile(label: "Sessions", value: "\(self.model.totalSessions)")
            WindowOverviewKpiTile(label: "Turns", value: "\(self.model.totalTurns)")
        }
    }
}

// MARK: - Project registry

private struct ProjectsRegistrySection: View {
    @Bindable var model: ProjectsFeatureModel

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            HStack {
                WindowSectionHeader(
                    title: "Project registry",
                    subtitle: "All projects sorted by \(self.model.sortOrder.rawValue.lowercased())"
                )
                Spacer()
                Picker("Sort by", selection: self.$model.sortOrder) {
                    ForEach(ProjectSortOrder.allCases, id: \.self) { order in
                        Text(order.rawValue).tag(order)
                    }
                }
                .pickerStyle(.segmented)
                .fixedSize()
                .labelsHidden()
            }

            if self.model.byProject.isEmpty {
                ContentUnavailableView(
                    "No project data",
                    systemImage: "folder",
                    description: Text("Projects appear once sessions are associated with a working directory.")
                )
                .frame(maxWidth: .infinity)
                .padding(.vertical, 32)
            } else {
                ProjectRegistryTable(rows: self.model.byProject)
                    .padding(18)
                    .menuCardBackground(opacity: 0.04, cornerRadius: 16)
            }
        }
    }
}

// MARK: - Registry table

private struct ProjectRegistryTable: View {
    let rows: [ProviderProjectRow]

    private static let maxCost: Double = 1000

    var body: some View {
        let maxCost = self.rows.map(\.costUSD).max() ?? 1
        VStack(spacing: 0) {
            ProjectRegistryHeader()
            Divider().padding(.vertical, 6)
            ForEach(self.rows) { row in
                ProjectRegistryRow(row: row, maxCost: maxCost)
                if row.id != self.rows.last?.id {
                    Divider().opacity(0.4)
                }
            }
        }
    }
}

private struct ProjectRegistryHeader: View {
    var body: some View {
        HStack(spacing: 0) {
            Text("PROJECT")
                .frame(maxWidth: .infinity, alignment: .leading)
            Text("COST")
                .frame(width: 72, alignment: .trailing)
            Text("SESSIONS")
                .frame(width: 72, alignment: .trailing)
            Text("TURNS")
                .frame(width: 64, alignment: .trailing)
        }
        .font(.caption2.weight(.semibold).monospaced())
        .foregroundStyle(.secondary)
    }
}

private struct ProjectRegistryRow: View {
    let row: ProviderProjectRow
    let maxCost: Double

    var body: some View {
        VStack(alignment: .leading, spacing: 3) {
            HStack(spacing: 0) {
                Text(self.row.displayName)
                    .font(.caption.weight(.medium))
                    .lineLimit(1)
                    .truncationMode(.middle)
                    .frame(maxWidth: .infinity, alignment: .leading)
                Text(FormatHelpers.formatUSD(self.row.costUSD))
                    .font(.caption.monospacedDigit())
                    .frame(width: 72, alignment: .trailing)
                Text("\(self.row.sessions)")
                    .font(.caption.monospacedDigit())
                    .frame(width: 72, alignment: .trailing)
                Text("\(self.row.turns)")
                    .font(.caption.monospacedDigit())
                    .frame(width: 64, alignment: .trailing)
            }
            GeometryReader { geo in
                ZStack(alignment: .leading) {
                    Rectangle()
                        .fill(Color.primary.opacity(0.08))
                        .frame(height: 2)
                    Rectangle()
                        .fill(Color.primary.opacity(0.45))
                        .frame(
                            width: max(2, geo.size.width * CGFloat(self.row.costUSD / max(self.maxCost, 1e-9))),
                            height: 2
                        )
                }
                .clipShape(RoundedRectangle(cornerRadius: 1, style: .continuous))
            }
            .frame(height: 2)
        }
        .padding(.vertical, 5)
    }
}
