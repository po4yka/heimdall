import Foundation

// MARK: - Contract version namespaces

public enum LiveProviderContract {
    public static let version = 2
}

public enum LiveMonitorContract {
    public static let version = 2
}

public enum MobileSnapshotContract {
    public static let version = 1
}

public enum SyncedAggregateContract {
    public static let version = 1
}

// MARK: - Provider identity enums

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

public extension ProviderID {
    /// Counterpart to `MergeMenuTab.providerID`. Adding a new `ProviderID`
    /// case forces this exhaustive switch to be updated, preventing the
    /// silent `.codex` fallback that the previous hardcoded ternary at
    /// `RootMenuView.swift` exhibited.
    var menuTab: MergeMenuTab {
        switch self {
        case .claude:
            return .claude
        case .codex:
            return .codex
        }
    }
}

// MARK: - Source / display preferences

public enum UsageSourcePreference: String, Codable, CaseIterable, Sendable {
    case auto
    case oauth
    case web
    case cli
}

public enum BrowserSource: String, Codable, CaseIterable, Sendable, Identifiable {
    case safari
    case chrome
    case arc
    case brave

    public var id: String { self.rawValue }

    public var title: String {
        switch self {
        case .safari:
            return "Safari"
        case .chrome:
            return "Chrome"
        case .arc:
            return "Arc"
        case .brave:
            return "Brave"
        }
    }
}

public enum ResetDisplayMode: String, Codable, CaseIterable, Sendable {
    case countdown
    case absolute
}

// MARK: - Per-provider and app-wide config

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

// MARK: - Rust-mirrored config sub-types (M6 Settings parity)

public struct HeimdallDisplay: Codable, Sendable, Equatable {
    public var currency: String?
    public var locale: String?
    public var compact: Bool?

    public init(currency: String? = nil, locale: String? = nil, compact: Bool? = nil) {
        self.currency = currency
        self.locale = locale
        self.compact = compact
    }
}

public struct HeimdallOAuthConfig: Codable, Sendable, Equatable {
    public var enabled: Bool
    public var refreshInterval: UInt64

    public init(enabled: Bool = true, refreshInterval: UInt64 = 60) {
        self.enabled = enabled
        self.refreshInterval = refreshInterval
    }
}

public struct HeimdallClaudeAdminConfig: Codable, Sendable, Equatable {
    public var enabled: Bool
    public var refreshInterval: UInt64
    public var lookbackDays: Int64

    public init(enabled: Bool = true, refreshInterval: UInt64 = 300, lookbackDays: Int64 = 30) {
        self.enabled = enabled
        self.refreshInterval = refreshInterval
        self.lookbackDays = lookbackDays
    }
}

public struct HeimdallOpenAiConfig: Codable, Sendable, Equatable {
    public var enabled: Bool
    public var refreshInterval: UInt64
    public var lookbackDays: Int64

    public init(enabled: Bool = true, refreshInterval: UInt64 = 300, lookbackDays: Int64 = 30) {
        self.enabled = enabled
        self.refreshInterval = refreshInterval
        self.lookbackDays = lookbackDays
    }
}

public enum HeimdallAlertSeverity: String, Codable, Sendable, CaseIterable {
    case minor
    case major
    case critical
}

public struct HeimdallAgentStatusConfig: Codable, Sendable, Equatable {
    public var enabled: Bool
    public var refreshInterval: UInt64
    public var claudeEnabled: Bool
    public var openaiEnabled: Bool
    public var alertMinSeverity: HeimdallAlertSeverity

    public init(
        enabled: Bool = true,
        refreshInterval: UInt64 = 60,
        claudeEnabled: Bool = true,
        openaiEnabled: Bool = true,
        alertMinSeverity: HeimdallAlertSeverity = .major
    ) {
        self.enabled = enabled
        self.refreshInterval = refreshInterval
        self.claudeEnabled = claudeEnabled
        self.openaiEnabled = openaiEnabled
        self.alertMinSeverity = alertMinSeverity
    }
}

public struct HeimdallAggregatorConfig: Codable, Sendable, Equatable {
    public var enabled: Bool
    public var refreshInterval: UInt64
    public var spikeWebhook: Bool

