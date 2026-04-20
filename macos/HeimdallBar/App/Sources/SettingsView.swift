import HeimdallBarShared
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
            }
        }
        .formStyle(.grouped)
        .padding()
    }
}
