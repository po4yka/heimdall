import AppKit
import HeimdallDomain
import SwiftUI

struct AppShellView: View {
    @Bindable var shell: AppShellModel
    @Bindable var overview: OverviewFeatureModel
    let helperPort: Int
    let providerModel: (ProviderID) -> ProviderFeatureModel

    var body: some View {
        NavigationSplitView {
            List(selection: self.$shell.navigationSelection) {
                ForEach(self.shell.navigationItems, id: \.id) { item in
                    Label(item.title, systemImage: item.systemImage)
                        .tag(item)
                }
            }
            .listStyle(.sidebar)
            .navigationTitle("HeimdallBar")
            .onChange(of: self.shell.navigationSelection) { _, newValue in
                self.shell.selectNavigation(newValue)
            }
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
                        case .overview:
                            await self.overview.refreshAll()
                        case .provider(let provider):
                            await self.providerModel(provider).refresh()
                        }
                    }
                } label: {
                    Label("Refresh", systemImage: "arrow.clockwise")
                }
                .disabled(self.isBusy)

                SettingsLink {
                    Label("Settings", systemImage: "gearshape")
                }

                Button {
                    if let url = URL(string: "http://127.0.0.1:\(self.helperPort)") {
                        NSWorkspace.shared.open(url)
                    }
                } label: {
                    Label("Open Dashboard", systemImage: "safari")
                }
            }
        }
    }

    private var isBusy: Bool {
        switch self.shell.navigationSelection {
        case .overview:
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
                issue: projection.globalIssueLabel,
                onRetry: {
                    Task { await self.overview.refreshAll() }
                },
                isRetrying: projection.isRefreshing
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

    /// If we have a weekly projection, append it to the refresh-status line
    /// so the user sees the pace at a glance on every provider page.
    static func headerSubtitle(_ projection: ProviderMenuProjection) -> String {
        let status = projection.refreshStatusLabel
        guard let projected = projection.weeklyProjectedCostUSD, projected > 0 else {
            return status
        }
        let formatted: String
        if projected >= 1000 {
            formatted = String(format: "$%.0f", projected)
        } else if projected >= 10 {
            formatted = String(format: "$%.1f", projected)
        } else {
            formatted = String(format: "$%.2f", projected)
        }
        return "\(status) · Weekly projected \(formatted)"
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 18) {
            WindowHeader(
                title: self.model.provider.title,
                subtitle: Self.headerSubtitle(self.model.projection),
                issue: self.model.issue?.message ?? self.model.projection.globalIssueLabel,
                onRetry: {
                    Task { await self.model.refresh() }
                },
                isRetrying: self.model.isBusy
            )

            ProviderMenuCard(providerModel: self.model)

            ProviderSessionDetails(model: self.model)
        }
    }
}

private struct WindowHeader: View {
    let title: String
    let subtitle: String
    let issue: String?
    var onRetry: (() -> Void)? = nil
    var isRetrying: Bool = false

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(self.title)
                .font(.system(size: 24, weight: .semibold))
            Text(self.subtitle)
                .font(.callout)
                .foregroundStyle(.secondary)
            if let issue, !issue.isEmpty {
                HStack(alignment: .top, spacing: 10) {
                    Text(issue)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .fixedSize(horizontal: false, vertical: true)
                    Spacer(minLength: 8)
                    if let onRetry {
                        Button(self.isRetrying ? "Retrying…" : "Retry", action: onRetry)
                            .buttonStyle(.bordered)
                            .controlSize(.small)
                            .disabled(self.isRetrying)
                    }
                }
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
