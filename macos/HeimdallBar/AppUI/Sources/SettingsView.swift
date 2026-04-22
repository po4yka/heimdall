import HeimdallDomain
import SwiftUI

struct SettingsView: View {
    @Bindable var model: SettingsFeatureModel
    let providerModel: (ProviderID) -> ProviderFeatureModel

    var body: some View {
        Form {
            CombinedProviderSection(
                title: "Claude",
                config: self.$model.draftConfig.claude,
                extrasTitle: "Enable Claude Web Extras",
                providerModel: self.providerModel(.claude)
            )
            ProviderWebSessionSection(model: self.providerModel(.claude))

            CombinedProviderSection(
                title: "Codex",
                config: self.$model.draftConfig.codex,
                extrasTitle: "Enable Codex OpenAI Web Extras",
                providerModel: self.providerModel(.codex)
            )
            ProviderWebSessionSection(model: self.providerModel(.codex))

            Section("Web Extras Policy") {
                Text("OpenAI web extras use a hidden local WKWebView with imported browser cookies. Refreshes are cached and rate-limited, but they still cost battery and expose your local browser session to this app only on this machine.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            Section("Display") {
                Toggle("Merge Icons", isOn: self.$model.draftConfig.mergeIcons)
                Toggle("Show Used Values", isOn: self.$model.draftConfig.showUsedValues)
                Toggle("Check Provider Status", isOn: self.$model.draftConfig.checkProviderStatus)
                Picker("Reset Display", selection: self.$model.draftConfig.resetDisplayMode) {
                    ForEach(ResetDisplayMode.allCases, id: \.self) { mode in
                        Text(mode.rawValue.capitalized).tag(mode)
                    }
                }
                Stepper(
                    "Refresh Interval: \(self.model.draftConfig.refreshIntervalSeconds)s",
                    value: self.$model.draftConfig.refreshIntervalSeconds,
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
        .onAppear {
            self.model.resetDraftFromLiveConfig()
        }
        .task {
            await self.model.refreshBrowserImports()
        }
    }
}

private struct CombinedProviderSection: View {
    let title: String
    @Binding var config: ProviderConfig
    let extrasTitle: String
    @Bindable var providerModel: ProviderFeatureModel

    var body: some View {
        Section(self.title) {
            // Config rows
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

            // Auth sub-header
            HStack {
                Text("Auth")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Spacer()
            }
            .frame(maxWidth: .infinity, alignment: .leading)

            // Auth rows
            let projection = self.providerModel.projection
            let auth = self.providerModel.authHealth

            if let headline = projection.authHeadline {
                Text(headline)
                    .font(.headline)
            } else if auth == nil {
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
                LabeledContent("Credential Store", value: backend.replacingOccurrences(of: "-", with: " "))
            }
            // Fix 1: show Auth Mode only when it differs from Login Method
            if let authMode = auth?.authMode,
               authMode.lowercased() != (auth?.loginMethod ?? "").lowercased() {
                LabeledContent("Auth Mode", value: authMode)
            }
            if let diagnostic = auth?.diagnosticCode {
                LabeledContent("Diagnostic", value: diagnostic)
            }

            // Fix 3: single Status row collapsing three health flags
            if let auth {
                let statusValue: String = {
                    if auth.isAuthenticated != true { return "Not authenticated" }
                    if auth.requiresRelogin == true { return "Needs re-login" }
                    if auth.isSourceCompatible != true { return "Source not compatible" }
                    return "Healthy"
                }()
                let tooltip = "Authenticated: \(auth.isAuthenticated == true ? "Yes" : "No") · Source Compatible: \(auth.isSourceCompatible == true ? "Yes" : "No") · Requires Re-login: \(auth.requiresRelogin == true ? "Yes" : "No")"
                LabeledContent("Status", value: statusValue)
                    .help(tooltip)
            }

            // Fix 2 & 5: horizontal row of labelled recovery buttons
            if !self.providerModel.authRecoveryActions.isEmpty {
                HStack {
                    ForEach(self.providerModel.authRecoveryActions) { action in
                        Button(self.displayLabel(for: action)) {
                            Task { await self.providerModel.runAuthRecoveryAction(action) }
                        }
                        .buttonStyle(.bordered)
                        .controlSize(.small)
                    }
                    Spacer()
                }
            }
        }
    }

    private func displayLabel(for action: AuthRecoveryAction) -> String {
        switch action.actionID {
        case "claude-run": return "Open Claude Code"
        case "claude-login": return "Open Claude Code (then /login)"
        case "claude-doctor": return "Open Claude Code (then /doctor)"
        case "codex-login": return "Run Codex Login"
        case "codex-login-device": return "Run Codex Device Login"
        default: return action.label
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
