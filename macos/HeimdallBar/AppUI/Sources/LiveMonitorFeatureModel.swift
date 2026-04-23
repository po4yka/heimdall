import Foundation
import HeimdallDomain
import HeimdallServices
import Observation

@MainActor
@Observable
public final class LiveMonitorFeatureModel {
    public var envelope: LiveMonitorEnvelope?
    public var focus: LiveMonitorFocus
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
        self.focus = .all
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
            return providers
        }
        return providers.filter {
            $0.activeBlock != nil
                || $0.contextWindow != nil
                || $0.recentSession != nil
                || $0.depletionForecast != nil
                || !$0.warnings.isEmpty
        }
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
            self.focus = envelope.defaultFocus
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
}
