import SwiftUI

struct SavedViewsBar: View {
    @Bindable var model: SavedViewsModel
    @Bindable var filters: DashboardFiltersModel
    @State private var isSavingNew = false
    @State private var newViewName = ""

    var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 6) {
                ForEach(self.model.views) { view in
                    SavedViewChip(
                        label: view.name,
                        isActive: self.model.activeViewID == view.id,
                        isDeletable: !view.isBuiltIn,
                        onTap: { self.model.activate(view, applying: self.filters) },
                        onDelete: { self.model.delete(view) }
                    )
                }

                if self.isSavingNew {
                    HStack(spacing: 4) {
                        TextField("View name", text: self.$newViewName)
                            .textFieldStyle(.roundedBorder)
                            .frame(width: 120)
                            .onSubmit { self.commitSave() }
                        Button("Save", action: self.commitSave)
                            .buttonStyle(.borderedProminent)
                            .controlSize(.small)
                            .disabled(self.newViewName.trimmingCharacters(in: .whitespaces).isEmpty)
                        Button("Cancel") {
                            self.isSavingNew = false
                            self.newViewName = ""
                        }
                        .buttonStyle(.borderless)
                        .controlSize(.small)
                    }
                    .padding(.vertical, 2)
                } else {
                    Button {
                        self.isSavingNew = true
                    } label: {
                        Label("Save view", systemImage: "plus")
                            .font(.caption.weight(.medium))
                    }
                    .buttonStyle(.borderless)
                    .foregroundStyle(.secondary)
                }
            }
            .padding(.horizontal, 20)
            .padding(.vertical, 6)
        }
        .background(.bar)
    }

    private func commitSave() {
        let name = newViewName.trimmingCharacters(in: .whitespaces)
        guard !name.isEmpty else { return }
        model.save(name: name, snapshot: filters.snapshot())
        isSavingNew = false
        newViewName = ""
    }
}

private struct SavedViewChip: View {
    let label: String
    let isActive: Bool
    let isDeletable: Bool
    let onTap: () -> Void
    let onDelete: () -> Void

    var body: some View {
        HStack(spacing: 4) {
            Button(action: self.onTap) {
                Text(self.label)
                    .font(.caption.weight(.medium))
                    .foregroundStyle(self.isActive ? .primary : .secondary)
            }
            .buttonStyle(.plain)

            if self.isDeletable {
                Button(action: self.onDelete) {
                    Image(systemName: "xmark")
                        .font(.caption2.weight(.semibold))
                        .foregroundStyle(.tertiary)
                }
                .buttonStyle(.plain)
                .accessibilityLabel("Delete \(self.label)")
            }
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 4)
        .background(
            Capsule()
                .fill(self.isActive
                    ? Color.accentInteractive.opacity(0.15)
                    : Color.primary.opacity(0.06))
        )
        .overlay(
            Capsule()
                .strokeBorder(
                    self.isActive ? Color.accentInteractive.opacity(0.4) : Color.clear,
                    lineWidth: 1
                )
        )
    }
}
