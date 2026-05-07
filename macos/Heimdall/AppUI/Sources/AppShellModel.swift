import HeimdallDomain
import HeimdallServices
import Observation

public enum AppNavigationItem: Hashable, Sendable, Identifiable {
    case overview
    case today
    case activity
    case agents
    case costModels
    case sessions
    case projects
    case liveMonitor
    // Programmatic-only destinations (not shown in sidebar):
    case provider(ProviderID)
    case toolErrors(toolName: String)

    public var id: String {
        switch self {
        case .overview:            return "overview"
        case .today:               return "today"
        case .activity:            return "activity"
        case .agents:              return "agents"
        case .costModels:          return "cost-models"
        case .sessions:            return "sessions"
        case .projects:            return "projects"
        case .liveMonitor:         return "live-monitor"
        case .provider(let p):     return "provider:\(p.rawValue)"
        case .toolErrors(let t):   return "tool-errors:\(t)"
        }
    }

    public var title: String {
        switch self {
        case .overview:            return "Overview"
        case .today:               return "Today"
        case .activity:            return "Activity"
        case .agents:              return "Agents"
        case .costModels:          return "Cost & Models"
        case .sessions:            return "Sessions"
        case .projects:            return "Projects"
        case .liveMonitor:         return "Live Monitor"
        case .provider(let p):     return p.title
        case .toolErrors:          return "Tool Errors"
        }
    }

    public var subtitle: String {
        switch self {
        case .overview:            return "All providers"
        case .today:               return "Today's usage"
        case .activity:            return "Trends & charts"
        case .agents:              return "Agent activity"
        case .costModels:          return "Cost & models"
        case .sessions:            return "Sessions"
        case .projects:            return "Projects"
        case .liveMonitor:         return "Fast refresh"
        case .provider(.claude):   return "Anthropic usage"
        case .provider(.codex):    return "OpenAI usage"
        case .toolErrors(let t):   return t
        }
    }

    public var systemImage: String {
        switch self {
        case .overview:            return "square.grid.2x2"
        case .today:               return "sun.max"
        case .activity:            return "chart.line.uptrend.xyaxis"
        case .agents:              return "person.3"
        case .costModels:          return "dollarsign.circle"
        case .sessions:            return "list.bullet.rectangle"
        case .projects:            return "folder"
        case .liveMonitor:         return "waveform.path.ecg.rectangle"
        case .provider(.claude):   return "quote.bubble"
        case .provider(.codex):    return "curlybraces.square"
        case .toolErrors:          return "exclamationmark.triangle"
        }
    }

    public var providerID: ProviderID? {
        if case .provider(let p) = self { return p }
        return nil
    }

    /// True for items that appear in the sidebar navigation list.
    public var isNavigationItem: Bool {
        switch self {
        case .provider, .toolErrors: return false
        default: return true
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

    /// Ordered sidebar destinations. Provider and toolErrors are programmatic-only and excluded.
    public var navigationItems: [AppNavigationItem] {
        [.overview, .today, .activity, .agents, .costModels, .sessions, .projects, .liveMonitor]
    }

    public func selectNavigation(_ item: AppNavigationItem) {
        self.navigationSelection = item
        switch item {
        case .provider(let provider):
            self.sessionStore.selectedProvider = provider
            self.selectedMenuTab = provider == .claude ? .claude : .codex
            self.sessionStore.selectedMergeTab = self.selectedMenuTab
        case .overview, .today, .activity, .agents, .costModels, .sessions, .projects, .liveMonitor:
            self.selectedMenuTab = .overview
            self.sessionStore.selectedMergeTab = .overview
        case .toolErrors:
            break
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
        // .provider and .toolErrors are programmatic-only — never evict them during sync
        if !self.navigationSelection.isNavigationItem {
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
