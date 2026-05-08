import SwiftUI

struct WindowMcpServersView: View {
    @Bindable var model: McpServersFeatureModel

    var body: some View {
        VStack(alignment: .leading, spacing: 24) {
            WindowHeader(
                title: "MCP servers",
                subtitle: "Configured servers, transports, runtime state",
                onRetry: { Task { await self.model.reload() } },
                isRetrying: self.model.isLoading
            ) {
                EmptyView()
            }

            if let error = self.model.errorMessage {
                McpServersErrorView(message: error)
            } else if let report = self.model.report {
                McpServersSummaryRow(totals: report.totals)
                McpProvidersSection(report: report)
                McpServersFooterView(report: report)
            } else if self.model.isLoading {
                ProgressView("Loading MCP servers\u{2026}")
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 32)
            }
        }
        .task { await self.model.reload() }
    }
}

// MARK: - Error

private struct McpServersErrorView: View {
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

private struct McpServersSummaryRow: View {
    let totals: McpServerTotals

    var body: some View {
        LazyVGrid(
            columns: [
                GridItem(.flexible()), GridItem(.flexible()),
                GridItem(.flexible()), GridItem(.flexible()),
                GridItem(.flexible()), GridItem(.flexible()),
            ],
            spacing: 12
        ) {
            WindowOverviewKpiTile(label: "Configured", value: "\(totals.configuredCount)")
            WindowOverviewKpiTile(label: "Running", value: "\(totals.runningCount)")
            WindowOverviewKpiTile(label: "Never invoked", value: "\(totals.neverInvokedCount)")
            WindowOverviewKpiTile(label: "Claude", value: "\(totals.claudeCount)")
            WindowOverviewKpiTile(label: "Codex", value: "\(totals.codexCount)")
            WindowOverviewKpiTile(label: "Projects", value: "\(totals.projectCount)")
            if totals.dormantCount > 0 {
                WindowOverviewKpiTile(label: "Dormant", value: "\(totals.dormantCount)")
            }
        }
    }
}

// MARK: - Providers section

private struct McpProvidersSection: View {
    let report: McpServerReport

    var body: some View {
        HStack(alignment: .top, spacing: 16) {
            McpProviderColumn(title: "Claude Code", entries: report.claude)
            McpProviderColumn(title: "Codex", entries: report.codex)
        }
    }
}

private struct McpProviderColumn: View {
    let title: String
    let entries: [McpServerEntry]

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            WindowSectionHeader(title: title, subtitle: "\(entries.count) servers")
            if entries.isEmpty {
                Text("None configured")
                    .font(.caption2.monospaced())
                    .foregroundStyle(.secondary)
            } else {
                VStack(spacing: 8) {
                    ForEach(entries) { entry in
                        McpServerCard(entry: entry)
                    }
                }
            }
        }
        .frame(maxWidth: .infinity, alignment: .topLeading)
    }
}

// MARK: - Server card

private struct McpServerCard: View {
    let entry: McpServerEntry
    @State private var expanded = false

    var runtimeLabel: String {
        switch entry.runtime {
        case .running(let pid, _, _): return "[RUNNING pid:\(pid)]"
        case .notRunning: return "[STOPPED]"
        case .notApplicable: return "[N/A]"
        }
    }

    var runtimeColor: Color {
        switch entry.runtime {
        case .running: return Color.green.opacity(0.8)
        case .notRunning: return Color.secondary
        case .notApplicable: return Color.secondary.opacity(0.6)
        }
    }

    var transportLabel: String {
        switch entry.transport {
        case .stdio: return "stdio"
        case .http: return "http"
        case .sse: return "sse"
        }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Button {
                withAnimation(.easeInOut(duration: 0.15)) { expanded.toggle() }
            } label: {
                HStack {
                    Text(entry.name)
                        .font(.caption.weight(.medium))
                        .lineLimit(1)
                    Text(transportLabel)
                        .font(.caption2.monospaced())
                        .foregroundStyle(.secondary)
                        .padding(.horizontal, 4)
                        .background(Color.primary.opacity(0.08), in: RoundedRectangle(cornerRadius: 3))
                    if let managedBy = entry.managedBy {
                        Text("[\(managedBy)]")
                            .font(.caption2.monospaced())
                            .foregroundStyle(.tertiary)
                    }
                    Spacer()
                    if entry.isDormant {
                        let dormantBadge: String = {
                            if let lastUsed = entry.usage?.lastUsed,
                               let date = ISO8601DateFormatter().date(from: lastUsed) {
                                let days = Int(Date().timeIntervalSince(date) / 86400)
                                return "[DORMANT \(days)d]"
                            }
                            return "[NEVER]"
                        }()
                        Text(dormantBadge)
                            .font(.caption2.monospaced())
                            .foregroundStyle(.secondary)
                    }
                    Text(runtimeLabel)
                        .font(.caption2.monospaced())
                        .foregroundStyle(runtimeColor)
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
                McpServerDetailView(entry: entry)
                    .padding(.horizontal, 10)
                    .padding(.vertical, 8)
            }
        }
        .menuCardBackground(opacity: 0.04, cornerRadius: 10)
    }
}

// MARK: - Server detail

private struct McpServerDetailView: View {
    let entry: McpServerEntry

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(entry.sourcePath)
                .font(.caption2.monospaced())
                .foregroundStyle(.tertiary)
                .lineLimit(1)
                .truncationMode(.middle)

            switch entry.transport {
            case .stdio(let command, let args):
                Text(([command] + args).joined(separator: " "))
                    .font(.caption2.monospaced())
                    .foregroundStyle(.secondary)
                    .lineLimit(2)
            case .http(let url), .sse(let url):
                Text(url)
                    .font(.caption2.monospaced())
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }

            if !entry.env.isEmpty {
                VStack(spacing: 2) {
                    ForEach(Array(entry.env.keys.sorted()), id: \.self) { key in
                        McpEnvRow(key: key, value: entry.env[key]!)
                    }
                }
            }

            if let log = entry.logProbe {
                Text("logs: \(log.path) \u{00B7} \(formatBytes(log.bytes))")
                    .font(.caption2.monospaced())
                    .foregroundStyle(.tertiary)
                    .lineLimit(1)
                    .truncationMode(.middle)
            }

            if let u = entry.usage {
                Text("\(u.totalCalls) calls \u{00B7} \(u.distinctSessions) sessions \u{00B7} \(u.distinctTools) distinct tools")
                    .font(.caption2.monospaced())
                    .foregroundStyle(.secondary)
            } else {
                Text("never invoked")
                    .font(.caption2.monospaced())
                    .foregroundStyle(.tertiary)
            }
        }
    }
}

private struct McpEnvRow: View {
    let key: String
    let value: McpRedactedValue

    var isSecret: Bool {
        if case .secret = value { return true }
        return false
    }

    var displayValue: String {
        switch value {
        case .plain(let v): return v
        case .secret(let m): return m
        case .envFromFile(let path, _, _): return "(file: \(path))"
        }
    }

    var body: some View {
        HStack {
            Text(key)
                .font(.caption2.monospaced())
                .foregroundStyle(.secondary)
                .lineLimit(1)
            Spacer()
            if isSecret {
                Text("[SECRET]")
                    .font(.caption2.monospaced())
                    .foregroundStyle(.red)
                    .padding(.trailing, 4)
            }
            Text(displayValue)
                .font(.caption2.monospaced())
                .foregroundStyle(.tertiary)
                .lineLimit(1)
        }
    }
}

// MARK: - Footer

private struct McpServersFooterView: View {
    let report: McpServerReport

    var body: some View {
        Text("generated: \(report.generatedAt)")
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
