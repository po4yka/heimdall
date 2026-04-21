import HeimdallDomain
import Observation

@MainActor
@Observable
public final class AppSessionStore {
    public var config: HeimdallBarConfig
    public var selectedProvider: ProviderID
    public var selectedMergeTab: MergeMenuTab

    public init(
        config: HeimdallBarConfig = .default,
        selectedProvider: ProviderID = .claude,
        selectedMergeTab: MergeMenuTab = .overview
    ) {
        self.config = config
        self.selectedProvider = selectedProvider
        self.selectedMergeTab = selectedMergeTab
    }

    public var visibleProviders: [ProviderID] {
        ProviderID.allCases.filter { self.config.providerConfig(for: $0).enabled }
    }

    public var visibleTabs: [MergeMenuTab] {
        MenuProjectionBuilder.availableTabs(config: self.config)
    }
}