    public init(enabled: Bool = false, refreshInterval: UInt64 = 300, spikeWebhook: Bool = true) {
        self.enabled = enabled
        self.refreshInterval = refreshInterval
        self.spikeWebhook = spikeWebhook
    }
}

public struct HeimdallBlocksConfig: Codable, Sendable, Equatable {
    public var tokenLimit: Int64?
    public var sessionLengthHours: Double?

    public init(tokenLimit: Int64? = nil, sessionLengthHours: Double? = nil) {
        self.tokenLimit = tokenLimit
        self.sessionLengthHours = sessionLengthHours
    }
}

public struct HeimdallStatuslineConfig: Codable, Sendable, Equatable {
    public var contextLowThreshold: Double
    public var contextMediumThreshold: Double
    public var burnRateNormalMax: Double
    public var burnRateModerateMax: Double

    public init(
        contextLowThreshold: Double = 0.5,
        contextMediumThreshold: Double = 0.8,
        burnRateNormalMax: Double = 4000,
        burnRateModerateMax: Double = 10000
    ) {
        self.contextLowThreshold = contextLowThreshold
        self.contextMediumThreshold = contextMediumThreshold
        self.burnRateNormalMax = burnRateNormalMax
        self.burnRateModerateMax = burnRateModerateMax
    }
}

public struct HeimdallWebhookConfig: Codable, Sendable, Equatable {
    public var url: String?
    public var costThreshold: Double?
    public var sessionDepleted: Bool
    public var agentStatus: Bool
    public var spikeWebhook: Bool
    public var capChanges: Bool
    public var agentStopReason: Bool
    public var agentStopReasonFilter: [String]?

    public init(
        url: String? = nil,
        costThreshold: Double? = nil,
        sessionDepleted: Bool = false,
        agentStatus: Bool = true,
        spikeWebhook: Bool = true,
        capChanges: Bool = true,
        agentStopReason: Bool = true,
        agentStopReasonFilter: [String]? = nil
    ) {
        self.url = url
        self.costThreshold = costThreshold
        self.sessionDepleted = sessionDepleted
        self.agentStatus = agentStatus
        self.spikeWebhook = spikeWebhook
        self.capChanges = capChanges
        self.agentStopReason = agentStopReason
        self.agentStopReasonFilter = agentStopReasonFilter
    }
}

public struct HeimdallPricingOverride: Codable, Sendable, Equatable {
    public var input: Double
    public var output: Double
    public var cacheWrite: Double?
    public var cacheRead: Double?

    public init(input: Double, output: Double, cacheWrite: Double? = nil, cacheRead: Double? = nil) {
        self.input = input
        self.output = output
        self.cacheWrite = cacheWrite
        self.cacheRead = cacheRead
    }
}

public struct HeimdallConfig: Codable, Sendable {
    public var claude: ProviderConfig
    public var codex: ProviderConfig
    public var mergeIcons: Bool
    public var showUsedValues: Bool
    public var refreshIntervalSeconds: Int
    public var resetDisplayMode: ResetDisplayMode
    public var checkProviderStatus: Bool
    public var localNotificationsEnabled: Bool
    public var helperPort: Int

    // M6: Rust-mirrored fields exposed by web Settings modal.
    public var display: HeimdallDisplay
    public var oauth: HeimdallOAuthConfig
    public var claudeAdmin: HeimdallClaudeAdminConfig
    public var openai: HeimdallOpenAiConfig
    public var agentStatus: HeimdallAgentStatusConfig
    /// JSON wire key is `status_aggregator` (Rust uses `#[serde(rename = "status_aggregator")]`).
    public var aggregator: HeimdallAggregatorConfig
    public var blocks: HeimdallBlocksConfig
    public var statusline: HeimdallStatuslineConfig
    public var webhooks: HeimdallWebhookConfig
    public var projectAliases: [String: String]
    public var pricing: [String: HeimdallPricingOverride]

