import HeimdallDomain
import HeimdallServices
import Observation

public enum AppNavigationItem: Hashable, Sendable, Identifiable {
    case overview
    case liveMonitor
    case provider(ProviderID)
    case toolErrors(toolName: String)

    public var id: String {
        switch self {
        case .overview:
            return "overview"
        case .liveMonitor:
            return "live-monitor"
        case .provider(let provider):
            return "provider:\(provider.rawValue)"
        case .toolErrors(let toolName):
            return "tool-errors:\(toolName)"
        }
    }

    public var title: String {
        switch self {
        case .overview:
            return "Overview"
        case .liveMonitor:
            return "Live Monitor"
        case .provider(let provider):
            return provider.title
        case .toolErrors:
            return "Tool Errors"
        }
    }

    public var subtitle: String {
        switch self {
        case .overview:
            return "All providers"
        case .liveMonitor:
            return "Fast refresh"
        case .provider(.claude):
            return "Anthropic usage"
        case .provider(.codex):
            return "OpenAI usage"
        case .toolErrors(let toolName):
            return toolName
        }
    }

    public var systemImage: String {
        switch self {
        case .overview:
            return "square.grid.2x2"
        case .liveMonitor:
            return "waveform.path.ecg.rectangle"
        case .provider(.claude):
            return "quote.bubble"
        case .provider(.codex):
            return "curlybraces.square"
        case .toolErrors:
            return "exclamationmark.triangle"
        }
    }

    public var providerID: ProviderID? {
        switch self {
        case .provider(let provider):
            return provider
        case .overview, .liveMonitor, .toolErrors:
            return nil
        }
    }
}

@MainActor
@Observable
public final class AppShellModel {
    private let sessionStore: AppSessionStore

    public var navigationSelection: AppNavigationItem
    public var selectedMenuTab: MergeMenuTab

    public init(
        sessionStore: AppSessionStore,
        navigationSelection: AppNavigationItem = .overview,
        selectedMenuTab: MergeMenuTab = .overview
    ) {
        self.sessionStore = sessionStore
        let persistedMenuTab = sessionStore.selectedMergeTab
        self.selectedMenuTab = persistedMenuTab
        self.navigationSelection = Self.navigationSelection(for: persistedMenuTab)
        if persistedMenuTab == .overview && selectedMenuTab != .overview {
            self.selectedMenuTab = selectedMenuTab
            self.navigationSelection = Self.navigationSelection(for: selectedMenuTab)
        }
        self.syncSelections()
    }

    public var visibleProviders: [ProviderID] {
        self.sessionStore.visibleProviders
    }

    public var visibleTabs: [MergeMenuTab] {
        MenuProjectionBuilder.availableTabs(config: self.sessionStore.config)
    }

    public var navigationItems: [AppNavigationItem] {
        [.overview, .liveMonitor] + self.visibleProviders.map(AppNavigationItem.provider)
    }

    public func selectNavigation(_ item: AppNavigationItem) {
        self.navigationSelection = item
        if let provider = item.providerID {
            self.sessionStore.selectedProvider = provider
            self.selectedMenuTab = provider == .claude ? .claude : .codex
            self.sessionStore.selectedMergeTab = self.selectedMenuTab
        } else if item == .overview || item == .liveMonitor {
            self.selectedMenuTab = .overview
            self.sessionStore.selectedMergeTab = .overview
        }
    }

    public func selectMenuTab(_ tab: MergeMenuTab) {
        self.selectedMenuTab = tab
        self.sessionStore.selectedMergeTab = tab
        switch tab {
        case .overview:
            self.navigationSelection = .overview
        case .claude:
            self.sessionStore.selectedProvider = .claude
            self.navigationSelection = .provider(.claude)
        case .codex:
            self.sessionStore.selectedProvider = .codex
            self.navigationSelection = .provider(.codex)
        }
    }

    public func syncSelections() {
        if !self.visibleTabs.contains(self.selectedMenuTab) {
            self.selectedMenuTab = self.visibleTabs.first ?? .overview
        }
        // .toolErrors is a programmatic-only destination — never evict it during sync
        if case .toolErrors = self.navigationSelection {
            // keep as-is
        } else if !self.navigationItems.contains(self.navigationSelection) {
            self.navigationSelection = self.navigationItems.first ?? .overview
        }
        if let provider = self.navigationSelection.providerID {
            self.sessionStore.selectedProvider = provider
        }
        self.sessionStore.selectedMergeTab = self.selectedMenuTab
    }

    private static func navigationSelection(for tab: MergeMenuTab) -> AppNavigationItem {
        switch tab {
        case .overview:
            return .overview
        case .claude:
            return .provider(.claude)
        case .codex:
            return .provider(.codex)
        }
    }
}
