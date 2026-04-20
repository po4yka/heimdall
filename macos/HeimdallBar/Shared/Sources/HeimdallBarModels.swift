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

public struct ProviderSourceAttempt: Codable, Sendable, Identifiable {
    public var source: String
    public var outcome: String
    public var message: String?

    public var id: String { "\(self.source):\(self.outcome):\(self.message ?? "")" }
}

public struct AuthRecoveryAction: Codable, Sendable, Identifiable {
    public var label: String
    public var actionID: String
    public var command: String?
    public var detail: String?

    public var id: String { self.actionID }

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
    public var lastRefresh: String
    public var stale: Bool
    public var error: String?

    public var id: String { self.provider }
    public var providerID: ProviderID? { ProviderID(rawValue: self.provider) }

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
        case lastRefresh = "last_refresh"
        case stale
        case error
    }
}

public struct ProviderSnapshotEnvelope: Codable, Sendable {
    public var providers: [ProviderSnapshot]
    public var fetchedAt: String
    public var requestedProvider: String?
    public var responseScope: String
    public var cacheHit: Bool
    public var refreshedProviders: [String]

    enum CodingKeys: String, CodingKey {
        case providers
        case fetchedAt = "fetched_at"
        case requestedProvider = "requested_provider"
        case responseScope = "response_scope"
        case cacheHit = "cache_hit"
        case refreshedProviders = "refreshed_providers"
    }
}

public struct CostSummaryEnvelope: Codable, Sendable {
    public var provider: String
    public var summary: ProviderCostSummary
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
}

public struct ImportedBrowserSession: Codable, Sendable {
    public var provider: ProviderID
    public var browserSource: BrowserSource
    public var profileName: String
    public var importedAt: String
    public var sourcePath: String
    public var storageKind: String
    public var cookies: [ImportedSessionCookie]
    public var loginRequired: Bool
    public var expired: Bool
    public var lastValidatedAt: String?

    public var authCookieCount: Int {
        self.cookies.count
    }
}

public struct DashboardWebQuotaLane: Codable, Sendable, Identifiable {
    public var title: String
    public var window: ProviderRateWindow

    public var id: String { self.title }
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
}

public struct ProviderPresentationState: Sendable {
    public var provider: ProviderID
    public var snapshot: ProviderSnapshot?
    public var adjunct: DashboardAdjunctSnapshot?
    public var resolution: ProviderSourceResolution

    public var auth: ProviderAuthHealth? {
        self.snapshot?.auth
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
            auth.credentialBackend?.capitalized,
        ]
        .compactMap { $0 }
        return parts.isEmpty ? nil : parts.joined(separator: " · ")
    }
}

public struct WidgetProviderEntry: Codable, Sendable, Identifiable {
    public var provider: ProviderID
    public var title: String
    public var visualState: ProviderVisualState
    public var statusLabel: String
    public var refreshLabel: String
    public var usageLines: [WidgetUsageLine]
    public var creditsLabel: String?
    public var warningLabel: String?
    public var unavailableLabel: String?
    public var loginRequired: Bool
    public var historyFractions: [Double]
    public var costSummary: ProviderCostSummary
    public var todayCostLabel: String
    public var last30DaysCostLabel: String
    public var todayTokensLabel: String
    public var activityLabel: String
    public var sourceLabel: String
    public var updatedAt: String

    public var id: String { self.provider.rawValue }
}

public struct WidgetSnapshot: Codable, Sendable {
    public var generatedAt: String
    public var refreshIntervalSeconds: Int
    public var entries: [WidgetProviderEntry]

    public init(generatedAt: String, refreshIntervalSeconds: Int, entries: [WidgetProviderEntry]) {
        self.generatedAt = generatedAt
        self.refreshIntervalSeconds = refreshIntervalSeconds
        self.entries = entries
    }
}

public struct WidgetUsageLine: Codable, Sendable, Identifiable {
    public var title: String
    public var valueLabel: String
    public var detailLabel: String?
    public var fraction: Double?

    public var id: String { self.title }
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

public struct LaneDetailProjection: Sendable, Identifiable {
    public var title: String
    public var summary: String
    public var remainingPercent: Int?
    public var resetDetail: String?
    public var paceLabel: String?

    public var id: String { self.title }
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
    public var visualState: ProviderVisualState
    public var stateLabel: String
    public var statusLabel: String?
    public var identityLabel: String?
    public var lastRefreshLabel: String
    public var refreshStatusLabel: String
    public var costLabel: String
    public var laneDetails: [LaneDetailProjection]
    public var creditsLabel: String?
    public var incidentLabel: String?
    public var stale: Bool
    public var isRefreshing: Bool
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
    public var activitySummaryLabel: String
    public var historyFractions: [Double]
    public var warningLabels: [String]
    public var isRefreshing: Bool
    public var refreshStatusLabel: String
}
