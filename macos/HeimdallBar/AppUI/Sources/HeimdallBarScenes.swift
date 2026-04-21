import AppKit
import Observation
import SwiftUI

final class AppDelegate: NSObject, NSApplicationDelegate {
    var model: AppModel?

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

    func applicationWillTerminate(_ notification: Notification) {
        guard let model else { return }
        let semaphore = DispatchSemaphore(value: 0)
        Task {
            await model.prepareForExit()
            semaphore.signal()
        }
        _ = semaphore.wait(timeout: .now() + 2)
    }
}

public struct HeimdallBarScenes: Scene {
    @NSApplicationDelegateAdaptor(AppDelegate.self) private var appDelegate
    @State private var model: AppModel

    public init(model: AppModel) {
        self._model = State(initialValue: model)
    }

    public var body: some Scene {
        WindowGroup("HeimdallBar") {
            AppShellView(
                shell: self.model.shell,
                overview: self.model.overview,
                settings: self.model.settings,
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
            Task {
                await self.model.prepareForExit()
                NSApplication.shared.terminate(nil)
            }
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
