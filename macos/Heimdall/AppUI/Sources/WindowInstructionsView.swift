import SwiftUI

struct WindowInstructionsView: View {
    @Bindable var model: InstructionsFeatureModel

    var body: some View {
        VStack(alignment: .leading, spacing: 24) {
            WindowHeader(
                title: "Instructions",
                subtitle: "Disk and context-budget impact of CLAUDE.md / AGENTS.md",
                onRetry: { Task { await self.model.reload() } },
                isRetrying: self.model.isLoading
            ) {
                EmptyView()
            }

            if let error = self.model.errorMessage {
                InstructionsErrorView(message: error)
            } else if let report = self.model.report {
                InstructionsSummaryRow(totals: report.totals)
                InstructionsBudgetSection(rows: report.budget, budgetFraction: report.budgetFraction)
                InstructionsScopesSection(scopes: report.scopes)
                InstructionsFooterView(report: report)
            } else if self.model.isLoading {
                ProgressView("Scanning instruction files…")
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 32)
            }
        }
        .task { await self.model.reload() }
    }
}

// MARK: - Error

private struct InstructionsErrorView: View {
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

private struct InstructionsSummaryRow: View {
    let totals: InstructionTotals

    var body: some View {
        LazyVGrid(
            columns: [
                GridItem(.flexible()), GridItem(.flexible()),
                GridItem(.flexible()), GridItem(.flexible()),
                GridItem(.flexible()), GridItem(.flexible()),
            ],
            spacing: 12
        ) {
            WindowOverviewKpiTile(label: "Files", value: "\(totals.fileCount)")
            WindowOverviewKpiTile(label: "Total disk", value: formatBytes(totals.totalBytes))
            WindowOverviewKpiTile(label: "Total tokens", value: "\(totals.totalTokens)")
            WindowOverviewKpiTile(label: "Claude disk", value: formatBytes(totals.claudeBytes))
            WindowOverviewKpiTile(label: "Codex disk", value: formatBytes(totals.codexBytes))
            WindowOverviewKpiTile(label: "Projects", value: "\(totals.projectCount)")
        }
    }
}

// MARK: - Budget progress bars

private struct InstructionsBudgetSection: View {
    let rows: [SkillsBudgetRow]
    let budgetFraction: Double

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            WindowSectionHeader(
                title: "Context budget",
                subtitle: "Estimated instruction-file token usage vs. available budget (5% of context)"
            )
            VStack(spacing: 10) {
                ForEach(rows) { row in
                    InstructionsBudgetProgressRow(row: row)
                }
            }
            .padding(18)
            .menuCardBackground(opacity: 0.04, cornerRadius: 16)
        }
    }
}

private struct InstructionsBudgetProgressRow: View {
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
                    Text("OVER: \(abs(row.headroomTokens)) tok over, ~\(row.simulatedDropCount) files dropped")
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

private struct InstructionsScopesSection: View {
    let scopes: [InstructionScope]

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            WindowSectionHeader(
                title: "Scopes",
                subtitle: "Per-provider, per-scope breakdown"
            )
            VStack(spacing: 8) {
                ForEach(scopes) { scope in
                    InstructionScopeRow(scope: scope)
                }
            }
        }
    }
}

private struct InstructionScopeRow: View {
    let scope: InstructionScope
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
                    Text("\(scope.files.count) files · \(formatBytes(scope.bytes)) · \(scope.tokens) tok")
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

            if self.expanded && !scope.files.isEmpty {
                Divider().opacity(0.4)
                InstructionsTable(files: scope.files)
                    .padding(.horizontal, 12)
                    .padding(.bottom, 10)
            }
        }
        .menuCardBackground(opacity: 0.04, cornerRadius: 12)
    }
}

private struct InstructionsTable: View {
    let files: [InstructionFile]

    var body: some View {
        VStack(spacing: 0) {
            InstructionsTableHeader()
            Divider().opacity(0.4).padding(.vertical, 4)
            ForEach(files) { file in
                InstructionsTableRow(file: file)
                if file.id != files.last?.id {
                    Divider().opacity(0.25)
                }
            }
        }
        .padding(.top, 6)
    }
}

private struct InstructionsTableHeader: View {
    var body: some View {
        HStack(spacing: 0) {
            Text("PATH").frame(maxWidth: .infinity, alignment: .leading)
            Text("BYTES").frame(width: 72, alignment: .trailing)
            Text("TOK").frame(width: 48, alignment: .trailing)
            Text("LINES").frame(width: 48, alignment: .trailing)
            Text("STATUS").frame(width: 72, alignment: .trailing)
        }
        .font(.caption2.weight(.semibold).monospaced())
        .foregroundStyle(.secondary)
        .textCase(.uppercase)
    }
}

private struct InstructionsTableRow: View {
    let file: InstructionFile

    private var statusColor: Color {
        file.frontmatterStatus == .invalid ? .red : .secondary
    }

    private var statusText: String {
        switch file.frontmatterStatus {
        case .ok: return "ok"
        case .invalid: return "invalid"
        case .notApplicable: return "n/a"
        }
    }

    var body: some View {
        HStack(spacing: 0) {
            HStack(spacing: 4) {
                Text((file.path as NSString).lastPathComponent)
                    .lineLimit(1)
                    .truncationMode(.middle)
                if file.isSymlink {
                    Text("[link]").foregroundStyle(.tertiary)
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            Text(formatBytes(file.bytes)).frame(width: 72, alignment: .trailing)
            Text("\(file.tokens)").frame(width: 48, alignment: .trailing)
            Text("\(file.lineCount)").frame(width: 48, alignment: .trailing)
            Text(statusText)
                .foregroundStyle(statusColor)
                .frame(width: 72, alignment: .trailing)
        }
        .font(.caption2.monospaced())
        .padding(.vertical, 3)
    }
}

// MARK: - Footer

private struct InstructionsFooterView: View {
    let report: InstructionFilesReport

    var body: some View {
        Text("tokenizer: \(report.tokenizer) · budget fraction: \(String(format: "%.0f", report.budgetFraction * 100))% · generated: \(report.generatedAt)")
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