    public static let `default` = HeimdallConfig(
        claude: ProviderConfig(enabled: true, source: .oauth, cookieSource: .auto, dashboardExtrasEnabled: false),
        codex: ProviderConfig(enabled: true, source: .auto, cookieSource: .auto, dashboardExtrasEnabled: false),
        mergeIcons: true,
        showUsedValues: false,
        refreshIntervalSeconds: 300,
        resetDisplayMode: .countdown,
        checkProviderStatus: true,
        localNotificationsEnabled: false,
        helperPort: 8787,
        display: HeimdallDisplay(),
        oauth: HeimdallOAuthConfig(),
        claudeAdmin: HeimdallClaudeAdminConfig(),
        openai: HeimdallOpenAiConfig(),
        agentStatus: HeimdallAgentStatusConfig(),
        aggregator: HeimdallAggregatorConfig(),
        blocks: HeimdallBlocksConfig(),
        statusline: HeimdallStatuslineConfig(),
        webhooks: HeimdallWebhookConfig(),
        projectAliases: [:],
        pricing: [:]
    )

    public init(
        claude: ProviderConfig,
        codex: ProviderConfig,
        mergeIcons: Bool,
        showUsedValues: Bool,
        refreshIntervalSeconds: Int,
        resetDisplayMode: ResetDisplayMode,
        checkProviderStatus: Bool,
        localNotificationsEnabled: Bool,
        helperPort: Int,
        display: HeimdallDisplay = HeimdallDisplay(),
        oauth: HeimdallOAuthConfig = HeimdallOAuthConfig(),
        claudeAdmin: HeimdallClaudeAdminConfig = HeimdallClaudeAdminConfig(),
        openai: HeimdallOpenAiConfig = HeimdallOpenAiConfig(),
        agentStatus: HeimdallAgentStatusConfig = HeimdallAgentStatusConfig(),
        aggregator: HeimdallAggregatorConfig = HeimdallAggregatorConfig(),
        blocks: HeimdallBlocksConfig = HeimdallBlocksConfig(),
        statusline: HeimdallStatuslineConfig = HeimdallStatuslineConfig(),
        webhooks: HeimdallWebhookConfig = HeimdallWebhookConfig(),
        projectAliases: [String: String] = [:],
        pricing: [String: HeimdallPricingOverride] = [:]
    ) {
        self.claude = claude
        self.codex = codex
        self.mergeIcons = mergeIcons
        self.showUsedValues = showUsedValues
        self.refreshIntervalSeconds = refreshIntervalSeconds
        self.resetDisplayMode = resetDisplayMode
        self.checkProviderStatus = checkProviderStatus
        self.localNotificationsEnabled = localNotificationsEnabled
        self.helperPort = helperPort
        self.display = display
        self.oauth = oauth
        self.claudeAdmin = claudeAdmin
        self.openai = openai
        self.agentStatus = agentStatus
        self.aggregator = aggregator
        self.blocks = blocks
        self.statusline = statusline
        self.webhooks = webhooks
        self.projectAliases = projectAliases
        self.pricing = pricing
    }

    // CodingKeys use camelCase raw values so they match what convertFromSnakeCase
    // produces after transforming the on-disk snake_case JSON keys.  The one
    // exception is `aggregator` whose Rust wire key is `status_aggregator` — that
    // requires a raw value of `"statusAggregator"` so the strategy maps correctly,
    // while a custom encode(to:) overrides the written key back to `status_aggregator`.
    enum CodingKeys: String, CodingKey {
        case claude
        case codex
        case mergeIcons
        case showUsedValues
        case refreshIntervalSeconds
        case resetDisplayMode
        case checkProviderStatus
        case localNotificationsEnabled
        case helperPort
        case display
        case oauth
        case claudeAdmin
        case openai
        case agentStatus
        // Rust wire key: "status_aggregator" → convertFromSnakeCase → "statusAggregator"
        case aggregator = "statusAggregator"
        case blocks
        case statusline
        case webhooks
        case projectAliases
        case pricing
    }

