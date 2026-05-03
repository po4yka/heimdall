import HeimdallDomain
import SwiftUI

struct WindowToolErrorsView: View {
    @State private var model: ToolErrorsFeatureModel
    let onBack: () -> Void

    init(toolName: String, port: Int, onBack: @escaping () -> Void) {
        self._model = State(initialValue: ToolErrorsFeatureModel(toolName: toolName, port: port))
        self.onBack = onBack
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 18) {
            // Header
            HStack(alignment: .firstTextBaseline) {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Tool Errors")
                        .font(.system(size: 24, weight: .semibold))
                    Text(self.model.toolName)
                        .font(.callout)
                        .foregroundStyle(.secondary)
                }
                Spacer(minLength: 12)
                Button {
                    self.onBack()
                } label: {
                    Text("Back")
                }
                .buttonStyle(.plain)
                .font(.callout)
                .foregroundStyle(Color.accentInteractive)
            }

            // Summary count
            if self.model.total > 0 {
                Text("\(self.model.total) error\(self.model.total == 1 ? "" : "s") total")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            // Content
            if self.model.isLoading {
                HStack {
                    Spacer()
                    ProgressView()
                    Spacer()
                }
                .padding(.vertical, 40)
            } else if let errorMessage = self.model.errorMessage {
                Text(errorMessage)
                    .font(.caption)
                    .foregroundStyle(.red)
                    .padding(.vertical, 8)
            } else if self.model.rows.isEmpty {
                Text("No errors found.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .padding(.vertical, 8)
            } else {
                ToolErrorsTable(rows: self.model.rows)
            }
        }
        .onAppear {
            Task { @MainActor in await self.model.load() }
        }
    }
}

private struct ToolErrorsTable: View {
    let rows: [ToolErrorRow]

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            // Column headers
            HStack(alignment: .center, spacing: 8) {
                Text("TIMESTAMP")
                    .frame(width: 140, alignment: .leading)
                Text("PROJECT")
                    .frame(width: 120, alignment: .leading)
                Text("SESSION")
                    .frame(width: 100, alignment: .leading)
                Text("INPUT")
                    .frame(minWidth: 120, maxWidth: .infinity, alignment: .leading)
                Text("ERROR")
                    .frame(minWidth: 160, maxWidth: .infinity, alignment: .leading)
            }
            .font(.system(size: 10, weight: .medium).monospaced())
            .foregroundStyle(.secondary)
            .padding(.horizontal, 10)
            .padding(.vertical, 6)

            Divider()

            ScrollView(.vertical, showsIndicators: true) {
                LazyVStack(alignment: .leading, spacing: 0) {
                    ForEach(self.rows) { row in
                        ToolErrorsRow(row: row)
                        Divider()
                            .opacity(0.5)
                    }
                }
            }
        }
        .background(
            RoundedRectangle(cornerRadius: 8, style: .continuous)
                .fill(Color.primary.opacity(0.03))
        )
        .overlay(
            RoundedRectangle(cornerRadius: 8, style: .continuous)
                .stroke(Color.primary.opacity(0.08), lineWidth: 0.5)
        )
    }
}

private struct ToolErrorsRow: View {
    let row: ToolErrorRow

    var body: some View {
        HStack(alignment: .top, spacing: 8) {
            Text(self.row.timestamp)
                .font(.caption2.monospaced())
                .foregroundStyle(.primary)
                .frame(width: 140, alignment: .leading)
                .lineLimit(1)

            Text(self.row.project)
                .font(.caption)
                .foregroundStyle(.primary)
                .frame(width: 120, alignment: .leading)
                .lineLimit(1)
                .truncationMode(.tail)

            Text(self.shortSessionId)
                .font(.caption2.monospaced())
                .foregroundStyle(.secondary)
                .frame(width: 100, alignment: .leading)
                .lineLimit(1)

            Text(self.truncatedInput)
                .font(.caption2.monospaced())
                .foregroundStyle(.secondary)
                .frame(minWidth: 120, maxWidth: .infinity, alignment: .leading)
                .lineLimit(2)

            Text(self.truncatedError)
                .font(.caption)
                .foregroundStyle(.red.opacity(0.85))
                .frame(minWidth: 160, maxWidth: .infinity, alignment: .leading)
                .lineLimit(3)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
    }

    private var shortSessionId: String {
        let id = self.row.sessionId
        guard id.count > 12 else { return id }
        return "..." + id.suffix(12)
    }

    private var truncatedInput: String {
        guard let input = self.row.toolInput, !input.isEmpty else { return "—" }
        let trimmed = input.trimmingCharacters(in: .whitespacesAndNewlines)
        guard trimmed.count > 80 else { return trimmed }
        return String(trimmed.prefix(80)) + "…"
    }

    private var truncatedError: String {
        guard let error = self.row.errorText, !error.isEmpty else { return "(no message)" }
        let trimmed = error.trimmingCharacters(in: .whitespacesAndNewlines)
        guard trimmed.count > 120 else { return trimmed }
        return String(trimmed.prefix(120)) + "…"
    }
}
