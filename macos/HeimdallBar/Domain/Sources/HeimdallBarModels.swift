import Foundation

public enum LiveProviderContract {
    public static let version = 1
}

public enum LiveMonitorContract {
    public static let version = 1
}

public enum MobileSnapshotContract {
    public static let version = 1
}

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
    public var localNotificationsEnabled: Bool
    public var helperPort: Int

    public static let `default` = HeimdallBarConfig(
        claude: ProviderConfig(enabled: true, source: .oauth, cookieSource: .auto, dashboardExtrasEnabled: false),
        codex: ProviderConfig(enabled: true, source: .auto, cookieSource: .auto, dashboardExtrasEnabled: false),
        mergeIcons: true,
        showUsedValues: false,
        refreshIntervalSeconds: 300,
        resetDisplayMode: .countdown,
        checkProviderStatus: true,
        localNotificationsEnabled: false,
        helperPort: 8787
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
        helperPort: Int
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
    }

    enum CodingKeys: String, CodingKey {
        case claude
        case codex
        case mergeIcons = "merge_icons"
        case showUsedValues = "show_used_values"
        case refreshIntervalSeconds = "refresh_interval_seconds"
        case resetDisplayMode = "reset_display_mode"
        case checkProviderStatus = "check_provider_status"
        case localNotificationsEnabled = "local_notifications_enabled"
        case helperPort = "helper_port"
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        self.claude = try container.decodeIfPresent(ProviderConfig.self, forKey: .claude) ?? HeimdallBarConfig.default.claude
        self.codex = try container.decodeIfPresent(ProviderConfig.self, forKey: .codex) ?? HeimdallBarConfig.default.codex
        self.mergeIcons = try container.decodeIfPresent(Bool.self, forKey: .mergeIcons) ?? HeimdallBarConfig.default.mergeIcons
        self.showUsedValues = try container.decodeIfPresent(Bool.self, forKey: .showUsedValues) ?? HeimdallBarConfig.default.showUsedValues
        self.refreshIntervalSeconds = try container.decodeIfPresent(Int.self, forKey: .refreshIntervalSeconds) ?? HeimdallBarConfig.default.refreshIntervalSeconds
        self.resetDisplayMode = try container.decodeIfPresent(ResetDisplayMode.self, forKey: .resetDisplayMode) ?? HeimdallBarConfig.default.resetDisplayMode
        self.checkProviderStatus = try container.decodeIfPresent(Bool.self, forKey: .checkProviderStatus) ?? HeimdallBarConfig.default.checkProviderStatus
        self.localNotificationsEnabled = try container.decodeIfPresent(Bool.self, forKey: .localNotificationsEnabled) ?? false
        self.helperPort = try container.decodeIfPresent(Int.self, forKey: .helperPort) ?? HeimdallBarConfig.default.helperPort
    }

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

    public init(
        usedPercent: Double,
        resetsAt: String?,
        resetsInMinutes: Int?,
        windowMinutes: Int?,
        resetLabel: String?
    ) {
        self.usedPercent = usedPercent
        self.resetsAt = resetsAt
        self.resetsInMinutes = resetsInMinutes
        self.windowMinutes = windowMinutes
        self.resetLabel = resetLabel
    }

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

    public init(
        provider: String,
        accountEmail: String?,
        accountOrganization: String?,
        loginMethod: String?,
        plan: String?
    ) {
        self.provider = provider
        self.accountEmail = accountEmail
        self.accountOrganization = accountOrganization
        self.loginMethod = loginMethod
        self.plan = plan
    }

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

    public init(indicator: String, description: String, pageURL: String) {
        self.indicator = indicator
        self.description = description
        self.pageURL = pageURL
    }

    enum CodingKeys: String, CodingKey {
        case indicator
        case description
        case pageURL = "page_url"
    }
}

/// Per-category breakdown of token usage. Optional on the wire so that
/// Mac app builds keep decoding against older helpers that don't emit it.
public struct TokenBreakdown: Codable, Sendable, Hashable {
    public var input: Int
    public var output: Int
    public var cacheRead: Int
    public var cacheCreation: Int
    public var reasoningOutput: Int

    public init(
        input: Int = 0,
        output: Int = 0,
        cacheRead: Int = 0,
        cacheCreation: Int = 0,
        reasoningOutput: Int = 0
    ) {
        self.input = input
        self.output = output
        self.cacheRead = cacheRead
        self.cacheCreation = cacheCreation
        self.reasoningOutput = reasoningOutput
    }

    public var total: Int {
        self.input + self.output + self.cacheRead + self.cacheCreation + self.reasoningOutput
    }

    public var isEmpty: Bool { self.total == 0 }

    enum CodingKeys: String, CodingKey {
        case input
        case output
        case cacheRead = "cache_read"
        case cacheCreation = "cache_creation"
        case reasoningOutput = "reasoning_output"
    }
}

public struct CostHistoryPoint: Codable, Sendable, Identifiable {
    public var day: String
    public var totalTokens: Int
    public var costUSD: Double
    public var breakdown: TokenBreakdown?

    public var id: String { self.day }

    public init(
        day: String,
        totalTokens: Int,
        costUSD: Double,
        breakdown: TokenBreakdown? = nil
    ) {
        self.day = day
        self.totalTokens = totalTokens
        self.costUSD = costUSD
        self.breakdown = breakdown
    }

    enum CodingKeys: String, CodingKey {
        case day
        case totalTokens = "total_tokens"
        case costUSD = "cost_usd"
        case breakdown
    }
}

public struct ProviderCostSummary: Codable, Sendable {
    public var todayTokens: Int
    public var todayCostUSD: Double
    public var last30DaysTokens: Int
    public var last30DaysCostUSD: Double
    public var daily: [CostHistoryPoint]
    public var todayBreakdown: TokenBreakdown?
    public var last30DaysBreakdown: TokenBreakdown?
    public var cacheHitRateToday: Double?
    public var cacheHitRate30d: Double?
    public var cacheSavings30dUSD: Double?
    public var byModel: [ProviderModelRow]
    public var byProject: [ProviderProjectRow]
    public var byTool: [ProviderToolRow]
    public var byMcp: [ProviderMcpRow]
    public var hourlyActivity: [ProviderHourlyBucket]
    public var activityHeatmap: [ProviderHeatmapCell]
    public var recentSessions: [ProviderSession]
    public var subagentBreakdown: ProviderSubagentBreakdown?
    public var versionBreakdown: [ProviderVersionRow]

    public init(
        todayTokens: Int,
        todayCostUSD: Double,
        last30DaysTokens: Int,
        last30DaysCostUSD: Double,
        daily: [CostHistoryPoint],
        todayBreakdown: TokenBreakdown? = nil,
        last30DaysBreakdown: TokenBreakdown? = nil,
        cacheHitRateToday: Double? = nil,
        cacheHitRate30d: Double? = nil,
        cacheSavings30dUSD: Double? = nil,
        byModel: [ProviderModelRow] = [],
        byProject: [ProviderProjectRow] = [],
        byTool: [ProviderToolRow] = [],
        byMcp: [ProviderMcpRow] = [],
        hourlyActivity: [ProviderHourlyBucket] = [],
        activityHeatmap: [ProviderHeatmapCell] = [],
        recentSessions: [ProviderSession] = [],
        subagentBreakdown: ProviderSubagentBreakdown? = nil,
        versionBreakdown: [ProviderVersionRow] = []
    ) {
        self.todayTokens = todayTokens
        self.todayCostUSD = todayCostUSD
        self.last30DaysTokens = last30DaysTokens
        self.last30DaysCostUSD = last30DaysCostUSD
        self.daily = daily
        self.todayBreakdown = todayBreakdown
        self.last30DaysBreakdown = last30DaysBreakdown
        self.cacheHitRateToday = cacheHitRateToday
        self.cacheHitRate30d = cacheHitRate30d
        self.cacheSavings30dUSD = cacheSavings30dUSD
        self.byModel = byModel
        self.byProject = byProject
        self.byTool = byTool
        self.byMcp = byMcp
        self.hourlyActivity = hourlyActivity
        self.activityHeatmap = activityHeatmap
        self.recentSessions = recentSessions
        self.subagentBreakdown = subagentBreakdown
        self.versionBreakdown = versionBreakdown
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        self.todayTokens = try container.decode(Int.self, forKey: .todayTokens)
        self.todayCostUSD = try container.decode(Double.self, forKey: .todayCostUSD)
        self.last30DaysTokens = try container.decode(Int.self, forKey: .last30DaysTokens)
        self.last30DaysCostUSD = try container.decode(Double.self, forKey: .last30DaysCostUSD)
        self.daily = try container.decode([CostHistoryPoint].self, forKey: .daily)
        self.todayBreakdown = try container.decodeIfPresent(TokenBreakdown.self, forKey: .todayBreakdown)
        self.last30DaysBreakdown = try container.decodeIfPresent(TokenBreakdown.self, forKey: .last30DaysBreakdown)
        self.cacheHitRateToday = try container.decodeIfPresent(Double.self, forKey: .cacheHitRateToday)
        self.cacheHitRate30d = try container.decodeIfPresent(Double.self, forKey: .cacheHitRate30d)
        self.cacheSavings30dUSD = try container.decodeIfPresent(Double.self, forKey: .cacheSavings30dUSD)
        self.byModel = try container.decodeIfPresent([ProviderModelRow].self, forKey: .byModel) ?? []
        self.byProject = try container.decodeIfPresent([ProviderProjectRow].self, forKey: .byProject) ?? []
        self.byTool = try container.decodeIfPresent([ProviderToolRow].self, forKey: .byTool) ?? []
        self.byMcp = try container.decodeIfPresent([ProviderMcpRow].self, forKey: .byMcp) ?? []
        self.hourlyActivity = try container.decodeIfPresent([ProviderHourlyBucket].self, forKey: .hourlyActivity) ?? []
        self.activityHeatmap = try container.decodeIfPresent([ProviderHeatmapCell].self, forKey: .activityHeatmap) ?? []
        self.recentSessions = try container.decodeIfPresent([ProviderSession].self, forKey: .recentSessions) ?? []
        self.subagentBreakdown = try container.decodeIfPresent(ProviderSubagentBreakdown.self, forKey: .subagentBreakdown)
        self.versionBreakdown = try container.decodeIfPresent([ProviderVersionRow].self, forKey: .versionBreakdown) ?? []
    }

