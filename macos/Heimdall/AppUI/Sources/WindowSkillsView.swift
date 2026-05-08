import SwiftUI

struct WindowSkillsView: View {
    @Bindable var model: SkillsFeatureModel

    var body: some View {
        VStack(alignment: .leading, spacing: 24) {
            WindowHeader(
                title: "Skills",
                subtitle: "Disk and context-budget impact of installed skills",
                onRetry: { Task { await self.model.reload() } },
                isRetrying: self.model.isLoading
            ) {
                EmptyView()
            }

            if let error = self.model.errorMessage {
                SkillsErrorView(message: error)
            } else if let report = self.model.report {
                SkillsSummaryRow(totals: report.totals)
                SkillsBudgetSection(rows: report.budget)
                SkillsScopesSection(scopes: report.scopes)
                SkillsFooterView(report: report)
            } else if self.model.isLoading {
                ProgressView("Scanning skills…")
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 32)
            }
        }
        .task { await self.model.reload() }
    }
}

// MARK: - Error

private struct SkillsErrorView: View {
    let message: String

    var body: some View {
        HStack(spacing: 8) {
            Image(systemName: "exclamationmark.triangle")
                .foregroundStyle(.red)
            Text(message)
                .font(.callout)
                .foregroundStyle(.secondary)
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .leading)
        .menuCardBackground(opacity: 0.04, cornerRadius: 12)
    }
}

// MARK: - Summary KPI row

private struct SkillsSummaryRow: View {
    let totals: SkillsTotals

    var body: some View {
        LazyVGrid(
            columns: [
                GridItem(.flexible()), GridItem(.flexible()),
                GridItem(.flexible()), GridItem(.flexible()),
                GridItem(.flexible()), GridItem(.flexible()),
            ],
            spacing: 12
        ) {
            WindowOverviewKpiTile(label: "Skills", value: "\(totals.skillsCount)")
            WindowOverviewKpiTile(label: "Total disk", value: formatBytes(totals.totalBytes))
            WindowOverviewKpiTile(label: "Listing tokens", value: "\(totals.totalListingTokens)")
            WindowOverviewKpiTile(label: "Claude disk", value: formatBytes(totals.claudeBytes))
            WindowOverviewKpiTile(label: "Codex disk", value: formatBytes(totals.codexBytes))
            WindowOverviewKpiTile(label: "Projects", value: "\(totals.projectCount)")
        }
    }
}

// MARK: - Budget progress bars

private struct SkillsBudgetSection: View {
    let rows: [SkillsBudgetRow]

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            WindowSectionHeader(
                title: "Context budget",
                subtitle: "Estimated listing token usage vs. available budget (skillListingBudgetFraction)"
            )
            VStack(spacing: 10) {
                ForEach(rows) { row in
                    BudgetProgressRow(row: row)
                }
            }
            .padding(18)
            .menuCardBackground(opacity: 0.04, cornerRadius: 16)
        }
    }
}

private struct BudgetProgressRow: View {
    let row: SkillsBudgetRow

    private var fill: Double { row.fillFraction }
    private var barColor: Color {
        if row.isOver { return .red }
        if fill > 0.8 { return Color.primary.opacity(0.80) }
        return Color.primary.opacity(0.55)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(row.modelLabel)
                    .font(.caption2.monospaced())
                    .foregroundStyle(.secondary)
                Spacer()
                if row.isOver {
                    Text("OVER: \(abs(row.headroomTokens)) tok over, ~\(row.simulatedDropCount) skills dropped")
                        .font(.caption2.monospaced())
                        .foregroundStyle(.red)
                } else {
                    Text("\(row.usedTokens) / \(row.budgetTokens) tok")
                        .font(.caption2.monospaced())
                        .foregroundStyle(.secondary)
                }
            }
            GeometryReader { geo in
                ZStack(alignment: .leading) {
                    Rectangle()
                        .fill(Color.primary.opacity(0.10))
                        .frame(height: 4)
                    Rectangle()
                        .fill(self.barColor)
                        .frame(width: max(0, geo.size.width * CGFloat(self.fill)), height: 4)
                }
                .clipShape(RoundedRectangle(cornerRadius: 2, style: .continuous))
            }
            .frame(height: 4)
        }
    }
}

