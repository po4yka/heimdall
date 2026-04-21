import AppKit
import Observation
import SwiftUI

enum HeimdallBarSceneID {
    static let mainWindow = "heimdall-main-window"
}

final class AppDelegate: NSObject, NSApplicationDelegate {
    var model: AppModel?
    private var isPreparingToTerminate = false

    func applicationDidFinishLaunching(_ notification: Notification) {
        Task { @MainActor in
            self.model?.start()
        }
    }

    @MainActor
    func attach(model: AppModel) {
        self.model = model
        model.start()
    }

    func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply {
        guard let model else { return .terminateNow }
        guard !self.isPreparingToTerminate else { return .terminateLater }

        self.isPreparingToTerminate = true
        Task {
            await model.prepareForExit()
            await MainActor.run {
                sender.reply(toApplicationShouldTerminate: true)
            }
        }
        return .terminateLater
    }
}

public struct HeimdallBarScenes: Scene {
    @NSApplicationDelegateAdaptor(AppDelegate.self) private var appDelegate
    @State private var model: AppModel

    public init(model: AppModel) {
        self._model = State(initialValue: model)
    }

    public var body: some Scene {
        WindowGroup("HeimdallBar", id: HeimdallBarSceneID.mainWindow) {
            AppShellView(
                shell: self.model.shell,
                overview: self.model.overview,
                helperPort: self.model.config.helperPort,
                providerModel: self.model.providerModel(for:)
            )
                .frame(minWidth: 900, idealWidth: 1080, minHeight: 620, idealHeight: 720)
                .task { self.model.start() }
                .onAppear { self.appDelegate.attach(model: self.model) }
        }

        MenuBarExtra(isInserted: .constant(self.model.config.mergeIcons)) {
            RootMenuView(
                shell: self.model.shell,
                overview: self.model.overview,
                providerModel: self.model.providerModel(for:),
                helperPort: self.model.config.helperPort,
                onQuit: self.quit
            )
                .task { self.model.start() }
                .onAppear { self.appDelegate.attach(model: self.model) }
        } label: {
            let overview = self.model.overview.projection
            MenuBarLabel(
                title: "Heimdall",
                image: MenuBarMeterRenderer.mergedImage(from: overview.items, isRefreshing: overview.isRefreshing)
            )
            .onAppear { self.appDelegate.attach(model: self.model) }
        }
        .menuBarExtraStyle(.window)

        MenuBarExtra(isInserted: .constant(!self.model.config.mergeIcons && self.model.config.claude.enabled)) {
            ProviderMenuView(
                model: self.model.providerModel(for: .claude),
                helperPort: self.model.config.helperPort,
                onQuit: self.quit
            )
                .task { self.model.start() }
                .onAppear { self.appDelegate.attach(model: self.model) }
        } label: {
            let providerModel = self.model.providerModel(for: .claude)
            MenuBarLabel(
                title: providerModel.menuTitle,
                image: MenuBarMeterRenderer.image(for: providerModel.projection)
            )
            .onAppear { self.appDelegate.attach(model: self.model) }
        }
        .menuBarExtraStyle(.window)

        MenuBarExtra(isInserted: .constant(!self.model.config.mergeIcons && self.model.config.codex.enabled)) {
            ProviderMenuView(
                model: self.model.providerModel(for: .codex),
                helperPort: self.model.config.helperPort,
                onQuit: self.quit
            )
                .task { self.model.start() }
                .onAppear { self.appDelegate.attach(model: self.model) }
        } label: {
            let providerModel = self.model.providerModel(for: .codex)
            MenuBarLabel(
                title: providerModel.menuTitle,
                image: MenuBarMeterRenderer.image(for: providerModel.projection)
            )
            .onAppear { self.appDelegate.attach(model: self.model) }
        }
        .menuBarExtraStyle(.window)

        Settings {
            SettingsView(
                model: self.model.settings,
                providerModel: self.model.providerModel(for:)
            )
                .frame(width: 480, height: 360)
                .onAppear { self.appDelegate.attach(model: self.model) }
        }
    }

    private var quit: () -> Void {
        {
            NSApplication.shared.terminate(nil)
        }
    }
}

struct MenuBarLabel: View {
    let title: String
    let image: NSImage

    var body: some View {
        HStack(spacing: 6) {
            Image(nsImage: self.image)
            Text(self.title)
        }
    }
}
