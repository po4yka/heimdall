import AppKit
import CloudKit
import Observation
import SwiftUI
import os.log

enum HeimdallBarSceneID {
    static let mainWindow = "heimdall-main-window"
}

final class AppDelegate: NSObject, NSApplicationDelegate {
    private static let shutdownTimeoutNanoseconds: UInt64 = 2_000_000_000
    private static let logger = Logger(subsystem: "dev.heimdall.HeimdallBar", category: "Lifecycle")

    var model: AppModel?
    private var isPreparingToTerminate = false

    @MainActor
    func attach(model: AppModel) {
        self.model = model
    }

    func application(_: NSApplication, userDidAcceptCloudKitShareWith metadata: CKShare.Metadata) {
        guard let url = metadata.share.url else { return }
        Task { @MainActor [weak self] in
            await self?.model?.handleIncomingCloudShare(url: url)
        }
    }

    func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply {
        guard let model else { return .terminateNow }
        guard !self.isPreparingToTerminate else { return .terminateLater }

        self.isPreparingToTerminate = true
        Task { [weak self] in
            let shutdownCompleted = await Self.awaitShutdown(timeoutNanoseconds: Self.shutdownTimeoutNanoseconds) {
                await model.prepareForExit()
            }
            await MainActor.run {
                if !shutdownCompleted {
                    Self.logger.error("Shutdown timed out; terminating before helper cleanup completed.")
                }
                self?.isPreparingToTerminate = false
                sender.reply(toApplicationShouldTerminate: true)
            }
        }
        return .terminateLater
    }

    static func awaitShutdown(
        timeoutNanoseconds: UInt64,
        operation: @escaping @Sendable () async -> Void
    ) async -> Bool {
        await withTaskGroup(of: Bool.self) { group in
            group.addTask {
                await operation()
                return true
            }
            group.addTask {
                try? await Task.sleep(nanoseconds: timeoutNanoseconds)
                return false
            }

            let completed = await group.next() ?? true
            group.cancelAll()
            return completed
        }
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
                liveMonitor: self.model.liveMonitor,
                helperPort: self.model.config.helperPort,
                providerModel: self.model.providerModel(for:)
            )
                .frame(minWidth: 900, idealWidth: 1080, minHeight: 620, idealHeight: 720)
                .background(MainWindowIdentityTagger(sceneID: HeimdallBarSceneID.mainWindow))
                .onAppear { self.appDelegate.attach(model: self.model) }
                .onOpenURL { url in
                    Task { await self.model.handleIncomingCloudShare(url: url) }
                }
        }

        MenuBarExtra(isInserted: self.mergedMenuBarBinding) {
            RootMenuView(
                shell: self.model.shell,
                overview: self.model.overview,
                providerModel: self.model.providerModel(for:),
                helperPort: self.model.config.helperPort,
                onQuit: self.quit
            )
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

        MenuBarExtra(isInserted: self.claudeMenuBarBinding) {
            ProviderMenuView(
                model: self.model.providerModel(for: .claude),
                helperPort: self.model.config.helperPort,
                onQuit: self.quit
            )
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

        MenuBarExtra(isInserted: self.codexMenuBarBinding) {
            ProviderMenuView(
                model: self.model.providerModel(for: .codex),
                helperPort: self.model.config.helperPort,
                onQuit: self.quit
            )
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
                .onOpenURL { url in
                    Task { await self.model.handleIncomingCloudShare(url: url) }
                }
        }
    }

    private var mergedMenuBarBinding: Binding<Bool> {
        Self.readOnlyBinding {
            self.model.sessionStore.config.mergeIcons
        }
    }

    private var claudeMenuBarBinding: Binding<Bool> {
        Self.readOnlyBinding {
            let config = self.model.sessionStore.config
            return !config.mergeIcons && config.claude.enabled
        }
    }

    private var codexMenuBarBinding: Binding<Bool> {
        Self.readOnlyBinding {
            let config = self.model.sessionStore.config
            return !config.mergeIcons && config.codex.enabled
        }
    }

    private var quit: () -> Void {
        {
            NSApplication.shared.terminate(nil)
        }
    }

    private static func readOnlyBinding(
        _ get: @escaping @MainActor () -> Bool
    ) -> Binding<Bool> {
        Binding(
            get: { get() },
            set: { _ in }
        )
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

enum MainWindowFocusNormalizer {
    @MainActor
    static func shouldNormalizeInitialFocus(firstResponder: NSResponder?, contentView: NSView?) -> Bool {
        guard let contentView else { return false }
        guard let firstResponder else { return true }
        guard let focusedView = firstResponder as? NSView else { return true }
        return !focusedView.isDescendant(of: contentView)
    }
}

private struct MainWindowIdentityTagger: NSViewRepresentable {
    let sceneID: String

    func makeNSView(context _: Context) -> MainWindowIdentityView {
        let view = MainWindowIdentityView()
        view.sceneID = self.sceneID
        return view
    }

    func updateNSView(_ nsView: MainWindowIdentityView, context _: Context) {
        nsView.sceneID = self.sceneID
        nsView.applyWindowIdentifierIfNeeded()
    }
}

private final class MainWindowIdentityView: NSView {
    var sceneID: String = HeimdallBarSceneID.mainWindow
    private var didNormalizeInitialFocus = false

    override func viewDidMoveToWindow() {
        super.viewDidMoveToWindow()
        self.applyWindowIdentifierIfNeeded()
    }

    func applyWindowIdentifierIfNeeded() {
        guard let window else { return }
        window.identifier = NSUserInterfaceItemIdentifier(self.sceneID)
        self.normalizeInitialFocusIfNeeded(in: window)
    }

    private func normalizeInitialFocusIfNeeded(in window: NSWindow) {
        guard !self.didNormalizeInitialFocus else { return }
        guard MainWindowFocusNormalizer.shouldNormalizeInitialFocus(
            firstResponder: window.firstResponder,
            contentView: window.contentView
        ) else { return }

        self.didNormalizeInitialFocus = true
        DispatchQueue.main.async { [weak window] in
            guard let window else { return }
            _ = window.makeFirstResponder(window.contentView)
        }
    }
}
