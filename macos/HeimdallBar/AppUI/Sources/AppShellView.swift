import AppKit
import HeimdallDomain
import SwiftUI

struct AppShellView: View {
    @Bindable var shell: AppShellModel
    @Bindable var overview: OverviewFeatureModel
    @Bindable var settings: SettingsFeatureModel
    let providerModel: (ProviderID) -> ProviderFeatureModel

    var body: some View {
        NavigationSplitView {
            List(selection: self.selectionBinding) {
                ForEach(self.shell.navigationItems, id: \.id) { item in
                    Label(item.title, systemImage: item.systemImage)
                        .tag(Optional(item))
                }
            }
            .listStyle(.sidebar)
            .navigationTitle("HeimdallBar")
        } detail: {
            ScrollView {
                VStack(alignment: .leading, spacing: 18) {
                    switch self.shell.navigationSelection {
                    case .overview:
                        WindowOverviewView(
                            overview: self.overview,
                            shell: self.shell,
                            providerModel: self.providerModel
                        )
                    case .provider(let provider):
                        WindowProviderView(model: self.providerModel(provider))
                    case .settings:
                        WindowSettingsView(
                            settings: self.settings,
                            providerModel: self.providerModel
                        )
                    }
                }
                .padding(24)
                .frame(maxWidth: .infinity, alignment: .leading)
            }
            .background(Color(nsColor: .windowBackgroundColor))
        }
        .toolbar {
            ToolbarItemGroup {
                Button {
                    Task {
                        switch self.shell.navigationSelection {
                        case .overview, .settings:
                            await self.overview.refreshAll()
                        case .provider(let provider):
                            await self.providerModel(provider).refresh()
                        }
                    }
                } label: {
                    Label("Refresh", systemImage: "arrow.clockwise")
                }
                .disabled(self.isBusy)

                Button {
                    if let url = URL(string: "http://127.0.0.1:\(self.settings.config.helperPort)") {
                        NSWorkspace.shared.open(url)
                    }
                } label: {
                    Label("Open Dashboard", systemImage: "safari")
                }
            }
        }
        .task {
            await self.settings.refreshBrowserImports()
        }
    }

    private var selectionBinding: Binding<AppNavigationItem?> {
        Binding(
            get: { self.shell.navigationSelection },
            set: { selection in
                guard let selection else { return }
                self.shell.selectNavigation(selection)
            }
        )
    }

    private var isBusy: Bool {
        switch self.shell.navigationSelection {
        case .overview, .settings:
            return self.overview.projection.isRefreshing
        case .provider(let provider):
            return self.providerModel(provider).isBusy
        }
    }
}

private struct WindowOverviewView: View {
    @Bindable var overview: OverviewFeatureModel
    @Bindable var shell: AppShellModel
    let providerModel: (ProviderID) -> ProviderFeatureModel

    var body: some View {
        let projection = self.overview.projection

        VStack(alignment: .leading, spacing: 18) {
            WindowHeader(
                title: "Overview",
                subtitle: projection.refreshedAtLabel,
                issue: projection.globalIssueLabel
            )

            OverviewMenuCard(providerModel: self.providerModel, projection: projection)

            LazyVGrid(columns: [GridItem(.adaptive(minimum: 280), spacing: 12)], spacing: 12) {
                ForEach(self.shell.visibleProviders, id: \.self) { provider in
                    let providerModel = self.providerModel(provider)
                    Button {
                        self.shell.selectNavigation(.provider(provider))
                    } label: {
                        OverviewProviderCard(model: providerModel, item: providerModel.projection)
                    }
                    .buttonStyle(.plain)
                }
            }
        }
    }
}

private struct WindowProviderView: View {
    @Bindable var model: ProviderFeatureModel

    var body: some View {
        VStack(alignment: .leading, spacing: 18) {
            WindowHeader(
                title: self.model.provider.title,
                subtitle: self.model.projection.refreshStatusLabel,
                issue: self.model.issue?.message ?? self.model.projection.globalIssueLabel
            )

            ProviderMenuCard(providerModel: self.model)

            ProviderSessionDetails(model: self.model)
        }
    }
}

private struct WindowSettingsView: View {
    @Bindable var settings: SettingsFeatureModel
    let providerModel: (ProviderID) -> ProviderFeatureModel

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            WindowHeader(
                title: "Settings",
                subtitle: "Provider configuration, auth diagnostics, and web session imports",
                issue: self.settings.issue?.message
            )
            SettingsView(
                model: self.settings,
                providerModel: self.providerModel
            )
            .frame(maxWidth: 760, alignment: .leading)
        }
    }
}

private struct WindowHeader: View {
    let title: String
    let subtitle: String
    let issue: String?

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(self.title)
                .font(.system(size: 24, weight: .semibold))
            Text(self.subtitle)
                .font(.callout)
                .foregroundStyle(.secondary)
            if let issue, !issue.isEmpty {
                Text(issue)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .padding(.horizontal, 10)
                    .padding(.vertical, 8)
                    .background(
                        RoundedRectangle(cornerRadius: 10, style: .continuous)
                            .fill(Color.primary.opacity(0.05))
                    )
            }
        }
    }
}

private struct ProviderSessionDetails: View {
    @Bindable var model: ProviderFeatureModel

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Web Session")
                .font(.headline)
            if let session = self.model.importedSession {
                Text(session.expired ? "Stored session is expired." : (session.loginRequired ? "Stored session is missing an active auth cookie." : "Stored session is ready."))
                Text("Source: \(session.browserSource.title) · \(session.profileName)")
                    .foregroundStyle(.secondary)
                Button("Reset Session") {
                    Task { await self.model.resetBrowserSession() }
                }
                .disabled(self.model.isImportingSession)
            } else {
                Text("No imported browser session stored.")
                    .foregroundStyle(.secondary)
            }

            ForEach(self.model.importCandidates) { candidate in
                Button("Import from \(candidate.title)") {
                    Task { await self.model.importBrowserSession(candidate: candidate) }
                }
                .disabled(self.model.isImportingSession)
            }
        }
        .padding(14)
        .background(RoundedRectangle(cornerRadius: 14).fill(Color.primary.opacity(0.03)))
    }
}