    enum CodingKeys: String, CodingKey {
        case todayTokens = "today_tokens"
        case todayCostUSD = "today_cost_usd"
        case last30DaysTokens = "last_30_days_tokens"
        case last30DaysCostUSD = "last_30_days_cost_usd"
        case daily
        case todayBreakdown = "today_breakdown"
        case last30DaysBreakdown = "last_30_days_breakdown"
        case cacheHitRateToday = "cache_hit_rate_today"
        case cacheHitRate30d = "cache_hit_rate_30d"
        case cacheSavings30dUSD = "cache_savings_30d_usd"
        case byModel = "by_model"
        case byProject = "by_project"
        case byTool = "by_tool"
        case byMcp = "by_mcp"
        case hourlyActivity = "hourly_activity"
        case activityHeatmap = "activity_heatmap"
        case recentSessions = "recent_sessions"
        case subagentBreakdown = "subagent_breakdown"
        case versionBreakdown = "version_breakdown"
    }
}

public struct ProviderModelRow: Codable, Sendable, Hashable, Identifiable {
    public let model: String
    public let costUSD: Double
    public let input: Int
    public let output: Int
    public let cacheRead: Int
    public let cacheCreation: Int
    public let reasoningOutput: Int
    public let turns: Int

    public var id: String { self.model }

    public init(
        model: String,
        costUSD: Double,
        input: Int,
        output: Int,
        cacheRead: Int,
        cacheCreation: Int,
        reasoningOutput: Int,
        turns: Int
    ) {
        self.model = model
        self.costUSD = costUSD
        self.input = input
        self.output = output
        self.cacheRead = cacheRead
        self.cacheCreation = cacheCreation
        self.reasoningOutput = reasoningOutput
        self.turns = turns
    }

    enum CodingKeys: String, CodingKey {
        case model
        case costUSD = "cost_usd"
        case input
        case output
        case cacheRead = "cache_read"
        case cacheCreation = "cache_creation"
        case reasoningOutput = "reasoning_output"
        case turns
    }
}

public struct ProviderProjectRow: Codable, Sendable, Hashable, Identifiable {
    public let project: String
    public let displayName: String
    public let costUSD: Double
    public let turns: Int
    public let sessions: Int

    public var id: String { self.project }

    public init(
        project: String,
        displayName: String,
        costUSD: Double,
        turns: Int,
        sessions: Int
    ) {
        self.project = project
        self.displayName = displayName
        self.costUSD = costUSD
        self.turns = turns
        self.sessions = sessions
    }

    enum CodingKeys: String, CodingKey {
        case project
        case displayName = "display_name"
        case costUSD = "cost_usd"
        case turns
        case sessions
    }
}

public struct ProviderToolRow: Codable, Sendable, Hashable, Identifiable {
    public let toolName: String
    public let category: String?
    public let mcpServer: String?
    public let invocations: Int
    public let errors: Int
    public let turnsUsed: Int
    public let sessionsUsed: Int

    public var id: String { "\(self.mcpServer ?? "_")/\(self.toolName)" }

    public init(
        toolName: String,
        category: String?,
        mcpServer: String?,
        invocations: Int,
        errors: Int,
        turnsUsed: Int,
        sessionsUsed: Int
    ) {
        self.toolName = toolName
        self.category = category
        self.mcpServer = mcpServer
        self.invocations = invocations
        self.errors = errors
        self.turnsUsed = turnsUsed
        self.sessionsUsed = sessionsUsed
    }

    enum CodingKeys: String, CodingKey {
        case toolName = "tool_name"
        case category
        case mcpServer = "mcp_server"
        case invocations
        case errors
        case turnsUsed = "turns_used"
        case sessionsUsed = "sessions_used"
    }
}

public struct ProviderMcpRow: Codable, Sendable, Hashable, Identifiable {
    public let server: String
    public let invocations: Int
    public let toolsUsed: Int
    public let sessionsUsed: Int

    public var id: String { self.server }

    public init(
        server: String,
        invocations: Int,
        toolsUsed: Int,
        sessionsUsed: Int
    ) {
        self.server = server
        self.invocations = invocations
        self.toolsUsed = toolsUsed
        self.sessionsUsed = sessionsUsed
    }

    enum CodingKeys: String, CodingKey {
        case server
        case invocations
        case toolsUsed = "tools_used"
        case sessionsUsed = "sessions_used"
    }
}

public struct ProviderHourlyBucket: Codable, Sendable, Hashable, Identifiable {
    public let hour: Int
    public let turns: Int
    public let costUSD: Double
    public let tokens: Int
    public var id: Int { self.hour }

    public init(hour: Int, turns: Int, costUSD: Double, tokens: Int) {
        self.hour = hour; self.turns = turns; self.costUSD = costUSD; self.tokens = tokens
    }

    enum CodingKeys: String, CodingKey {
        case hour, turns, tokens
        case costUSD = "cost_usd"
    }
}

public struct ProviderHeatmapCell: Codable, Sendable, Hashable, Identifiable {
    public let dayOfWeek: Int  // 0..6, 0 = Sunday
    public let hour: Int       // 0..23
    public let turns: Int
    public var id: String { "\(self.dayOfWeek)-\(self.hour)" }

    public init(dayOfWeek: Int, hour: Int, turns: Int) {
        self.dayOfWeek = dayOfWeek; self.hour = hour; self.turns = turns
    }

    enum CodingKeys: String, CodingKey {
        case dayOfWeek = "day_of_week"
        case hour, turns
    }
}

public struct ProviderSession: Codable, Sendable, Hashable, Identifiable {
    public let sessionID: String
    public let displayName: String
    public let startedAt: String
    public let durationMinutes: Int
    public let turns: Int
    public let costUSD: Double
    public let model: String?
    public var id: String { self.sessionID }

    public init(
        sessionID: String, displayName: String, startedAt: String,
        durationMinutes: Int, turns: Int, costUSD: Double, model: String?
    ) {
        self.sessionID = sessionID; self.displayName = displayName; self.startedAt = startedAt
        self.durationMinutes = durationMinutes; self.turns = turns; self.costUSD = costUSD; self.model = model
    }

    enum CodingKeys: String, CodingKey {
        case sessionID = "session_id"
        case displayName = "display_name"
        case startedAt = "started_at"
        case durationMinutes = "duration_minutes"
        case turns
        case costUSD = "cost_usd"
        case model
    }
}

public struct ProviderSubagentBreakdown: Codable, Sendable, Hashable {
    public let totalTurns: Int
    public let totalCostUSD: Double
    public let sessionCount: Int
    public let agentCount: Int

    public init(totalTurns: Int, totalCostUSD: Double, sessionCount: Int, agentCount: Int) {
        self.totalTurns = totalTurns; self.totalCostUSD = totalCostUSD
        self.sessionCount = sessionCount; self.agentCount = agentCount
    }

    enum CodingKeys: String, CodingKey {
        case totalTurns = "total_turns"
        case totalCostUSD = "total_cost_usd"
        case sessionCount = "session_count"
        case agentCount = "agent_count"
    }
}

public struct ProviderVersionRow: Codable, Sendable, Hashable, Identifiable {
    public let version: String
    public let turns: Int
    public let sessions: Int
    public let costUSD: Double
    public var id: String { self.version }

    public init(version: String, turns: Int, sessions: Int, costUSD: Double) {
        self.version = version; self.turns = turns; self.sessions = sessions; self.costUSD = costUSD
    }

    enum CodingKeys: String, CodingKey {
        case version, turns, sessions
        case costUSD = "cost_usd"
    }
}

public struct ClaudeUsageFactorSnapshot: Codable, Sendable, Identifiable {
    public var factorKey: String
    public var displayLabel: String
    public var percent: Double
    public var adviceText: String

    public var id: String { self.factorKey }

    public init(factorKey: String, displayLabel: String, percent: Double, adviceText: String) {
        self.factorKey = factorKey
        self.displayLabel = displayLabel
        self.percent = percent
        self.adviceText = adviceText
    }

    enum CodingKeys: String, CodingKey {
        case factorKey = "factor_key"
        case displayLabel = "display_label"
        case percent
        case adviceText = "advice_text"
    }
}

public struct ClaudeUsageSnapshotPayload: Codable, Sendable {
    public var factors: [ClaudeUsageFactorSnapshot]

    public init(factors: [ClaudeUsageFactorSnapshot]) {
        self.factors = factors
    }
}

public struct ProviderSourceAttempt: Codable, Sendable, Identifiable {
    public var source: String
    public var outcome: String
    public var message: String?

    public var id: String { "\(self.source):\(self.outcome):\(self.message ?? "")" }

    public init(source: String, outcome: String, message: String?) {
        self.source = source
        self.outcome = outcome
        self.message = message
    }
}

public struct AuthRecoveryAction: Codable, Sendable, Identifiable {
    public var label: String
    public var actionID: String
    public var command: String?
    public var detail: String?

