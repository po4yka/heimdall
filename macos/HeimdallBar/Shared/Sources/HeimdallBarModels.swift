import Foundation

public enum ProviderID: String, Codable, CaseIterable, Sendable, Identifiable {
    case claude
    case codex

    public var id: String { self.rawValue }
    public var title: String {
        switch self {
        case .claude: return "Claude"
        case .codex: return "Codex"
        }
    }
}

public enum MergeMenuTab: String, Codable, CaseIterable, Sendable, Identifiable {
    case overview
    case claude
    case codex

    public var id: String { self.rawValue }

    public var title: String {
        switch self {
        case .overview:
            return "Overview"
        case .claude:
            return "Claude"
        case .codex:
            return "Codex"
        }
    }

    public var providerID: ProviderID? {
        switch self {
        case .overview:
            return nil
        case .claude:
            return .claude
        case .codex:
            return .codex
        }
    }
}

public enum UsageSourcePreference: String, Codable, CaseIterable, Sendable {
    case auto
    case oauth
    case web
    case cli
}

public enum ResetDisplayMode: String, Codable, CaseIterable, Sendable {
    case countdown
    case absolute
}

public struct ProviderConfig: Codable, Sendable {
    public var enabled: Bool
    public var source: UsageSourcePreference
    public var cookieSource: UsageSourcePreference
    public var dashboardExtrasEnabled: Bool

    public init(
        enabled: Bool = true,
        source: UsageSourcePreference = .auto,
        cookieSource: UsageSourcePreference = .auto,
        dashboardExtrasEnabled: Bool = false
    ) {
        self.enabled = enabled
        self.source = source
        self.cookieSource = cookieSource
        self.dashboardExtrasEnabled = dashboardExtrasEnabled
    }
}

public struct HeimdallBarConfig: Codable, Sendable {
    public var claude: ProviderConfig
    public var codex: ProviderConfig
    public var mergeIcons: Bool
    public var showUsedValues: Bool
    public var refreshIntervalSeconds: Int
    public var resetDisplayMode: ResetDisplayMode
    public var checkProviderStatus: Bool
    public var helperPort: Int

    public static let `default` = HeimdallBarConfig(
        claude: ProviderConfig(enabled: true, source: .oauth, cookieSource: .auto, dashboardExtrasEnabled: false),
        codex: ProviderConfig(enabled: true, source: .auto, cookieSource: .auto, dashboardExtrasEnabled: false),
        mergeIcons: true,
        showUsedValues: false,
        refreshIntervalSeconds: 300,
        resetDisplayMode: .countdown,
        checkProviderStatus: true,
        helperPort: 8787
    )

    public func providerConfig(for provider: ProviderID) -> ProviderConfig {
        switch provider {
        case .claude: return self.claude
        case .codex: return self.codex
        }
    }
}

public struct ProviderRateWindow: Codable, Sendable {
    public var usedPercent: Double
    public var resetsAt: String?
    public var resetsInMinutes: Int?
    public var windowMinutes: Int?
    public var resetLabel: String?

    enum CodingKeys: String, CodingKey {
        case usedPercent = "used_percent"
        case resetsAt = "resets_at"
        case resetsInMinutes = "resets_in_minutes"
        case windowMinutes = "window_minutes"
        case resetLabel = "reset_label"
    }
}

public struct ProviderIdentity: Codable, Sendable {
    public var provider: String
    public var accountEmail: String?
    public var accountOrganization: String?
    public var loginMethod: String?
    public var plan: String?

    enum CodingKeys: String, CodingKey {
        case provider
        case accountEmail = "account_email"
        case accountOrganization = "account_organization"
        case loginMethod = "login_method"
        case plan
    }
}

public struct ProviderStatusSummary: Codable, Sendable {
    public var indicator: String
    public var description: String
    public var pageURL: String

    enum CodingKeys: String, CodingKey {
        case indicator
        case description
        case pageURL = "page_url"
    }
}

public struct CostHistoryPoint: Codable, Sendable, Identifiable {
    public var day: String
    public var totalTokens: Int
    public var costUSD: Double

    public var id: String { self.day }

    enum CodingKeys: String, CodingKey {
        case day
        case totalTokens = "total_tokens"
        case costUSD = "cost_usd"
    }
}

public struct ProviderCostSummary: Codable, Sendable {
    public var todayTokens: Int
    public var todayCostUSD: Double
    public var last30DaysTokens: Int
    public var last30DaysCostUSD: Double
    public var daily: [CostHistoryPoint]