// MARK: - Per-scope expandable sections

private struct SkillsScopesSection: View {
    let scopes: [SkillScopeResponse]

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            WindowSectionHeader(
                title: "Scopes",
                subtitle: "Per-provider, per-scope breakdown"
            )
            VStack(spacing: 8) {
                ForEach(scopes) { scope in
                    SkillScopeRow(scope: scope)
                }
            }
        }
    }
}

private struct SkillScopeRow: View {
    let scope: SkillScopeResponse
    @State private var expanded = false

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Button {
                withAnimation(.easeInOut(duration: 0.15)) {
                    self.expanded.toggle()
                }
            } label: {
                HStack {
                    Text(scope.displayLabel)
                        .font(.caption2.monospaced())
                        .foregroundStyle(.secondary)
                    Spacer()
                    Text("\(scope.skills.count) skills · \(formatBytes(scope.bytes)) · \(scope.listingTokens) tok")
                        .font(.caption2.monospaced())
                        .foregroundStyle(.secondary)
                    Image(systemName: self.expanded ? "chevron.up" : "chevron.down")
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                }
                .padding(.vertical, 8)
                .padding(.horizontal, 12)
            }
            .buttonStyle(.plain)

            if self.expanded && !scope.skills.isEmpty {
                Divider().opacity(0.4)
                SkillsTable(skills: scope.skills)
                    .padding(.horizontal, 12)
                    .padding(.bottom, 10)
            }
        }
        .menuCardBackground(opacity: 0.04, cornerRadius: 12)
    }
}

private struct SkillsTable: View {
    let skills: [SkillRow]

    var body: some View {
        VStack(spacing: 0) {
            SkillsTableHeader()
            Divider().opacity(0.4).padding(.vertical, 4)
            ForEach(skills) { skill in
                SkillsTableRow(skill: skill)
                if skill.id != skills.last?.id {
                    Divider().opacity(0.25)
                }
            }
        }
        .padding(.top, 6)
    }
}

private struct SkillsTableHeader: View {
    var body: some View {
        HStack(spacing: 0) {
            Text("NAME").frame(maxWidth: .infinity, alignment: .leading)
            Text("DISK").frame(width: 64, alignment: .trailing)
            Text("TOK").frame(width: 48, alignment: .trailing)
            Text("STATUS").frame(width: 56, alignment: .trailing)
        }
        .font(.caption2.weight(.semibold).monospaced())
        .foregroundStyle(.secondary)
        .textCase(.uppercase)
    }
}

private struct SkillsTableRow: View {
    let skill: SkillRow

    var statusColor: Color {
        skill.frontmatterStatus == "ok" ? .secondary : .red
    }

    var body: some View {
        HStack(spacing: 0) {
            HStack(spacing: 4) {
                Text(skill.name)
                    .lineLimit(1)
                    .truncationMode(.tail)
                if skill.isSymlink {
                    Text("[link]").foregroundStyle(.tertiary)
                }
                if skill.descriptionTruncated {
                    Text("+").foregroundStyle(.tertiary)
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            Text(formatBytes(skill.bytes)).frame(width: 64, alignment: .trailing)
            Text("\(skill.listingTokens)").frame(width: 48, alignment: .trailing)
            Text(skill.frontmatterStatus)
                .foregroundStyle(self.statusColor)
                .frame(width: 56, alignment: .trailing)
        }
        .font(.caption2.monospaced())
        .padding(.vertical, 3)
    }
}

// MARK: - Footer

private struct SkillsFooterView: View {
    let report: SkillsReport

    var body: some View {
        Text("tokenizer: \(report.tokenizer) · budget fraction: \(String(format: "%.1f", report.budgetFraction * 100))% · generated: \(report.generatedAt)")
            .font(.caption2.monospaced())
            .foregroundStyle(.tertiary)
            .frame(maxWidth: .infinity, alignment: .leading)
    }
}

// MARK: - Helpers

private func formatBytes(_ bytes: UInt64) -> String {
    if bytes >= 1_048_576 {
        return String(format: "%.1f MB", Double(bytes) / 1_048_576)
    } else if bytes >= 1_024 {
        return String(format: "%.1f KB", Double(bytes) / 1_024)
    }
    return "\(bytes) B"
}