    public var id: String { self.actionID }

    public init(
        label: String,
        actionID: String,
        command: String?,
        detail: String?
    ) {
        self.label = label
        self.actionID = actionID
        self.command = command
        self.detail = detail
    }

    enum CodingKeys: String, CodingKey {
        case label
        case actionID = "action_id"
        case command
        case detail
    }
}

public struct ProviderAuthHealth: Codable, Sendable {
    public var loginMethod: String?
    public var credentialBackend: String?
    public var authMode: String?
    public var isAuthenticated: Bool
    public var isRefreshable: Bool
    public var isSourceCompatible: Bool
    public var requiresRelogin: Bool
    public var managedRestriction: String?
    public var diagnosticCode: String?
    public var failureReason: String?
    public var lastValidatedAt: String?
    public var recoveryActions: [AuthRecoveryAction]

    public init(
        loginMethod: String?,
        credentialBackend: String?,
        authMode: String?,
        isAuthenticated: Bool,
        isRefreshable: Bool,
        isSourceCompatible: Bool,
        requiresRelogin: Bool,
        managedRestriction: String?,
        diagnosticCode: String?,
        failureReason: String?,
        lastValidatedAt: String?,
        recoveryActions: [AuthRecoveryAction]
    ) {
        self.loginMethod = loginMethod
        self.credentialBackend = credentialBackend
        self.authMode = authMode
        self.isAuthenticated = isAuthenticated
        self.isRefreshable = isRefreshable
        self.isSourceCompatible = isSourceCompatible
        self.requiresRelogin = requiresRelogin
        self.managedRestriction = managedRestriction
        self.diagnosticCode = diagnosticCode
        self.failureReason = failureReason
        self.lastValidatedAt = lastValidatedAt
        self.recoveryActions = recoveryActions
    }

    enum CodingKeys: String, CodingKey {
        case loginMethod = "login_method"
        case credentialBackend = "credential_backend"
        case authMode = "auth_mode"
        case isAuthenticated = "is_authenticated"
        case isRefreshable = "is_refreshable"
        case isSourceCompatible = "is_source_compatible"
        case requiresRelogin = "requires_relogin"
        case managedRestriction = "managed_restriction"
        case diagnosticCode = "diagnostic_code"
        case failureReason = "failure_reason"
        case lastValidatedAt = "last_validated_at"
        case recoveryActions = "recovery_actions"
    }
}

public struct ProviderSnapshot: Codable, Sendable, Identifiable {
    public var provider: String
    public var available: Bool
    public var sourceUsed: String
    public var lastAttemptedSource: String?
    public var resolvedViaFallback: Bool
    public var refreshDurationMs: UInt64
    public var sourceAttempts: [ProviderSourceAttempt]
    public var identity: ProviderIdentity?
    public var primary: ProviderRateWindow?
    public var secondary: ProviderRateWindow?
    public var tertiary: ProviderRateWindow?
    public var credits: Double?
    public var status: ProviderStatusSummary?
    public var auth: ProviderAuthHealth
    public var costSummary: ProviderCostSummary
    public var claudeUsage: ClaudeUsageSnapshotPayload?
    public var quotaSuggestions: QuotaSuggestions?
    public var depletionForecast: DepletionForecast?
    public var predictiveInsights: LivePredictiveInsights?
    public var lastRefresh: String
    public var stale: Bool
    public var error: String?

    public var id: String { self.provider }
    public var providerID: ProviderID? { ProviderID(rawValue: self.provider) }

    public init(
        provider: String,
        available: Bool,
        sourceUsed: String,
        lastAttemptedSource: String?,
        resolvedViaFallback: Bool,
        refreshDurationMs: UInt64,
        sourceAttempts: [ProviderSourceAttempt],
        identity: ProviderIdentity?,
        primary: ProviderRateWindow?,
        secondary: ProviderRateWindow?,
        tertiary: ProviderRateWindow?,
        credits: Double?,
        status: ProviderStatusSummary?,
        auth: ProviderAuthHealth,
        costSummary: ProviderCostSummary,
        claudeUsage: ClaudeUsageSnapshotPayload?,
        quotaSuggestions: QuotaSuggestions? = nil,
        depletionForecast: DepletionForecast? = nil,
        predictiveInsights: LivePredictiveInsights? = nil,
        lastRefresh: String,
        stale: Bool,
        error: String?
    ) {
        self.provider = provider
        self.available = available
        self.sourceUsed = sourceUsed
        self.lastAttemptedSource = lastAttemptedSource
        self.resolvedViaFallback = resolvedViaFallback
        self.refreshDurationMs = refreshDurationMs
        self.sourceAttempts = sourceAttempts
        self.identity = identity
        self.primary = primary
        self.secondary = secondary
        self.tertiary = tertiary
        self.credits = credits
        self.status = status
        self.auth = auth
        self.costSummary = costSummary
        self.claudeUsage = claudeUsage
        self.quotaSuggestions = quotaSuggestions
        self.depletionForecast = depletionForecast
        self.predictiveInsights = predictiveInsights
        self.lastRefresh = lastRefresh
        self.stale = stale
        self.error = error
    }

    enum CodingKeys: String, CodingKey {
        case provider
        case available
        case sourceUsed = "source_used"
        case lastAttemptedSource = "last_attempted_source"
        case resolvedViaFallback = "resolved_via_fallback"
        case refreshDurationMs = "refresh_duration_ms"
        case sourceAttempts = "source_attempts"
        case identity
        case primary
        case secondary
        case tertiary
        case credits
        case status
        case auth
        case costSummary = "cost_summary"
        case claudeUsage = "claude_usage"
        case quotaSuggestions = "quota_suggestions"
        case depletionForecast = "depletion_forecast"
        case predictiveInsights = "predictive_insights"
        case lastRefresh = "last_refresh"
        case stale
        case error
    }
}

public struct ProviderSnapshotEnvelope: Codable, Sendable {
    public var contractVersion: Int
    public var providers: [ProviderSnapshot]
    public var fetchedAt: String
    public var requestedProvider: String?
    public var responseScope: String
    public var cacheHit: Bool
    public var refreshedProviders: [String]
    public var localNotificationState: LocalNotificationState?

    public init(
        contractVersion: Int = LiveProviderContract.version,
        providers: [ProviderSnapshot],
        fetchedAt: String,
        requestedProvider: String?,
        responseScope: String,
        cacheHit: Bool,
        refreshedProviders: [String],
        localNotificationState: LocalNotificationState? = nil
    ) {
        self.contractVersion = contractVersion
        self.providers = providers
        self.fetchedAt = fetchedAt
        self.requestedProvider = requestedProvider
        self.responseScope = responseScope
        self.cacheHit = cacheHit
        self.refreshedProviders = refreshedProviders
        self.localNotificationState = localNotificationState
    }

    enum CodingKeys: String, CodingKey {
        case contractVersion = "contract_version"
        case providers
        case fetchedAt = "fetched_at"
        case requestedProvider = "requested_provider"
        case responseScope = "response_scope"
        case cacheHit = "cache_hit"
        case refreshedProviders = "refreshed_providers"
        case localNotificationState = "local_notification_state"
    }
}

public struct LocalNotificationCondition: Codable, Sendable, Equatable, Identifiable {
    public var id: String
    public var kind: String
    public var provider: String?
    public var serviceLabel: String
    public var isActive: Bool
    public var activationTitle: String
    public var activationBody: String
    public var recoveryTitle: String?
    public var recoveryBody: String?
    public var dayKey: String?

    public init(
        id: String,
        kind: String,
        provider: String?,
        serviceLabel: String,
        isActive: Bool,
        activationTitle: String,
        activationBody: String,
        recoveryTitle: String? = nil,
        recoveryBody: String? = nil,
        dayKey: String? = nil
    ) {
        self.id = id
        self.kind = kind
        self.provider = provider
        self.serviceLabel = serviceLabel
        self.isActive = isActive
        self.activationTitle = activationTitle
        self.activationBody = activationBody
        self.recoveryTitle = recoveryTitle
        self.recoveryBody = recoveryBody
        self.dayKey = dayKey
    }

    enum CodingKeys: String, CodingKey {
        case id
        case kind
        case provider
        case serviceLabel = "service_label"
        case isActive = "is_active"
        case activationTitle = "activation_title"
        case activationBody = "activation_body"
        case recoveryTitle = "recovery_title"
        case recoveryBody = "recovery_body"
        case dayKey = "day_key"
    }
}

public struct LocalNotificationState: Codable, Sendable, Equatable {
    public var generatedAt: String
    public var costThresholdUSD: Double?
    public var conditions: [LocalNotificationCondition]

    public init(
        generatedAt: String,
        costThresholdUSD: Double? = nil,
        conditions: [LocalNotificationCondition]
    ) {
        self.generatedAt = generatedAt
        self.costThresholdUSD = costThresholdUSD
        self.conditions = conditions
    }

    enum CodingKeys: String, CodingKey {
        case generatedAt = "generated_at"
        case costThresholdUSD = "cost_threshold_usd"
        case conditions
    }
}

public enum LiveMonitorFocus: String, Codable, CaseIterable, Sendable, Identifiable {
    case all
    case claude
    case codex

    public var id: String { self.rawValue }

    public var title: String {
        switch self {
        case .all: return "All"
        case .claude: return "Claude"
        case .codex: return "Codex"
        }
    }
}

public struct LiveMonitorFreshness: Codable, Sendable {
    public var newestProviderRefresh: String?
    public var oldestProviderRefresh: String?
    public var staleProviders: [String]
    public var hasStaleProviders: Bool
    public var refreshState: String

