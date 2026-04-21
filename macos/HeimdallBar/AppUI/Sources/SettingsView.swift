import HeimdallDomain
import SwiftUI

struct SettingsView: View {
    @Bindable var model: AppModel

    var body: some View {
        Form {
            Section("Claude") {
                Toggle("Enable Claude", isOn: self.$model.config.claude.enabled)
                Picker("Claude Source", selection: self.$model.config.claude.source) {
                    ForEach(UsageSourcePreference.allCases, id: \.self) { source in
                        Text(source.rawValue.capitalized).tag(source)
                    }
                }
                Picker("Claude Cookie Source", selection: self.$model.config.claude.cookieSource) {
                    ForEach(UsageSourcePreference.allCases, id: \.self) { source in
                        Text(source.rawValue.capitalized).tag(source)
                    }
                }
                Toggle("Enable Claude Web Extras", isOn: self.$model.config.claude.dashboardExtrasEnabled)
            }
            ProviderAuthSection(model: self.model, provider: .claude)
            ProviderWebSessionSection(model: self.model, provider: .claude)

            Section("Codex") {
                Toggle("Enable Codex", isOn: self.$model.config.codex.enabled)
                Picker("Codex Source", selection: self.$model.config.codex.source) {
                    ForEach(UsageSourcePreference.allCases, id: \.self) { source in
                        Text(source.rawValue.capitalized).tag(source)
                    }
                }
                Picker("Codex Cookie Source", selection: self.$model.config.codex.cookieSource) {
                    ForEach(UsageSourcePreference.allCases, id: \.self) { source in
                        Text(source.rawValue.capitalized).tag(source)
                    }
                }
                Toggle("Enable Codex OpenAI Web Extras", isOn: self.$model.config.codex.dashboardExtrasEnabled)
            }
            ProviderAuthSection(model: self.model, provider: .codex)
            ProviderWebSessionSection(model: self.model, provider: .codex)

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
                Stepper("Refresh Interval: \(self.model.config.refreshIntervalSeconds)s", value: self.$model.config.refreshIntervalSeconds, in: 60...1800, step: 60)
            }

            Section {
                Button("Save Settings") {
                    self.model.saveConfig()
                }
                Button("Refresh Data") {
                    Task { await self.model.refresh(force: true, provider: nil) }
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

private struct ProviderAuthSection: View {
    @Bindable var model: AppModel
    let provider: ProviderID

    var body: some View {
        let projection = self.model.projection(for: self.provider)
        let auth = self.model.authHealth(for: self.provider)

        Section("\(self.provider.title) Auth") {
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

            ForEach(self.model.authRecoveryActions(for: self.provider)) { action in
                Button(action.label) {
                    Task { await self.model.runAuthRecoveryAction(action, for: self.provider) }
                }
            }
        }
    }
}

private struct ProviderWebSessionSection: View {
    let model: AppModel
    let provider: ProviderID

    var body: some View {
        Section("\(self.provider.title) Web Session") {
            if let session = self.model.importedSession(for: self.provider) {
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
                Button("Reset \(self.provider.title) Session") {
                    Task { await self.model.resetBrowserSession(provider: self.provider) }
                }
                .disabled(self.model.isImportingSession)
            } else {
                Text("No imported browser session stored.")
                    .foregroundStyle(.secondary)
            }

            if self.model.importCandidates(for: self.provider).isEmpty {
                Text("No Safari, Chrome, Arc, or Brave cookie stores were discovered.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            } else {
                ForEach(self.model.importCandidates(for: self.provider)) { candidate in
                    Button("Import from \(candidate.title)") {
                        Task { await self.model.importBrowserSession(provider: self.provider, candidate: candidate) }
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