    // StringCodingKey lets us write a literal string key to an encoder container,
    // bypassing the encoder's keyEncodingStrategy for that one key.
    private struct StringCodingKey: CodingKey {
        var stringValue: String
        var intValue: Int? { nil }
        init(stringValue: String) { self.stringValue = stringValue }
        init?(intValue: Int) { return nil }
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let defaults = HeimdallConfig.default
        self.claude = try container.decodeIfPresent(ProviderConfig.self, forKey: .claude) ?? defaults.claude
        self.codex = try container.decodeIfPresent(ProviderConfig.self, forKey: .codex) ?? defaults.codex
        self.mergeIcons = try container.decodeIfPresent(Bool.self, forKey: .mergeIcons) ?? defaults.mergeIcons
        self.showUsedValues = try container.decodeIfPresent(Bool.self, forKey: .showUsedValues) ?? defaults.showUsedValues
        self.refreshIntervalSeconds = try container.decodeIfPresent(Int.self, forKey: .refreshIntervalSeconds) ?? defaults.refreshIntervalSeconds
        self.resetDisplayMode = try container.decodeIfPresent(ResetDisplayMode.self, forKey: .resetDisplayMode) ?? defaults.resetDisplayMode
        self.checkProviderStatus = try container.decodeIfPresent(Bool.self, forKey: .checkProviderStatus) ?? defaults.checkProviderStatus
        self.localNotificationsEnabled = try container.decodeIfPresent(Bool.self, forKey: .localNotificationsEnabled) ?? false
        self.helperPort = try container.decodeIfPresent(Int.self, forKey: .helperPort) ?? defaults.helperPort
        self.display = try container.decodeIfPresent(HeimdallDisplay.self, forKey: .display) ?? defaults.display
        self.oauth = try container.decodeIfPresent(HeimdallOAuthConfig.self, forKey: .oauth) ?? defaults.oauth
        self.claudeAdmin = try container.decodeIfPresent(HeimdallClaudeAdminConfig.self, forKey: .claudeAdmin) ?? defaults.claudeAdmin
        self.openai = try container.decodeIfPresent(HeimdallOpenAiConfig.self, forKey: .openai) ?? defaults.openai
        self.agentStatus = try container.decodeIfPresent(HeimdallAgentStatusConfig.self, forKey: .agentStatus) ?? defaults.agentStatus
        self.aggregator = try container.decodeIfPresent(HeimdallAggregatorConfig.self, forKey: .aggregator) ?? defaults.aggregator
        self.blocks = try container.decodeIfPresent(HeimdallBlocksConfig.self, forKey: .blocks) ?? defaults.blocks
        self.statusline = try container.decodeIfPresent(HeimdallStatuslineConfig.self, forKey: .statusline) ?? defaults.statusline
        self.webhooks = try container.decodeIfPresent(HeimdallWebhookConfig.self, forKey: .webhooks) ?? defaults.webhooks
        self.projectAliases = try container.decodeIfPresent([String: String].self, forKey: .projectAliases) ?? defaults.projectAliases
        self.pricing = try container.decodeIfPresent([String: HeimdallPricingOverride].self, forKey: .pricing) ?? defaults.pricing
    }

    // Custom encode so that `aggregator` is written with the Rust wire key
    // `status_aggregator` regardless of the encoder's keyEncodingStrategy.
    // All other fields use synthesised-equivalent encoding via CodingKeys.
    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(self.claude, forKey: .claude)
        try container.encode(self.codex, forKey: .codex)
        try container.encode(self.mergeIcons, forKey: .mergeIcons)
        try container.encode(self.showUsedValues, forKey: .showUsedValues)
        try container.encode(self.refreshIntervalSeconds, forKey: .refreshIntervalSeconds)
        try container.encode(self.resetDisplayMode, forKey: .resetDisplayMode)
        try container.encode(self.checkProviderStatus, forKey: .checkProviderStatus)
        try container.encode(self.localNotificationsEnabled, forKey: .localNotificationsEnabled)
        try container.encode(self.helperPort, forKey: .helperPort)
        try container.encode(self.display, forKey: .display)
        try container.encode(self.oauth, forKey: .oauth)
        try container.encode(self.claudeAdmin, forKey: .claudeAdmin)
        try container.encode(self.openai, forKey: .openai)
        try container.encode(self.agentStatus, forKey: .agentStatus)
        // Force the Rust wire key "status_aggregator" regardless of strategy.
        var rawContainer = encoder.container(keyedBy: StringCodingKey.self)
        try rawContainer.encode(self.aggregator, forKey: StringCodingKey(stringValue: "status_aggregator"))
        try container.encode(self.blocks, forKey: .blocks)
        try container.encode(self.statusline, forKey: .statusline)
        try container.encode(self.webhooks, forKey: .webhooks)
        try container.encode(self.projectAliases, forKey: .projectAliases)
        try container.encode(self.pricing, forKey: .pricing)
    }

    public func providerConfig(for provider: ProviderID) -> ProviderConfig {
        switch provider {
        case .claude: return self.claude
        case .codex: return self.codex
        }
    }
}