    public init(
        newestProviderRefresh: String? = nil,
        oldestProviderRefresh: String? = nil,
        staleProviders: [String],
        hasStaleProviders: Bool,
        refreshState: String
    ) {
        self.newestProviderRefresh = newestProviderRefresh
        self.oldestProviderRefresh = oldestProviderRefresh
        self.staleProviders = staleProviders
        self.hasStaleProviders = hasStaleProviders
        self.refreshState = refreshState
    }

    enum CodingKeys: String, CodingKey {
        case newestProviderRefresh = "newest_provider_refresh"
        case oldestProviderRefresh = "oldest_provider_refresh"
        case staleProviders = "stale_providers"
        case hasStaleProviders = "has_stale_providers"
        case refreshState = "refresh_state"
    }
}

public struct LiveMonitorBurnRate: Codable, Sendable {
    public var tokensPerMin: Double
    public var costPerHourNanos: Int
    public var tier: String?

    public init(tokensPerMin: Double, costPerHourNanos: Int, tier: String? = nil) {
        self.tokensPerMin = tokensPerMin
        self.costPerHourNanos = costPerHourNanos
        self.tier = tier
    }

    enum CodingKeys: String, CodingKey {
        case tokensPerMin = "tokens_per_min"
        case costPerHourNanos = "cost_per_hour_nanos"
        case tier
    }
}

public struct LiveMonitorProjection: Codable, Sendable {
    public var projectedCostNanos: Int
    public var projectedTokens: Int

    public init(projectedCostNanos: Int, projectedTokens: Int) {
        self.projectedCostNanos = projectedCostNanos
        self.projectedTokens = projectedTokens
    }

    enum CodingKeys: String, CodingKey {
        case projectedCostNanos = "projected_cost_nanos"
        case projectedTokens = "projected_tokens"
    }
}

public struct LiveMonitorQuota: Codable, Sendable {
    public var limitTokens: Int
    public var usedTokens: Int
    public var projectedTokens: Int
    public var currentPercent: Double
    public var projectedPercent: Double
    public var remainingTokens: Int
    public var currentSeverity: String
    public var projectedSeverity: String

    public init(
        limitTokens: Int,
        usedTokens: Int,
        projectedTokens: Int,
        currentPercent: Double,
        projectedPercent: Double,
        remainingTokens: Int,
        currentSeverity: String,
        projectedSeverity: String
    ) {
        self.limitTokens = limitTokens
        self.usedTokens = usedTokens
        self.projectedTokens = projectedTokens
        self.currentPercent = currentPercent
        self.projectedPercent = projectedPercent
        self.remainingTokens = remainingTokens
        self.currentSeverity = currentSeverity
        self.projectedSeverity = projectedSeverity
    }

    enum CodingKeys: String, CodingKey {
        case limitTokens = "limit_tokens"
        case usedTokens = "used_tokens"
        case projectedTokens = "projected_tokens"
        case currentPercent = "current_pct"
        case projectedPercent = "projected_pct"
        case remainingTokens = "remaining_tokens"
        case currentSeverity = "current_severity"
        case projectedSeverity = "projected_severity"
    }
}

public struct DepletionForecastSignal: Codable, Sendable, Identifiable, Equatable {
    public var kind: String
    public var title: String
    public var usedPercent: Double
    public var projectedPercent: Double?
    public var remainingTokens: Int?
    public var remainingPercent: Double?
    public var resetsInMinutes: Int?
    public var paceLabel: String?
    public var endTime: String?

    public var id: String { "\(self.kind):\(self.title)" }

    public init(
        kind: String,
        title: String,
        usedPercent: Double,
        projectedPercent: Double? = nil,
        remainingTokens: Int? = nil,
        remainingPercent: Double? = nil,
        resetsInMinutes: Int? = nil,
        paceLabel: String? = nil,
        endTime: String? = nil
    ) {
        self.kind = kind
        self.title = title
        self.usedPercent = usedPercent
        self.projectedPercent = projectedPercent
        self.remainingTokens = remainingTokens
        self.remainingPercent = remainingPercent
        self.resetsInMinutes = resetsInMinutes
        self.paceLabel = paceLabel
        self.endTime = endTime
    }

    enum CodingKeys: String, CodingKey {
        case kind
        case title
        case usedPercent = "used_percent"
        case projectedPercent = "projected_percent"
        case remainingTokens = "remaining_tokens"
        case remainingPercent = "remaining_percent"
        case resetsInMinutes = "resets_in_minutes"
        case paceLabel = "pace_label"
        case endTime = "end_time"
    }
}

public struct DepletionForecast: Codable, Sendable, Equatable {
    public var primarySignal: DepletionForecastSignal
    public var secondarySignals: [DepletionForecastSignal]
    public var summaryLabel: String
    public var severity: String
    public var note: String?

    public init(
        primarySignal: DepletionForecastSignal,
        secondarySignals: [DepletionForecastSignal],
        summaryLabel: String,
        severity: String,
        note: String? = nil
    ) {
        self.primarySignal = primarySignal
        self.secondarySignals = secondarySignals
        self.summaryLabel = summaryLabel
        self.severity = severity
        self.note = note
    }

    enum CodingKeys: String, CodingKey {
        case primarySignal = "primary_signal"
        case secondarySignals = "secondary_signals"
        case summaryLabel = "summary_label"
        case severity
        case note
    }
}

public struct QuotaSuggestionLevel: Codable, Sendable, Identifiable, Equatable {
    public var key: String
    public var label: String
    public var limitTokens: Int

    public var id: String { self.key }

    public init(key: String, label: String, limitTokens: Int) {
        self.key = key
        self.label = label
        self.limitTokens = limitTokens
    }

    enum CodingKeys: String, CodingKey {
        case key
        case label
        case limitTokens = "limit_tokens"
    }
}

public struct QuotaSuggestions: Codable, Sendable, Equatable {
    public var sampleCount: Int
    public var populationCount: Int?
    public var sampleStrategy: String?
    public var sampleLabel: String?
    public var recommendedKey: String
    public var levels: [QuotaSuggestionLevel]
    public var note: String?

    public init(
        sampleCount: Int,
        recommendedKey: String,
        levels: [QuotaSuggestionLevel],
        note: String? = nil,
        populationCount: Int? = nil,
        sampleStrategy: String? = nil,
        sampleLabel: String? = nil
    ) {
        self.sampleCount = sampleCount
        self.populationCount = populationCount
        self.sampleStrategy = sampleStrategy
        self.sampleLabel = sampleLabel
        self.recommendedKey = recommendedKey
        self.levels = levels
        self.note = note
    }

    enum CodingKeys: String, CodingKey {
        case sampleCount = "sample_count"
        case populationCount = "population_count"
        case sampleStrategy = "sample_strategy"
        case sampleLabel = "sample_label"
        case recommendedKey = "recommended_key"
        case levels
        case note
    }
}

public struct LivePredictivePercentiles: Codable, Sendable, Equatable {
    public var average: Double
    public var p50: Double
    public var p75: Double
    public var p90: Double
    public var p95: Double

    public init(
        average: Double,
        p50: Double,
        p75: Double,
        p90: Double,
        p95: Double
    ) {
        self.average = average
        self.p50 = p50
        self.p75 = p75
        self.p90 = p90
        self.p95 = p95
    }
}

public struct LivePredictiveRollingHourBurn: Codable, Sendable, Equatable {
    public var tokensPerMin: Double
    public var costPerHourNanos: Int
    public var coverageMinutes: Double
    public var tier: String?

    public init(
        tokensPerMin: Double,
        costPerHourNanos: Int,
        coverageMinutes: Double,
        tier: String? = nil
    ) {
        self.tokensPerMin = tokensPerMin
        self.costPerHourNanos = costPerHourNanos
        self.coverageMinutes = coverageMinutes
        self.tier = tier
    }

    enum CodingKeys: String, CodingKey {
        case tokensPerMin = "tokens_per_min"
        case costPerHourNanos = "cost_per_hour_nanos"
        case coverageMinutes = "coverage_minutes"
        case tier
    }
}

public struct LivePredictiveHistoricalEnvelope: Codable, Sendable, Equatable {
    public var sampleCount: Int
    public var tokens: LivePredictivePercentiles
    public var costUSD: LivePredictivePercentiles
    public var turns: LivePredictivePercentiles

    public init(
        sampleCount: Int,
        tokens: LivePredictivePercentiles,
        costUSD: LivePredictivePercentiles,
        turns: LivePredictivePercentiles
    ) {
        self.sampleCount = sampleCount
        self.tokens = tokens
        self.costUSD = costUSD
        self.turns = turns
    }

    enum CodingKeys: String, CodingKey {
        case sampleCount = "sample_count"
        case tokens
        case costUSD = "cost_usd"
        case turns
    }
}

public struct LivePredictiveLimitHitAnalysis: Codable, Sendable, Equatable {
    public var sampleCount: Int
    public var hitCount: Int
    public var hitRate: Double
    public var thresholdTokens: Int?
    public var thresholdPercent: Double?
    public var activeCurrentHit: Bool?
    public var activeProjectedHit: Bool?
    public var riskLevel: String
    public var summaryLabel: String

    public init(
        sampleCount: Int,
        hitCount: Int,
        hitRate: Double,
        thresholdTokens: Int? = nil,
        thresholdPercent: Double? = nil,
        activeCurrentHit: Bool? = nil,
        activeProjectedHit: Bool? = nil,
        riskLevel: String,
        summaryLabel: String
    ) {
        self.sampleCount = sampleCount
        self.hitCount = hitCount
        self.hitRate = hitRate
        self.thresholdTokens = thresholdTokens
        self.thresholdPercent = thresholdPercent
        self.activeCurrentHit = activeCurrentHit
        self.activeProjectedHit = activeProjectedHit
        self.riskLevel = riskLevel
        self.summaryLabel = summaryLabel
    }

