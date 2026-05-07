import SwiftUI

struct DashboardFilterBar: View {
    @Bindable var filters: DashboardFiltersModel
    let tab: AppNavigationItem

    private var groups: DashboardFilterGroups {
        filters.activeGroups(for: tab)
    }

    var body: some View {
        if !groups.isEmpty {
            HStack(spacing: 10) {
                if groups.contains(.range) {
                    Picker("Range", selection: self.$filters.range) {
                        ForEach(DashboardRange.allCases, id: \.self) { range in
                            Text(range.shortLabel).tag(range)
                        }
                    }
                    .pickerStyle(.segmented)
                    .fixedSize()
                    .accessibilityLabel("Date range")
                }

                if groups.contains(.bucket) {
                    Picker("Bucket", selection: self.$filters.bucket) {
                        ForEach(DashboardBucket.allCases, id: \.self) { bucket in
                            Text(bucket.label).tag(bucket)
                        }
                    }
                    .pickerStyle(.segmented)
                    .fixedSize()
                    .accessibilityLabel("Grouping bucket")
                }

                if groups.contains(.provider) {
                    Picker("Provider", selection: self.$filters.provider) {
                        ForEach(ProviderScope.allCases, id: \.self) { scope in
                            Text(scope.label).tag(scope)
                        }
                    }
                    .pickerStyle(.segmented)
                    .fixedSize()
                    .accessibilityLabel("Provider scope")
                }

                if groups.contains(.models), !filters.availableModels.isEmpty {
                    ModelsFilterPopover(
                        availableModels: filters.availableModels,
                        selectedModels: self.$filters.selectedModels
                    )
                }

                if groups.contains(.projectSearch) {
                    TextField("Search projects", text: self.$filters.projectSearch)
                        .textFieldStyle(.roundedBorder)
                        .frame(minWidth: 160, maxWidth: 240)
                }

                Spacer(minLength: 0)
            }
            .padding(.horizontal, 20)
            .padding(.vertical, 8)
            .background(.bar)
        }
    }
}

private struct ModelsFilterPopover: View {
    let availableModels: [String]
    @Binding var selectedModels: Set<String>
    @State private var isPresented = false

    var body: some View {
        Button {
            self.isPresented.toggle()
        } label: {
            HStack(spacing: 4) {
                Text(self.buttonLabel)
                    .font(.callout)
                Image(systemName: "chevron.down")
                    .font(.caption2.weight(.semibold))
            }
        }
        .buttonStyle(.bordered)
        .popover(isPresented: self.$isPresented, arrowEdge: .bottom) {
            ModelsPopoverContent(
                availableModels: self.availableModels,
                selectedModels: self.$selectedModels
            )
        }
    }

    private var buttonLabel: String {
        if selectedModels.isEmpty { return "All models" }
        if selectedModels.count == 1 { return selectedModels.first! }
        return "\(selectedModels.count) models"
    }
}

private struct ModelsPopoverContent: View {
    let availableModels: [String]
    @Binding var selectedModels: Set<String>

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack {
                Text("Models")
                    .font(.callout.weight(.semibold))
                Spacer()
                Button("All") { selectedModels = [] }
                    .buttonStyle(.borderless)
                    .font(.caption)
                Button("None") { selectedModels = Set(availableModels) }
                    .buttonStyle(.borderless)
                    .font(.caption)
            }
            .padding(.horizontal, 14)
            .padding(.vertical, 10)

            Divider()

            ScrollView {
                VStack(alignment: .leading, spacing: 2) {
                    ForEach(availableModels, id: \.self) { model in
                        Toggle(model, isOn: Binding(
                            get: { !selectedModels.contains(model) },
                            set: { include in
                                if include { selectedModels.remove(model) }
                                else { selectedModels.insert(model) }
                            }
                        ))
                        .toggleStyle(.checkbox)
                        .font(.callout)
                        .padding(.horizontal, 14)
                        .padding(.vertical, 3)
                    }
                }
                .padding(.vertical, 6)
            }
            .frame(maxHeight: 260)
        }
        .frame(minWidth: 220)
    }
}
