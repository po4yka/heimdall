import Foundation
import Observation

public enum DashboardRange: String, CaseIterable, Codable, Sendable {
    case last7d = "7d"
    case last30d = "30d"
    case last90d = "90d"
    case allTime = "all"

    var label: String {
        switch self {
        case .last7d: return "7 days"
        case .last30d: return "30 days"
        case .last90d: return "90 days"
        case .allTime: return "All"
        }
    }

    var shortLabel: String {
        switch self {
        case .last7d: return "7d"
        case .last30d: return "30d"
        case .last90d: return "90d"
        case .allTime: return "All"
        }
    }
}

public enum DashboardBucket: String, CaseIterable, Codable, Sendable {
    case day
    case week

    var label: String {
        switch self {
        case .day: return "Day"
        case .week: return "Week"
        }
    }
}

public enum ProviderScope: String, CaseIterable, Codable, Sendable {
    case both
    case claude
    case codex

    var label: String {
        switch self {
        case .both: return "Both"
        case .claude: return "Claude"
        case .codex: return "Codex"
        }
    }
}

public struct DashboardFilterGroups: OptionSet, Sendable {
    public let rawValue: Int
    public init(rawValue: Int) { self.rawValue = rawValue }

    public static let range         = DashboardFilterGroups(rawValue: 1 << 0)
    public static let bucket        = DashboardFilterGroups(rawValue: 1 << 1)
    public static let provider      = DashboardFilterGroups(rawValue: 1 << 2)
    public static let models        = DashboardFilterGroups(rawValue: 1 << 3)
    public static let projectSearch = DashboardFilterGroups(rawValue: 1 << 4)
}

// Mirrors SECTION_FILTER_GROUPS in src/ui/components/FilterBar.tsx:23
func dashboardFilterGroups(for tab: AppNavigationItem) -> DashboardFilterGroups {
    switch tab {
    case .overview:   return [.range, .bucket]
    case .today:      return []
    case .activity:   return [.range, .bucket, .provider, .models]
    case .agents:     return [.range, .provider]
    case .costModels: return [.range, .bucket, .provider, .models]
    case .sessions:   return [.range, .provider, .models, .projectSearch]
    case .projects:   return [.projectSearch]
    case .skills, .instructions, .mcpServers, .liveMonitor, .provider, .toolErrors: return []
    }
}

@MainActor
@Observable
public final class DashboardFiltersModel {
    public var range: DashboardRange = .last30d
    public var bucket: DashboardBucket = .day
    public var provider: ProviderScope = .both
    public var selectedModels: Set<String> = []
    public var projectSearch: String = ""

    public var availableModels: [String] = []

    public init() {}

    public func activeGroups(for tab: AppNavigationItem) -> DashboardFilterGroups {
        dashboardFilterGroups(for: tab)
    }

    public func snapshot() -> SavedFilterState {
        SavedFilterState(
            range: range,
            bucket: bucket,
            provider: provider,
            selectedModels: Array(selectedModels),
            projectSearch: projectSearch
        )
    }

    public func apply(_ state: SavedFilterState) {
        if let r = state.range { range = r }
        if let b = state.bucket { bucket = b }
        if let p = state.provider { provider = p }
        selectedModels = Set(state.selectedModels)
        projectSearch = state.projectSearch
    }
}
