import SwiftUI

struct CommandPaletteView: View {
    @Bindable var model: CommandPaletteModel
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        VStack(spacing: 0) {
            // Search field
            HStack(spacing: 8) {
                Image(systemName: "magnifyingglass")
                    .foregroundStyle(.secondary)
                    .imageScale(.medium)
                TextField("Search commands, tabs, sessions…", text: self.$model.query)
                    .textFieldStyle(.plain)
                    .font(.body)
                if !self.model.query.isEmpty {
                    Button {
                        self.model.query = ""
                    } label: {
                        Image(systemName: "xmark.circle.fill")
                            .foregroundStyle(.secondary)
                    }
                    .buttonStyle(.plain)
                }
                Text("[esc]")
                    .font(.caption.monospaced())
                    .foregroundStyle(Color.primary.opacity(0.3))
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 12)

            Divider()

            // Results list
            if self.model.filteredCommands.isEmpty {
                Text("No results")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 32)
            } else {
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 0, pinnedViews: .sectionHeaders) {
                        ForEach(self.model.groupedCommands, id: \.group) { entry in
                            Section {
                                ForEach(entry.commands) { cmd in
                                    PaletteCommandRow(
                                        cmd: cmd,
                                        isSelected: cmd.id == self.model.selectedCommandID
                                    ) {
                                        cmd.run()
                                        self.dismiss()
                                    }
                                }
                            } header: {
                                Text(entry.group.rawValue.uppercased())
                                    .font(.caption2.weight(.semibold).monospaced())
                                    .foregroundStyle(.secondary)
                                    .padding(.horizontal, 16)
                                    .padding(.top, 10)
                                    .padding(.bottom, 4)
                                    .frame(maxWidth: .infinity, alignment: .leading)
                                    .background(.background)
                            }
                        }
                    }
                }
                .frame(maxHeight: 360)
            }

            Divider()

            // Footer hints
            HStack(spacing: 16) {
                Label("Run", systemImage: "return")
                Label("Close", systemImage: "escape")
            }
            .font(.caption2)
            .foregroundStyle(.secondary)
            .padding(.horizontal, 16)
            .padding(.vertical, 8)
        }
        .frame(width: 560)
        .background(.regularMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
        .shadow(color: .black.opacity(0.2), radius: 24, x: 0, y: 8)
        .onKeyPress(.escape) {
            self.dismiss()
            return .handled
        }
    }
}

private struct PaletteCommandRow: View {
    let cmd: PaletteCommand
    let isSelected: Bool
    let onRun: () -> Void

    var body: some View {
        Button(action: self.onRun) {
            HStack(spacing: 10) {
                VStack(alignment: .leading, spacing: 2) {
                    Text(self.cmd.label)
                        .font(.body)
                        .lineLimit(1)
                    if let hint = self.cmd.hint {
                        Text(hint)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                    }
                }
                Spacer()
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 7)
            .background(self.isSelected ? Color.accentColor.opacity(0.12) : Color.clear)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .onHover { hovering in
            if hovering {
                self.cmd.id.hashValue  // suppress warning; selection handled by hover
            }
        }
    }
}