    enum CodingKeys: String, CodingKey {
        case sampleCount = "sample_count"
        case hitCount = "hit_count"
        case hitRate = "hit_rate"
        case thresholdTokens = "threshold_tokens"
        case thresholdPercent = "threshold_percent"
        case activeCurrentHit = "active_current_hit"
        case activeProjectedHit = "active_projected_hit"
        case riskLevel = "risk_level"
        case summaryLabel = "summary_label"
    }
}

public struct LivePredictiveInsights: Codable, Sendable, Equatable {
    public var rollingHourBurn: LivePredictiveRollingHourBurn?
    public var historicalEnvelope: LivePredictiveHistoricalEnvelope?
    public var limitHitAnalysis: LivePredictiveLimitHitAnalysis?

    public init(
        rollingHourBurn: LivePredictiveRollingHourBurn? = nil,
        historicalEnvelope: LivePredictiveHistoricalEnvelope? = nil,
        limitHitAnalysis: LivePredictiveLimitHitAnalysis? = nil
    ) {
        self.rollingHourBurn = rollingHourBurn
        self.historicalEnvelope = historicalEnvelope
        self.limitHitAnalysis = limitHitAnalysis
    }

    enum CodingKeys: String, CodingKey {
        case rollingHourBurn = "rolling_hour_burn"
        case historicalEnvelope = "historical_envelope"
        case limitHitAnalysis = "limit_hit_analysis"
    }
}

public struct LiveMonitorBlock: Codable, Sendable {
    public var start: String
    public var end: String
    public var firstTimestamp: String
    public var lastTimestamp: String
    public var tokens: TokenBreakdown
    public var costNanos: Int
    public var entryCount: Int
    public var burnRate: LiveMonitorBurnRate?
    public var projection: LiveMonitorProjection?
    public var quota: LiveMonitorQuota?

    public init(
        start: String,
        end: String,
        firstTimestamp: String,
        lastTimestamp: String,
        tokens: TokenBreakdown,
        costNanos: Int,
        entryCount: Int,
        burnRate: LiveMonitorBurnRate? = nil,
        projection: LiveMonitorProjection? = nil,
        quota: LiveMonitorQuota? = nil
    ) {
        self.start = start
        self.end = end
        self.firstTimestamp = firstTimestamp
        self.lastTimestamp = lastTimestamp
        self.tokens = tokens
        self.costNanos = costNanos
        self.entryCount = entryCount
        self.burnRate = burnRate
        self.projection = projection
        self.quota = quota
    }

    enum CodingKeys: String, CodingKey {
        case start
        case end
        case firstTimestamp = "first_timestamp"
        case lastTimestamp = "last_timestamp"
        case tokens
        case costNanos = "cost_nanos"
        case entryCount = "entry_count"
        case burnRate = "burn_rate"
        case projection
        case quota
    }
}

public struct LiveMonitorContextWindow: Codable, Sendable {
    public var totalInputTokens: Int
    public var contextWindowSize: Int
    public var pct: Double
    public var severity: String
    public var sessionID: String?
    public var capturedAt: String?

    public init(
        totalInputTokens: Int,
        contextWindowSize: Int,
        pct: Double,
        severity: String,
        sessionID: String? = nil,
        capturedAt: String? = nil
    ) {
        self.totalInputTokens = totalInputTokens
        self.contextWindowSize = contextWindowSize
        self.pct = pct
        self.severity = severity
        self.sessionID = sessionID
        self.capturedAt = capturedAt
    }

    enum CodingKeys: String, CodingKey {
        case totalInputTokens = "total_input_tokens"
        case contextWindowSize = "context_window_size"
        case pct
        case severity
        case sessionID = "session_id"
        case capturedAt = "captured_at"
    }
}

public struct LiveMonitorProvider: Codable, Sendable, Identifiable {
    public var provider: String
    public var title: String
    public var visualState: String
    public var sourceLabel: String
    public var warnings: [String]
    public var identityLabel: String?
    public var primary: ProviderRateWindow?
    public var secondary: ProviderRateWindow?
    public var todayCostUSD: Double
    public var projectedWeeklySpendUSD: Double?
    public var lastRefresh: String
    public var lastRefreshLabel: String
    public var activeBlock: LiveMonitorBlock?
    public var contextWindow: LiveMonitorContextWindow?
    public var recentSession: ProviderSession?
    public var quotaSuggestions: QuotaSuggestions?
    public var depletionForecast: DepletionForecast?
    public var predictiveInsights: LivePredictiveInsights?

    public var id: String { self.provider }
    public var providerID: ProviderID? { ProviderID(rawValue: self.provider) }

    public init(
        provider: String,
        title: String,
        visualState: String,
        sourceLabel: String,
        warnings: [String],
        identityLabel: String? = nil,
        primary: ProviderRateWindow? = nil,
        secondary: ProviderRateWindow? = nil,
        todayCostUSD: Double,
        projectedWeeklySpendUSD: Double? = nil,
        lastRefresh: String,
        lastRefreshLabel: String,
        activeBlock: LiveMonitorBlock? = nil,
        contextWindow: LiveMonitorContextWindow? = nil,
        recentSession: ProviderSession? = nil,
        quotaSuggestions: QuotaSuggestions? = nil,
        depletionForecast: DepletionForecast? = nil,
        predictiveInsights: LivePredictiveInsights? = nil
    ) {
        self.provider = provider
        self.title = title
        self.visualState = visualState
        self.sourceLabel = sourceLabel
        self.warnings = warnings
        self.identityLabel = identityLabel
        self.primary = primary
        self.secondary = secondary
        self.todayCostUSD = todayCostUSD
        self.projectedWeeklySpendUSD = projectedWeeklySpendUSD
        self.lastRefresh = lastRefresh
        self.lastRefreshLabel = lastRefreshLabel
        self.activeBlock = activeBlock
        self.contextWindow = contextWindow
        self.recentSession = recentSession
        self.quotaSuggestions = quotaSuggestions
        self.depletionForecast = depletionForecast
        self.predictiveInsights = predictiveInsights
    }

    enum CodingKeys: String, CodingKey {
        case provider
        case title
        case visualState = "visual_state"
        case sourceLabel = "source_label"
        case warnings
        case identityLabel = "identity_label"
        case primary
        case secondary
        case todayCostUSD = "today_cost_usd"
        case projectedWeeklySpendUSD = "projected_weekly_spend_usd"
        case lastRefresh = "last_refresh"
        case lastRefreshLabel = "last_refresh_label"
        case activeBlock = "active_block"
        case contextWindow = "context_window"
        case recentSession = "recent_session"
        case quotaSuggestions = "quota_suggestions"
        case depletionForecast = "depletion_forecast"
        case predictiveInsights = "predictive_insights"
    }
}

public struct LiveMonitorEnvelope: Codable, Sendable {
    public var contractVersion: Int
    public var generatedAt: String
    public var defaultFocus: LiveMonitorFocus
    public var globalIssue: String?
    public var freshness: LiveMonitorFreshness
    public var providers: [LiveMonitorProvider]

    public init(
        contractVersion: Int = LiveMonitorContract.version,
        generatedAt: String,
        defaultFocus: LiveMonitorFocus,
        globalIssue: String? = nil,
        freshness: LiveMonitorFreshness,
        providers: [LiveMonitorProvider]
    ) {
        self.contractVersion = contractVersion
        self.generatedAt = generatedAt
        self.defaultFocus = defaultFocus
        self.globalIssue = globalIssue
        self.freshness = freshness
        self.providers = providers
    }

    enum CodingKeys: String, CodingKey {
        case contractVersion = "contract_version"
        case generatedAt = "generated_at"
        case defaultFocus = "default_focus"
        case globalIssue = "global_issue"
        case freshness
        case providers
    }
}

public struct CostSummaryEnvelope: Codable, Sendable {
    public var provider: String
    public var summary: ProviderCostSummary

    public init(provider: String, summary: ProviderCostSummary) {
        self.provider = provider
        self.summary = summary
    }
}

public struct MobileProviderHistorySeries: Codable, Sendable, Identifiable {
    public var provider: String
    public var daily: [CostHistoryPoint]
    public var totalTokens: Int
    public var totalCostUSD: Double

    public var id: String { self.provider }
    public var providerID: ProviderID? { ProviderID(rawValue: self.provider) }

    public init(
        provider: String,
        daily: [CostHistoryPoint],
        totalTokens: Int,
        totalCostUSD: Double
    ) {
        self.provider = provider
        self.daily = daily
        self.totalTokens = totalTokens
        self.totalCostUSD = totalCostUSD
    }

    enum CodingKeys: String, CodingKey {
        case provider
        case daily
        case totalTokens = "total_tokens"
        case totalCostUSD = "total_cost_usd"
    }
}

public struct MobileSnapshotTotals: Codable, Sendable {
    public var todayTokens: Int
    public var todayCostUSD: Double
    public var last90DaysTokens: Int
    public var last90DaysCostUSD: Double
    public var todayBreakdown: TokenBreakdown?
    public var last90DaysBreakdown: TokenBreakdown?

    public init(
        todayTokens: Int,
        todayCostUSD: Double,
        last90DaysTokens: Int,
        last90DaysCostUSD: Double,
        todayBreakdown: TokenBreakdown? = nil,
        last90DaysBreakdown: TokenBreakdown? = nil
    ) {
        self.todayTokens = todayTokens
        self.todayCostUSD = todayCostUSD
        self.last90DaysTokens = last90DaysTokens
        self.last90DaysCostUSD = last90DaysCostUSD
        self.todayBreakdown = todayBreakdown
        self.last90DaysBreakdown = last90DaysBreakdown
    }

