import Foundation
import HeimdallDomain
import HeimdallServices
import Observation

@MainActor
@Observable
public final class LiveMonitorFeatureModel {
    public var envelope: LiveMonitorEnvelope?
    public var focus: LiveMonitorFocus
    public var density: LiveMonitorDensity
    public var hiddenPanels: Set<LiveMonitorPanelID>
    public var isRefreshing: Bool
    public var issue: String?

    private let sessionStore: AppSessionStore
    private let clientFactory: @Sendable (Int) -> any LiveMonitorClient
    private let pollInterval: Duration
    private let reconnectInitialDelayNanoseconds: UInt64
    private let reconnectMaxDelayNanoseconds: UInt64
    private var pollTask: Task<Void, Never>?
    private var eventTask: Task<Void, Never>?
    private var isActive = false

    public init(
        sessionStore: AppSessionStore,
        clientFactory: @escaping @Sendable (Int) -> any LiveMonitorClient,
        pollInterval: Duration = .seconds(10),
        reconnectInitialDelayNanoseconds: UInt64 = 1_000_000_000,
        reconnectMaxDelayNanoseconds: UInt64 = 30_000_000_000
    ) {
        self.sessionStore = sessionStore
        self.clientFactory = clientFactory
        self.pollInterval = pollInterval
        self.reconnectInitialDelayNanoseconds = reconnectInitialDelayNanoseconds
        self.reconnectMaxDelayNanoseconds = reconnectMaxDelayNanoseconds
        let savedPreferences = sessionStore.liveMonitorPreferences
        self.focus = savedPreferences?.focus ?? .all
        self.density = savedPreferences?.density ?? .expanded
        self.hiddenPanels = Set(savedPreferences?.hiddenPanels ?? [])
        self.isRefreshing = false
    }

    public var providers: [LiveMonitorProvider] {
        guard let envelope else { return [] }
        switch self.focus {
        case .all:
            return envelope.providers
        case .claude, .codex:
            return envelope.providers.filter { $0.provider == self.focus.rawValue }
        }
    }

    public var detailProviders: [LiveMonitorProvider] {
        let providers = self.providers
        if self.focus != .all {
            return providers.filter { self.hasVisibleDetailPanels(for: $0) }
        }
        return providers.filter { self.hasVisibleDetailPanels(for: $0) }
    }

    public func setFocus(_ focus: LiveMonitorFocus) {
        self.focus = focus
        self.persistPreferences()
    }

    public func setDensity(_ density: LiveMonitorDensity) {
        self.density = density
        self.persistPreferences()
    }

    public func isPanelHidden(_ panelID: LiveMonitorPanelID) -> Bool {
        self.hiddenPanels.contains(panelID)
    }

    public func setPanelVisibility(_ panelID: LiveMonitorPanelID, isVisible: Bool) {
        var updatedHiddenPanels = self.hiddenPanels
        if isVisible {
            updatedHiddenPanels.remove(panelID)
        } else {
            updatedHiddenPanels.insert(panelID)
        }
        self.hiddenPanels = updatedHiddenPanels
        self.persistPreferences()
    }

    public func updateActivity(isSelected: Bool, appIsActive: Bool) {
        let shouldBeActive = isSelected && appIsActive
        guard shouldBeActive != self.isActive else { return }
        self.isActive = shouldBeActive
        if shouldBeActive {
            self.startMonitoring()
        } else {
            self.stopMonitoring()
        }
    }

    public func refresh() async {
        self.isRefreshing = true
        defer { self.isRefreshing = false }

        do {
            let envelope = try await self.clientFactory(self.sessionStore.config.helperPort).fetchLiveMonitor()
            self.envelope = envelope
            self.applyResolvedFocus(for: envelope)
            self.issue = envelope.globalIssue
        } catch {
            self.issue = error.localizedDescription
        }
    }

    private func startMonitoring() {
        if self.envelope == nil {
            Task { await self.refresh() }
        }
        self.startPollingLoop()
        self.startEventLoop()
    }

    private func stopMonitoring() {
        self.pollTask?.cancel()
        self.pollTask = nil
        self.eventTask?.cancel()
        self.eventTask = nil
    }

    private func startPollingLoop() {
        self.pollTask?.cancel()
        self.pollTask = Task { @MainActor [weak self] in
            while let self, !Task.isCancelled {
                try? await Task.sleep(for: self.pollInterval)
                if Task.isCancelled { return }
                await self.refresh()
            }
        }
    }

    private func startEventLoop() {
        self.eventTask?.cancel()
        self.eventTask = Task { @MainActor [weak self] in
            guard let self else { return }
            var backoffNanoseconds: UInt64 = self.reconnectInitialDelayNanoseconds

            while !Task.isCancelled {
                do {
                    let stream = self.clientFactory(self.sessionStore.config.helperPort).liveMonitorEvents()
                    for try await event in stream {
                        if Task.isCancelled { return }
                        if event == "scan_completed" {
                            await self.refresh()
                        }
                    }
                    backoffNanoseconds = 1_000_000_000
                } catch {
                    if self.envelope == nil {
                        self.issue = error.localizedDescription
                    }
                    try? await Task.sleep(nanoseconds: backoffNanoseconds)
                    backoffNanoseconds = min(backoffNanoseconds * 2, self.reconnectMaxDelayNanoseconds)
                }
            }
        }
    }

    private func applyResolvedFocus(for envelope: LiveMonitorEnvelope) {
        let savedFocus = self.sessionStore.liveMonitorPreferences?.focus
        if let savedFocus {
            let resolvedFocus = self.resolveFocus(savedFocus, in: envelope)
            self.focus = resolvedFocus
            if resolvedFocus != savedFocus {
                self.persistPreferences()
            }
            return
        }
        self.focus = self.resolveFocus(envelope.defaultFocus, in: envelope)
    }

    private func resolveFocus(_ focus: LiveMonitorFocus, in envelope: LiveMonitorEnvelope) -> LiveMonitorFocus {
        if focus == .all {
            return .all
        }
        return envelope.providers.contains(where: { $0.provider == focus.rawValue }) ? focus : .all
    }

    private func hasVisibleDetailPanels(for provider: LiveMonitorProvider) -> Bool {
        (!self.hiddenPanels.contains(.activeBlock) && provider.activeBlock != nil)
            || (!self.hiddenPanels.contains(.depletionForecast) && provider.depletionForecast != nil)
            || (!self.hiddenPanels.contains(.quotaSuggestions) && provider.quotaSuggestions != nil)
            || provider.predictiveInsights != nil
            || (!self.hiddenPanels.contains(.contextWindow) && provider.contextWindow != nil)
            || (!self.hiddenPanels.contains(.recentSession) && provider.recentSession != nil)
            || (!self.hiddenPanels.contains(.warnings) && !provider.warnings.isEmpty)
    }

    private func persistPreferences() {
        self.sessionStore.liveMonitorPreferences = LiveMonitorPreferences(
            focus: self.focus,
            density: self.density,
            hiddenPanels: self.hiddenPanels.sorted { $0.rawValue < $1.rawValue }
        )
    }
}