    enum CodingKeys: String, CodingKey {
        case todayTokens = "today_tokens"
        case todayCostUSD = "today_cost_usd"
        case last30DaysTokens = "last_30_days_tokens"
        case last30DaysCostUSD = "last_30_days_cost_usd"
        case daily
    }
}

public struct ClaudeUsageFactorSnapshot: Codable, Sendable, Identifiable {
    public var factorKey: String
    public var displayLabel: String
    public var percent: Double
    public var adviceText: String

    public var id: String { self.factorKey }

    enum CodingKeys: String, CodingKey {
        case factorKey = "factor_key"
        case displayLabel = "display_label"
        case percent
        case adviceText = "advice_text"
    }
}

public struct ClaudeUsageSnapshotPayload: Codable, Sendable {
    public var factors: [ClaudeUsageFactorSnapshot]
}

public struct ProviderSnapshot: Codable, Sendable, Identifiable {
    public var provider: String
    public var available: Bool
    public var sourceUsed: String
    public var identity: ProviderIdentity?
    public var primary: ProviderRateWindow?
    public var secondary: ProviderRateWindow?
    public var tertiary: ProviderRateWindow?
    public var credits: Double?
    public var status: ProviderStatusSummary?
    public var costSummary: ProviderCostSummary
    public var claudeUsage: ClaudeUsageSnapshotPayload?
    public var lastRefresh: String
    public var stale: Bool
    public var error: String?

    public var id: String { self.provider }
    public var providerID: ProviderID? { ProviderID(rawValue: self.provider) }

    enum CodingKeys: String, CodingKey {
        case provider
        case available
        case sourceUsed = "source_used"
        case identity
        case primary
        case secondary
        case tertiary
        case credits
        case status
        case costSummary = "cost_summary"
        case claudeUsage = "claude_usage"
        case lastRefresh = "last_refresh"
        case stale
        case error
    }
}

public struct ProviderSnapshotEnvelope: Codable, Sendable {
    public var providers: [ProviderSnapshot]
    public var fetchedAt: String

    enum CodingKeys: String, CodingKey {
        case providers
        case fetchedAt = "fetched_at"
    }
}

public struct CostSummaryEnvelope: Codable, Sendable {
    public var provider: String
    public var summary: ProviderCostSummary
}

public struct WidgetProviderEntry: Codable, Sendable, Identifiable {
    public var provider: ProviderID
    public var title: String
    public var primary: ProviderRateWindow?
    public var secondary: ProviderRateWindow?
    public var credits: Double?
    public var costSummary: ProviderCostSummary
    public var updatedAt: String

    public var id: String { self.provider.rawValue }
}

public struct WidgetSnapshot: Codable, Sendable {
    public var generatedAt: String
    public var entries: [WidgetProviderEntry]

    public init(generatedAt: String, entries: [WidgetProviderEntry]) {
        self.generatedAt = generatedAt
        self.entries = entries
    }
}

public struct DashboardAdjunctSnapshot: Codable, Sendable {
    public var provider: ProviderID
    public var source: UsageSourcePreference
    public var headline: String
    public var detailLines: [String]
    public var statusText: String?
    public var isLoginRequired: Bool
    public var lastUpdated: String?

    public init(
        provider: ProviderID,
        source: UsageSourcePreference,
        headline: String,
        detailLines: [String],
        statusText: String? = nil,
        isLoginRequired: Bool = false,
        lastUpdated: String? = nil
    ) {
        self.provider = provider
        self.source = source
        self.headline = headline
        self.detailLines = detailLines
        self.statusText = statusText
        self.isLoginRequired = isLoginRequired
        self.lastUpdated = lastUpdated
    }
}

public struct ProviderMenuProjection: Sendable, Identifiable {
    public var provider: ProviderID
    public var title: String
    public var sourceLabel: String
    public var statusLabel: String?
    public var identityLabel: String?
    public var lastRefreshLabel: String
    public var costLabel: String
    public var laneSummaries: [String]
    public var creditsLabel: String?
    public var incidentLabel: String?
    public var stale: Bool
    public var error: String?
    public var historyFractions: [Double]
    public var claudeFactors: [ClaudeUsageFactorSnapshot]
    public var adjunct: DashboardAdjunctSnapshot?

    public var id: String { self.provider.rawValue }
}

public struct OverviewMenuProjection: Sendable {
    public var items: [ProviderMenuProjection]
    public var combinedCostLabel: String
    public var refreshedAtLabel: String
}