    enum CodingKeys: String, CodingKey {
        case todayTokens = "today_tokens"
        case todayCostUSD = "today_cost_usd"
        case last90DaysTokens = "last_90_days_tokens"
        case last90DaysCostUSD = "last_90_days_cost_usd"
        case todayBreakdown = "today_breakdown"
        case last90DaysBreakdown = "last_90_days_breakdown"
    }
}

public struct MobileSnapshotFreshness: Codable, Sendable {
    public var newestProviderRefresh: String?
    public var oldestProviderRefresh: String?
    public var staleProviders: [String]
    public var hasStaleProviders: Bool

    public init(
        newestProviderRefresh: String?,
        oldestProviderRefresh: String?,
        staleProviders: [String],
        hasStaleProviders: Bool
    ) {
        self.newestProviderRefresh = newestProviderRefresh
        self.oldestProviderRefresh = oldestProviderRefresh
        self.staleProviders = staleProviders
        self.hasStaleProviders = hasStaleProviders
    }

    enum CodingKeys: String, CodingKey {
        case newestProviderRefresh = "newest_provider_refresh"
        case oldestProviderRefresh = "oldest_provider_refresh"
        case staleProviders = "stale_providers"
        case hasStaleProviders = "has_stale_providers"
    }
}

public struct MobileSnapshotEnvelope: Codable, Sendable {
    public var contractVersion: Int
    public var generatedAt: String
    public var sourceDevice: String
    public var providers: [ProviderSnapshot]
    public var history90d: [MobileProviderHistorySeries]
    public var totals: MobileSnapshotTotals
    public var freshness: MobileSnapshotFreshness

    public init(
        contractVersion: Int = MobileSnapshotContract.version,
        generatedAt: String,
        sourceDevice: String,
        providers: [ProviderSnapshot],
        history90d: [MobileProviderHistorySeries],
        totals: MobileSnapshotTotals,
        freshness: MobileSnapshotFreshness
    ) {
        self.contractVersion = contractVersion
        self.generatedAt = generatedAt
        self.sourceDevice = sourceDevice
        self.providers = providers
        self.history90d = history90d
        self.totals = totals
        self.freshness = freshness
    }

    public var providerEnvelope: ProviderSnapshotEnvelope {
        ProviderSnapshotEnvelope(
            contractVersion: LiveProviderContract.version,
            providers: self.providers,
            fetchedAt: self.generatedAt,
            requestedProvider: nil,
            responseScope: "all",
            cacheHit: false,
            refreshedProviders: self.providers.map(\.provider)
        )
    }

    enum CodingKeys: String, CodingKey {
        case contractVersion = "contract_version"
        case generatedAt = "generated_at"
        case sourceDevice = "source_device"
        case providers
        case history90d = "history_90d"
        case totals
        case freshness
    }
}

public struct ImportedSessionCookie: Codable, Sendable, Identifiable, Hashable {
    public var domain: String
    public var name: String
    public var value: String?
    public var path: String
    public var expiresAt: String?
    public var secure: Bool
    public var httpOnly: Bool

    public var id: String { "\(self.domain)|\(self.name)|\(self.path)" }

    public init(
        domain: String,
        name: String,
        value: String?,
        path: String,
        expiresAt: String?,
        secure: Bool,
        httpOnly: Bool
    ) {
        self.domain = domain
        self.name = name
        self.value = value
        self.path = path
        self.expiresAt = expiresAt
        self.secure = secure
        self.httpOnly = httpOnly
    }
}

public struct ImportedBrowserSession: Codable, Sendable {
    public var provider: ProviderID
    public var browserSource: BrowserSource
    public var profileName: String
    public var importedAt: String
    public var storageKind: String
    public var cookies: [ImportedSessionCookie]
    public var loginRequired: Bool
    public var expired: Bool
    public var lastValidatedAt: String?

    public var authCookieCount: Int {
        self.cookies.count
    }

    public init(
        provider: ProviderID,
        browserSource: BrowserSource,
        profileName: String,
        importedAt: String,
        storageKind: String,
        cookies: [ImportedSessionCookie],
        loginRequired: Bool,
        expired: Bool,
        lastValidatedAt: String?
    ) {
        self.provider = provider
        self.browserSource = browserSource
        self.profileName = profileName
        self.importedAt = importedAt
        self.storageKind = storageKind
        self.cookies = cookies
        self.loginRequired = loginRequired
        self.expired = expired
        self.lastValidatedAt = lastValidatedAt
    }
}

public struct DashboardWebQuotaLane: Codable, Sendable, Identifiable {
    public var title: String
    public var window: ProviderRateWindow

    public var id: String { self.title }

    public init(title: String, window: ProviderRateWindow) {
        self.title = title
        self.window = window
    }
}

public struct DashboardWebExtras: Codable, Sendable {
    public var signedInEmail: String?
    public var accountPlan: String?
    public var creditsRemaining: Double?
    public var creditsPurchaseURL: String?
    public var quotaLanes: [DashboardWebQuotaLane]
    public var sourceURL: String?
    public var fetchedAt: String

    public var primaryLane: ProviderRateWindow? {
        self.quotaLanes.first?.window
    }

    public var secondaryLane: ProviderRateWindow? {
        self.quotaLanes.dropFirst().first?.window
    }

    public var tertiaryLane: ProviderRateWindow? {
        self.quotaLanes.dropFirst(2).first?.window
    }

    public init(
        signedInEmail: String?,
        accountPlan: String?,
        creditsRemaining: Double?,
        creditsPurchaseURL: String?,
        quotaLanes: [DashboardWebQuotaLane],
        sourceURL: String?,
        fetchedAt: String
    ) {
        self.signedInEmail = signedInEmail
        self.accountPlan = accountPlan
        self.creditsRemaining = creditsRemaining
        self.creditsPurchaseURL = creditsPurchaseURL
        self.quotaLanes = quotaLanes
        self.sourceURL = sourceURL
        self.fetchedAt = fetchedAt
    }
}

public struct BrowserSessionImportCandidate: Codable, Sendable, Identifiable, Hashable {
    public var browserSource: BrowserSource
    public var profileName: String
    public var storePath: String
    public var storageKind: String

    public var id: String { "\(self.browserSource.rawValue)|\(self.profileName)|\(self.storePath)" }

    public var title: String {
        "\(self.browserSource.title) · \(self.profileName)"
    }

    public init(
        browserSource: BrowserSource,
        profileName: String,
        storePath: String,
        storageKind: String
    ) {
        self.browserSource = browserSource
        self.profileName = profileName
        self.storePath = storePath
        self.storageKind = storageKind
    }
}

public struct ProviderSourceResolution: Sendable {
    public var provider: ProviderID
    public var requestedSource: UsageSourcePreference
    public var effectiveSource: UsageSourcePreference?
    public var effectiveSourceDetail: String?
    public var sourceLabel: String
    public var explanation: String
    public var warnings: [String]
    public var fallbackChain: [String]
    public var usageAvailable: Bool
    public var isUnsupported: Bool
    public var requiresLogin: Bool
    public var usesFallback: Bool

    public init(
        provider: ProviderID,
        requestedSource: UsageSourcePreference,
        effectiveSource: UsageSourcePreference?,
        effectiveSourceDetail: String?,
        sourceLabel: String,
        explanation: String,
        warnings: [String],
        fallbackChain: [String],
        usageAvailable: Bool,
        isUnsupported: Bool,
        requiresLogin: Bool,
        usesFallback: Bool
    ) {
        self.provider = provider
        self.requestedSource = requestedSource
        self.effectiveSource = effectiveSource
        self.effectiveSourceDetail = effectiveSourceDetail
        self.sourceLabel = sourceLabel
        self.explanation = explanation
        self.warnings = warnings
        self.fallbackChain = fallbackChain
        self.usageAvailable = usageAvailable
        self.isUnsupported = isUnsupported
        self.requiresLogin = requiresLogin
        self.usesFallback = usesFallback
    }
}

public struct ProviderPresentationState: Sendable {
    public var provider: ProviderID
    public var snapshot: ProviderSnapshot?
    public var adjunct: DashboardAdjunctSnapshot?
    public var resolution: ProviderSourceResolution

    public var auth: ProviderAuthHealth? {
        self.snapshot?.auth
    }

    public init(
        provider: ProviderID,
        snapshot: ProviderSnapshot?,
        adjunct: DashboardAdjunctSnapshot?,
        resolution: ProviderSourceResolution
    ) {
        self.provider = provider
        self.snapshot = snapshot
        self.adjunct = adjunct
        self.resolution = resolution
    }

    public var webExtras: DashboardWebExtras? {
        self.adjunct?.webExtras
    }

    public var primary: ProviderRateWindow? {
        guard self.resolution.usageAvailable else { return nil }
        if self.resolution.effectiveSource == .web {
            return self.webExtras?.primaryLane
        }
        return self.snapshot?.primary ?? self.webExtras?.primaryLane
    }

    public var secondary: ProviderRateWindow? {
        guard self.resolution.usageAvailable else { return nil }
        if self.resolution.effectiveSource == .web {
            return self.webExtras?.secondaryLane
        }
        return self.snapshot?.secondary ?? self.webExtras?.secondaryLane
    }

    public var tertiary: ProviderRateWindow? {
        guard self.resolution.usageAvailable else { return nil }
        if self.resolution.effectiveSource == .web {
            return self.webExtras?.tertiaryLane
        }
        return self.snapshot?.tertiary ?? self.webExtras?.tertiaryLane
    }

