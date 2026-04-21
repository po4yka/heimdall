import HeimdallDomain
import SwiftUI

struct SettingsView: View {
    @Bindable var model: SettingsFeatureModel
    let providerModel: (ProviderID) -> ProviderFeatureModel

    var body: some View {
        Form {
            ProviderSettingsSection(
                title: "Claude",
                config: self.$model.config.claude,
                extrasTitle: "Enable Claude Web Extras"
            )
            ProviderAuthSection(model: self.providerModel(.claude))
            ProviderWebSessionSection(model: self.providerModel(.claude))

            ProviderSettingsSection(
                title: "Codex",
                config: self.$model.config.codex,
                extrasTitle: "Enable Codex OpenAI Web Extras"
            )
            ProviderAuthSection(model: self.providerModel(.codex))
            ProviderWebSessionSection(model: self.providerModel(.codex))

            Section("Web Extras Policy") {
                Text("OpenAI web extras use a hidden local WKWebView with imported browser cookies. Refreshes are cached and rate-limited, but they still cost battery and expose your local browser session to this app only on this machine.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            Section("Display") {
                Toggle("Merge Icons", isOn: self.$model.config.mergeIcons)
                Toggle("Show Used Values", isOn: self.$model.config.showUsedValues)
                Toggle("Check Provider Status", isOn: self.$model.config.checkProviderStatus)
                Picker("Reset Display", selection: self.$model.config.resetDisplayMode) {
                    ForEach(ResetDisplayMode.allCases, id: \.self) { mode in
                        Text(mode.rawValue.capitalized).tag(mode)
                    }
                }
                Stepper(
                    "Refresh Interval: \(self.model.config.refreshIntervalSeconds)s",
                    value: self.$model.config.refreshIntervalSeconds,
                    in: 60...1800,
                    step: 60
                )
            }

            Section {
                Button("Save Settings") {
                    self.model.save()
                }
                Button("Refresh Data") {
                    Task { await self.model.refreshAll() }
                }
                Button("Refresh Browser Discovery") {
                    Task { await self.model.refreshBrowserImports() }
                }
            }
        }
        .formStyle(.grouped)
        .padding()
        .task {
            await self.model.refreshBrowserImports()
        }
    }
}

private struct ProviderSettingsSection: View {
    let title: String
    @Binding var config: ProviderConfig
    let extrasTitle: String

    var body: some View {
        Section(self.title) {
            Toggle("Enable \(self.title)", isOn: self.$config.enabled)
            Picker("\(self.title) Source", selection: self.$config.source) {
                ForEach(UsageSourcePreference.allCases, id: \.self) { source in
                    Text(source.rawValue.capitalized).tag(source)
                }
            }
            Picker("\(self.title) Cookie Source", selection: self.$config.cookieSource) {
                ForEach(UsageSourcePreference.allCases, id: \.self) { source in
                    Text(source.rawValue.capitalized).tag(source)
                }
            }
            Toggle(self.extrasTitle, isOn: self.$config.dashboardExtrasEnabled)
        }
    }
}

private struct ProviderAuthSection: View {
    @Bindable var model: ProviderFeatureModel

    var body: some View {
        let projection = self.model.projection
        let auth = self.model.authHealth

        Section("\(self.model.provider.title) Auth") {
            if let headline = projection.authHeadline {
                Text(headline)
                    .font(.headline)
            } else {
                Text("No auth diagnosis available yet.")
                    .foregroundStyle(.secondary)
            }

            if let detail = projection.authDetail {
                Text(detail)
                    .foregroundStyle(.secondary)
            }

            if let loginMethod = auth?.loginMethod {
                LabeledContent("Login Method", value: loginMethod)
            }
            if let backend = auth?.credentialBackend {
                LabeledContent("Credential Store", value: backend)
            }
            if let authMode = auth?.authMode {
                LabeledContent("Auth Mode", value: authMode)
            }
            if let diagnostic = auth?.diagnosticCode {
                LabeledContent("Diagnostic", value: diagnostic)
            }

            LabeledContent("Authenticated", value: auth?.isAuthenticated == true ? "Yes" : "No")
            LabeledContent("Source Compatible", value: auth?.isSourceCompatible == true ? "Yes" : "No")
            LabeledContent("Requires Re-login", value: auth?.requiresRelogin == true ? "Yes" : "No")

            ForEach(self.model.authRecoveryActions) { action in
                Button(action.label) {
                    Task { await self.model.runAuthRecoveryAction(action) }
                }
            }
        }
    }
}

struct ProviderWebSessionSection: View {
    @Bindable var model: ProviderFeatureModel

    var body: some View {
        Section("\(self.model.provider.title) Web Session") {
            if let session = self.model.importedSession {
                Text(statusLine(for: session))
                Text("Source: \(session.browserSource.title) · \(session.profileName)")
                    .foregroundStyle(.secondary)
                Text("Cookies: \(session.cookies.count) · Imported: \(session.importedAt)")
                    .foregroundStyle(.secondary)
                if !session.cookies.isEmpty {
                    Text("Domains: \(previewDomains(session))")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                Button("Reset \(self.model.provider.title) Session") {
                    Task { await self.model.resetBrowserSession() }
                }
                .disabled(self.model.isImportingSession)
            } else {
                Text("No imported browser session stored.")
                    .foregroundStyle(.secondary)
            }

            if self.model.importCandidates.isEmpty {
                Text("No Safari, Chrome, Arc, or Brave cookie stores were discovered.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            } else {
                ForEach(self.model.importCandidates) { candidate in
                    Button("Import from \(candidate.title)") {
                        Task { await self.model.importBrowserSession(candidate: candidate) }
                    }
                    .disabled(self.model.isImportingSession)
                }
            }
        }
    }

    private func statusLine(for session: ImportedBrowserSession) -> String {
        if session.expired {
            return "Stored session is expired."
        }
        if session.loginRequired {
            return "Stored session is missing an active auth cookie."
        }
        return "Stored session is ready."
    }

    private func previewDomains(_ session: ImportedBrowserSession) -> String {
        let domains = Array(Set(session.cookies.map(\.domain))).sorted()
        return domains.prefix(3).joined(separator: ", ")
    }
}
