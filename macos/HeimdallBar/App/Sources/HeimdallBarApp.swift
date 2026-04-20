import AppKit
import HeimdallBarShared
import Observation
import SwiftUI

final class AppDelegate: NSObject, NSApplicationDelegate {
    var model: AppModel?

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

@main
struct HeimdallBarApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) private var appDelegate
    @State private var model = AppModel()

    var body: some Scene {
        MenuBarExtra(isInserted: .constant(self.model.config.mergeIcons)) {
            RootMenuView(model: self.model)
                .task { self.model.start() }
                .onAppear { self.appDelegate.model = self.model }
        } label: {
            let overview = self.model.overviewProjection()
            MenuBarLabel(
                title: "Heimdall",
                image: MenuBarMeterRenderer.mergedImage(from: overview.items, isRefreshing: overview.isRefreshing)
            )
        }
        .menuBarExtraStyle(.window)

        MenuBarExtra(isInserted: .constant(!self.model.config.mergeIcons && self.model.config.claude.enabled)) {
            ProviderMenuView(model: self.model, provider: .claude)
                .task { self.model.start() }
                .onAppear { self.appDelegate.model = self.model }
        } label: {
            let projection = self.model.projection(for: .claude)
            MenuBarLabel(
                title: self.model.menuTitle(for: .claude),
                image: MenuBarMeterRenderer.image(for: projection)
            )
        }
        .menuBarExtraStyle(.window)

        MenuBarExtra(isInserted: .constant(!self.model.config.mergeIcons && self.model.config.codex.enabled)) {
            ProviderMenuView(model: self.model, provider: .codex)
                .task { self.model.start() }
                .onAppear { self.appDelegate.model = self.model }
        } label: {
            let projection = self.model.projection(for: .codex)
            MenuBarLabel(
                title: self.model.menuTitle(for: .codex),
                image: MenuBarMeterRenderer.image(for: projection)
            )
        }
        .menuBarExtraStyle(.window)

        Settings {
            SettingsView(model: self.model)
                .frame(width: 480, height: 360)
                .onAppear { self.appDelegate.model = self.model }
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