    public var displayCredits: Double? {
        if self.resolution.effectiveSource == .web {
            return self.webExtras?.creditsRemaining ?? self.snapshot?.credits
        }
        return self.snapshot?.credits ?? self.webExtras?.creditsRemaining
    }

    public var displayIdentityLabel: String? {
        if let identity = self.snapshot?.identity {
            return [identity.accountEmail, identity.plan]
                .compactMap { $0 }
                .joined(separator: " · ")
        }
        return [self.webExtras?.signedInEmail, self.webExtras?.accountPlan]
            .compactMap { $0 }
            .joined(separator: " · ")
    }

    public var authSummaryLabel: String? {
        guard let auth else { return nil }
        let parts = [
            auth.loginMethod?.replacingOccurrences(of: "-", with: " ").capitalized,
            auth.credentialBackend?.replacingOccurrences(of: "-", with: " ").capitalized,
        ]
        .compactMap { $0 }
        return parts.isEmpty ? nil : parts.joined(separator: " · ")
    }
}

public enum WidgetIssueSeverity: String, Codable, Sendable {
    case info
    case warning
    case error
}

public struct WidgetSnapshotIssue: Codable, Sendable, Identifiable {
    public var code: String
    public var message: String
    public var severity: WidgetIssueSeverity

    public var id: String { "\(self.code):\(self.message)" }

    public init(code: String, message: String, severity: WidgetIssueSeverity) {
        self.code = code
        self.message = message
        self.severity = severity
    }
}

public struct WidgetProviderSourceSnapshot: Codable, Sendable {
    public var requested: UsageSourcePreference
    public var effective: UsageSourcePreference?
    public var detail: String?
    public var usesFallback: Bool
    public var isUnsupported: Bool
    public var usageAvailable: Bool

    public init(
        requested: UsageSourcePreference,
        effective: UsageSourcePreference?,
        detail: String?,
        usesFallback: Bool,
        isUnsupported: Bool,
        usageAvailable: Bool
    ) {
        self.requested = requested
        self.effective = effective
        self.detail = detail
        self.usesFallback = usesFallback
        self.isUnsupported = isUnsupported
        self.usageAvailable = usageAvailable
    }
}

public struct WidgetProviderFreshnessSnapshot: Codable, Sendable {
    public var visualState: ProviderVisualState
    public var available: Bool
    public var stale: Bool
    public var lastRefreshAt: String?
    public var error: String?
    public var statusIndicator: String?
    public var statusDescription: String?

    public init(
        visualState: ProviderVisualState,
        available: Bool,
        stale: Bool,
        lastRefreshAt: String?,
        error: String?,
        statusIndicator: String?,
        statusDescription: String?
    ) {
        self.visualState = visualState
        self.available = available
        self.stale = stale
        self.lastRefreshAt = lastRefreshAt
        self.error = error
        self.statusIndicator = statusIndicator
        self.statusDescription = statusDescription
    }
}

public struct WidgetProviderAuthSnapshot: Codable, Sendable {
    public var loginMethod: String?
    public var credentialBackend: String?
    public var authMode: String?
    public var isAuthenticated: Bool
    public var isSourceCompatible: Bool
    public var requiresRelogin: Bool
    public var diagnosticCode: String?
    public var failureReason: String?
    public var lastValidatedAt: String?

    public init(
        loginMethod: String?,
        credentialBackend: String?,
        authMode: String?,
        isAuthenticated: Bool,
        isSourceCompatible: Bool,
        requiresRelogin: Bool,
        diagnosticCode: String?,
        failureReason: String?,
        lastValidatedAt: String?
    ) {
        self.loginMethod = loginMethod
        self.credentialBackend = credentialBackend
        self.authMode = authMode
        self.isAuthenticated = isAuthenticated
        self.isSourceCompatible = isSourceCompatible
        self.requiresRelogin = requiresRelogin
        self.diagnosticCode = diagnosticCode
        self.failureReason = failureReason
        self.lastValidatedAt = lastValidatedAt
    }
}

public struct WidgetProviderLaneSnapshot: Codable, Sendable, Identifiable {
    public var slot: Int
    public var title: String
    public var usedPercent: Double
    public var remainingPercent: Double
    public var resetsAt: String?
    public var resetsInMinutes: Int?
    public var windowMinutes: Int?

    public var id: String { "\(self.slot):\(self.title)" }

    public init(
        slot: Int,
        title: String,
        usedPercent: Double,
        remainingPercent: Double,
        resetsAt: String?,
        resetsInMinutes: Int?,
        windowMinutes: Int?
    ) {
        self.slot = slot
        self.title = title
        self.usedPercent = usedPercent
        self.remainingPercent = remainingPercent
        self.resetsAt = resetsAt
        self.resetsInMinutes = resetsInMinutes
        self.windowMinutes = windowMinutes
    }
}

public struct WidgetProviderCostSnapshot: Codable, Sendable {
    public var todayTokens: Int
    public var todayCostUSD: Double
    public var last30DaysTokens: Int
    public var last30DaysCostUSD: Double
    public var daily: [CostHistoryPoint]

    public init(
        todayTokens: Int,
        todayCostUSD: Double,
        last30DaysTokens: Int,
        last30DaysCostUSD: Double,
        daily: [CostHistoryPoint]
    ) {
        self.todayTokens = todayTokens
        self.todayCostUSD = todayCostUSD
        self.last30DaysTokens = last30DaysTokens
        self.last30DaysCostUSD = last30DaysCostUSD
        self.daily = daily
    }
}

public struct WidgetProviderAdjunctSnapshot: Codable, Sendable {
    public var source: UsageSourcePreference
    public var isLoginRequired: Bool
    public var hasWebExtras: Bool
    public var lastUpdatedAt: String?

    public init(
        source: UsageSourcePreference,
        isLoginRequired: Bool,
        hasWebExtras: Bool,
        lastUpdatedAt: String?
    ) {
        self.source = source
        self.isLoginRequired = isLoginRequired
        self.hasWebExtras = hasWebExtras
        self.lastUpdatedAt = lastUpdatedAt
    }
}

public struct WidgetProviderSnapshot: Codable, Sendable, Identifiable {
    public var provider: ProviderID
    public var source: WidgetProviderSourceSnapshot
    public var freshness: WidgetProviderFreshnessSnapshot
    public var auth: WidgetProviderAuthSnapshot?
    public var identity: ProviderIdentity?
    public var lanes: [WidgetProviderLaneSnapshot]
    public var credits: Double?
    public var cost: WidgetProviderCostSnapshot
    public var issues: [WidgetSnapshotIssue]
    public var adjunct: WidgetProviderAdjunctSnapshot?

    public var id: String { self.provider.rawValue }

    public init(
        provider: ProviderID,
        source: WidgetProviderSourceSnapshot,
        freshness: WidgetProviderFreshnessSnapshot,
        auth: WidgetProviderAuthSnapshot?,
        identity: ProviderIdentity?,
        lanes: [WidgetProviderLaneSnapshot],
        credits: Double?,
        cost: WidgetProviderCostSnapshot,
        issues: [WidgetSnapshotIssue],
        adjunct: WidgetProviderAdjunctSnapshot?
    ) {
        self.provider = provider
        self.source = source
        self.freshness = freshness
        self.auth = auth
        self.identity = identity
        self.lanes = lanes
        self.credits = credits
        self.cost = cost
        self.issues = issues
        self.adjunct = adjunct
    }
}

public struct WidgetSnapshot: Codable, Sendable {
    public static let currentSchemaVersion = 1

    public var schemaVersion: Int
    public var generatedAt: String
    public var defaultRefreshIntervalSeconds: Int
    public var providers: [String: WidgetProviderSnapshot]
    public var issues: [WidgetSnapshotIssue]

    public init(
        schemaVersion: Int = Self.currentSchemaVersion,
        generatedAt: String,
        defaultRefreshIntervalSeconds: Int,
        providers: [String: WidgetProviderSnapshot],
        issues: [WidgetSnapshotIssue] = []
    ) {
        self.schemaVersion = schemaVersion
        self.generatedAt = generatedAt
        self.defaultRefreshIntervalSeconds = defaultRefreshIntervalSeconds
        self.providers = providers
        self.issues = issues
    }

    public func providerSnapshot(for provider: ProviderID) -> WidgetProviderSnapshot? {
        self.providers[provider.rawValue]
    }

    public var allProviders: [WidgetProviderSnapshot] {
        ProviderID.allCases.compactMap { self.providers[$0.rawValue] }
    }
}

public struct DashboardAdjunctSnapshot: Codable, Sendable {
    public var provider: ProviderID
    public var source: UsageSourcePreference
    public var headline: String
    public var detailLines: [String]
    public var webExtras: DashboardWebExtras?
    public var statusText: String?
    public var isLoginRequired: Bool
    public var lastUpdated: String?

    public init(
        provider: ProviderID,
        source: UsageSourcePreference,
        headline: String,
        detailLines: [String],
        webExtras: DashboardWebExtras? = nil,
        statusText: String? = nil,
        isLoginRequired: Bool = false,
        lastUpdated: String? = nil
    ) {
        self.provider = provider
        self.source = source
        self.headline = headline
        self.detailLines = detailLines
        self.webExtras = webExtras
        self.statusText = statusText
        self.isLoginRequired = isLoginRequired
        self.lastUpdated = lastUpdated
    }
}

public enum ProviderVisualState: String, Codable, Sendable {
    case healthy
    case refreshing
    case stale
    case degraded
    case incident
    case error
}

