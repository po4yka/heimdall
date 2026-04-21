import HeimdallDomain
import HeimdallServices
import Observation

public enum AppNavigationItem: Hashable, Sendable, Identifiable {
    case overview
    case provider(ProviderID)

    public var id: String {
        switch self {
        case .overview:
            return "overview"
        case .provider(let provider):
            return "provider:\(provider.rawValue)"
        }
    }

    public var title: String {
        switch self {
        case .overview:
            return "Overview"
        case .provider(let provider):
            return provider.title
        }
    }

    public var systemImage: String {
        switch self {
        case .overview:
            return "square.grid.2x2"
        case .provider(.claude):
            return "bolt.horizontal"
        case .provider(.codex):
            return "terminal"
        }
    }

    public var providerID: ProviderID? {
        switch self {
        case .provider(let provider):
            return provider
        case .overview:
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
        [.overview] + self.visibleProviders.map(AppNavigationItem.provider)
    }

    public func selectNavigation(_ item: AppNavigationItem) {
        self.navigationSelection = item
        if let provider = item.providerID {
            self.sessionStore.selectedProvider = provider
            self.selectedMenuTab = provider == .claude ? .claude : .codex
            self.sessionStore.selectedMergeTab = self.selectedMenuTab
        } else if item == .overview {
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
        if !self.navigationItems.contains(self.navigationSelection) {
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