/// Spend-direction classification over the last 7 days. Computed from the
/// costSummary.daily points: compare the last 3 days' average vs the prior 4,
/// classify the ratio against ±10% thresholds so small jitter doesn't flap.
public enum TrendDirection: String, Sendable {
    case up
    case flat
    case down
}

public struct LaneDetailProjection: Sendable, Identifiable {
    public var title: String
    public var summary: String
    public var remainingPercent: Int?
    public var resetDetail: String?
    public var paceLabel: String?
    public var resetMinutes: Int?
    public var windowMinutes: Int?

    public var id: String { self.title }

    public init(
        title: String,
        summary: String,
        remainingPercent: Int?,
        resetDetail: String?,
        paceLabel: String?,
        resetMinutes: Int? = nil,
        windowMinutes: Int? = nil
    ) {
        self.title = title
        self.summary = summary
        self.remainingPercent = remainingPercent
        self.resetDetail = resetDetail
        self.paceLabel = paceLabel
        self.resetMinutes = resetMinutes
        self.windowMinutes = windowMinutes
    }
}

public struct ProviderMenuProjection: Sendable, Identifiable {
    public var provider: ProviderID
    public var title: String
    public var sourceLabel: String
    public var sourceExplanationLabel: String?
    public var authHeadline: String?
    public var authDetail: String?
    public var authDiagnosticCode: String?
    public var authSummaryLabel: String?
    public var authRecoveryActions: [AuthRecoveryAction]
    public var warningLabels: [String]
    public var quotaSuggestions: QuotaSuggestions?
    public var depletionForecast: DepletionForecast?
    public var predictiveInsights: LivePredictiveInsights?
    public var visualState: ProviderVisualState
    public var stateLabel: String
    public var statusLabel: String?
    public var identityLabel: String?
    public var lastRefreshLabel: String
    public var refreshStatusLabel: String
    public var costLabel: String
    public var todayCostUSD: Double?
    public var last30DaysCostUSD: Double?
    public var laneDetails: [LaneDetailProjection]
    public var creditsLabel: String?
    public var incidentLabel: String?
    public var stale: Bool
    public var isShowingCachedData: Bool
    public var isRefreshing: Bool
    public var error: String?
    public var globalIssueLabel: String?
    public var historyFractions: [Double]
    public var historyBreakdowns: [TokenBreakdown]
    /// Full cost-per-day series from the underlying snapshot (typically the
    /// trailing 30 days). Exposed so the menu can render a real 30-day chart
    /// separately from the normalized 7-day `historyFractions` strip.
    public var dailyCosts: [CostHistoryPoint]
    public var todayBreakdown: TokenBreakdown?
    public var last30DaysBreakdown: TokenBreakdown?
    public var cacheHitRateToday: Double?
    public var cacheHitRate30d: Double?
    public var cacheSavings30dUSD: Double?
    /// Projected spend for the current weekly window, linearly extrapolated
    /// from elapsed-time fraction. Nil when the weekly window data (time in
    /// window, minutes remaining) is unavailable or the fraction is too
    /// small to be meaningful (< 10%).
    public var weeklyProjectedCostUSD: Double?
    /// 7-day spend trend classification (up / flat / down).
    public var spendTrendDirection: TrendDirection?
    public var claudeFactors: [ClaudeUsageFactorSnapshot]
    public var adjunct: DashboardAdjunctSnapshot?
    public var byModel: [ProviderModelRow]
    public var byProject: [ProviderProjectRow]
    public var byTool: [ProviderToolRow]
    public var byMcp: [ProviderMcpRow]
    public var hourlyActivity: [ProviderHourlyBucket]
    public var activityHeatmap: [ProviderHeatmapCell]
    public var recentSessions: [ProviderSession]
    public var subagentBreakdown: ProviderSubagentBreakdown?
    public var versionBreakdown: [ProviderVersionRow]

    public var id: String { self.provider.rawValue }

    public init(
        provider: ProviderID,
        title: String,
        sourceLabel: String,
        sourceExplanationLabel: String?,
        authHeadline: String?,
        authDetail: String?,
        authDiagnosticCode: String?,
        authSummaryLabel: String?,
        authRecoveryActions: [AuthRecoveryAction],
        warningLabels: [String],
        quotaSuggestions: QuotaSuggestions? = nil,
        depletionForecast: DepletionForecast? = nil,
        predictiveInsights: LivePredictiveInsights? = nil,
        visualState: ProviderVisualState,
        stateLabel: String,
        statusLabel: String?,
        identityLabel: String?,
        lastRefreshLabel: String,
        refreshStatusLabel: String,
        costLabel: String,
        todayCostUSD: Double? = nil,
        last30DaysCostUSD: Double? = nil,
        laneDetails: [LaneDetailProjection],
        creditsLabel: String?,
        incidentLabel: String?,
        stale: Bool,
        isShowingCachedData: Bool,
        isRefreshing: Bool,
        error: String?,
        globalIssueLabel: String?,
        historyFractions: [Double],
        claudeFactors: [ClaudeUsageFactorSnapshot],
        adjunct: DashboardAdjunctSnapshot?,
        historyBreakdowns: [TokenBreakdown] = [],
        todayBreakdown: TokenBreakdown? = nil,
        last30DaysBreakdown: TokenBreakdown? = nil,
        cacheHitRateToday: Double? = nil,
        cacheHitRate30d: Double? = nil,
        cacheSavings30dUSD: Double? = nil,
        weeklyProjectedCostUSD: Double? = nil,
        spendTrendDirection: TrendDirection? = nil,
        dailyCosts: [CostHistoryPoint] = [],
        byModel: [ProviderModelRow] = [],
        byProject: [ProviderProjectRow] = [],
        byTool: [ProviderToolRow] = [],
        byMcp: [ProviderMcpRow] = [],
        hourlyActivity: [ProviderHourlyBucket] = [],
        activityHeatmap: [ProviderHeatmapCell] = [],
        recentSessions: [ProviderSession] = [],
        subagentBreakdown: ProviderSubagentBreakdown? = nil,
        versionBreakdown: [ProviderVersionRow] = []
    ) {
        self.provider = provider
        self.title = title
        self.sourceLabel = sourceLabel
        self.sourceExplanationLabel = sourceExplanationLabel
        self.authHeadline = authHeadline
        self.authDetail = authDetail
        self.authDiagnosticCode = authDiagnosticCode
        self.authSummaryLabel = authSummaryLabel
        self.authRecoveryActions = authRecoveryActions
        self.warningLabels = warningLabels
        self.quotaSuggestions = quotaSuggestions
        self.depletionForecast = depletionForecast
        self.predictiveInsights = predictiveInsights
        self.visualState = visualState
        self.stateLabel = stateLabel
        self.statusLabel = statusLabel
        self.identityLabel = identityLabel
        self.lastRefreshLabel = lastRefreshLabel
        self.refreshStatusLabel = refreshStatusLabel
        self.costLabel = costLabel
        self.todayCostUSD = todayCostUSD
        self.last30DaysCostUSD = last30DaysCostUSD
        self.laneDetails = laneDetails
        self.creditsLabel = creditsLabel
        self.incidentLabel = incidentLabel
        self.stale = stale
        self.isShowingCachedData = isShowingCachedData
        self.isRefreshing = isRefreshing
        self.error = error
        self.globalIssueLabel = globalIssueLabel
        self.historyFractions = historyFractions
        self.historyBreakdowns = historyBreakdowns
        self.dailyCosts = dailyCosts
        self.todayBreakdown = todayBreakdown
        self.last30DaysBreakdown = last30DaysBreakdown
        self.cacheHitRateToday = cacheHitRateToday
        self.cacheHitRate30d = cacheHitRate30d
        self.cacheSavings30dUSD = cacheSavings30dUSD
        self.weeklyProjectedCostUSD = weeklyProjectedCostUSD
        self.spendTrendDirection = spendTrendDirection
        self.claudeFactors = claudeFactors
        self.adjunct = adjunct
        self.byModel = byModel
        self.byProject = byProject
        self.byTool = byTool
        self.byMcp = byMcp
        self.hourlyActivity = hourlyActivity
        self.activityHeatmap = activityHeatmap
        self.recentSessions = recentSessions
        self.subagentBreakdown = subagentBreakdown
        self.versionBreakdown = versionBreakdown
    }
}

public struct OverviewMenuProjection: Sendable {
    public var items: [ProviderMenuProjection]
    public var combinedCostLabel: String
    public var combinedTodayCostUSD: Double
    public var refreshedAtLabel: String
    public var activitySummaryLabel: String
    public var historyFractions: [Double]
    public var warningLabels: [String]
    public var isShowingCachedData: Bool
    public var isRefreshing: Bool
    public var refreshStatusLabel: String
    public var globalIssueLabel: String?

    public init(
        items: [ProviderMenuProjection],
        combinedCostLabel: String,
        combinedTodayCostUSD: Double,
        refreshedAtLabel: String,
        activitySummaryLabel: String,
        historyFractions: [Double],
        warningLabels: [String],
        isShowingCachedData: Bool,
        isRefreshing: Bool,
        refreshStatusLabel: String,
        globalIssueLabel: String?
    ) {
        self.items = items
        self.combinedCostLabel = combinedCostLabel
        self.combinedTodayCostUSD = combinedTodayCostUSD
        self.refreshedAtLabel = refreshedAtLabel
        self.activitySummaryLabel = activitySummaryLabel
        self.historyFractions = historyFractions
        self.warningLabels = warningLabels
        self.isShowingCachedData = isShowingCachedData
        self.isRefreshing = isRefreshing
        self.refreshStatusLabel = refreshStatusLabel
        self.globalIssueLabel = globalIssueLabel
    }
}
