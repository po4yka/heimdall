import Foundation

public enum LiveProviderContract {
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

    public init(
        claude: ProviderConfig,
        codex: ProviderConfig,
        mergeIcons: Bool,
        showUsedValues: Bool,
        refreshIntervalSeconds: Int,
        resetDisplayMode: ResetDisplayMode,
        checkProviderStatus: Bool,
        helperPort: Int
    ) {
        self.claude = claude
        self.codex = codex
        self.mergeIcons = mergeIcons
        self.showUsedValues = showUsedValues
        self.refreshIntervalSeconds = refreshIntervalSeconds
        self.resetDisplayMode = resetDisplayMode
        self.checkProviderStatus = checkProviderStatus
        self.helperPort = helperPort
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

public struct CostHistoryPoint: Codable, Sendable, Identifiable {
    public var day: String
    public var totalTokens: Int
    public var costUSD: Double

    public var id: String { self.day }

    public init(day: String, totalTokens: Int, costUSD: Double) {
        self.day = day
        self.totalTokens = totalTokens
        self.costUSD = costUSD
    }

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

    public init(
        contractVersion: Int = LiveProviderContract.version,
        providers: [ProviderSnapshot],
        fetchedAt: String,
        requestedProvider: String?,
        responseScope: String,
        cacheHit: Bool,
        refreshedProviders: [String]
    ) {
        self.contractVersion = contractVersion
        self.providers = providers
        self.fetchedAt = fetchedAt
        self.requestedProvider = requestedProvider
        self.responseScope = responseScope
        self.cacheHit = cacheHit
        self.refreshedProviders = refreshedProviders
    }

    enum CodingKeys: String, CodingKey {
        case contractVersion = "contract_version"
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

    public init(provider: String, summary: ProviderCostSummary) {
        self.provider = provider
        self.summary = summary
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
            auth.credentialBackend?.capitalized,
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

public struct LaneDetailProjection: Sendable, Identifiable {
    public var title: String
    public var summary: String
    public var remainingPercent: Int?
    public var resetDetail: String?
    public var paceLabel: String?

    public var id: String { self.title }

    public init(
        title: String,
        summary: String,
        remainingPercent: Int?,
        resetDetail: String?,
        paceLabel: String?
    ) {
        self.title = title
        self.summary = summary
        self.remainingPercent = remainingPercent
        self.resetDetail = resetDetail
        self.paceLabel = paceLabel
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
    public var isShowingCachedData: Bool
    public var isRefreshing: Bool
    public var error: String?
    public var globalIssueLabel: String?
    public var historyFractions: [Double]
    public var claudeFactors: [ClaudeUsageFactorSnapshot]
    public var adjunct: DashboardAdjunctSnapshot?

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
        visualState: ProviderVisualState,
        stateLabel: String,
        statusLabel: String?,
        identityLabel: String?,
        lastRefreshLabel: String,
        refreshStatusLabel: String,
        costLabel: String,
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
        adjunct: DashboardAdjunctSnapshot?
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
        self.visualState = visualState
        self.stateLabel = stateLabel
        self.statusLabel = statusLabel
        self.identityLabel = identityLabel
        self.lastRefreshLabel = lastRefreshLabel
        self.refreshStatusLabel = refreshStatusLabel
        self.costLabel = costLabel
        self.laneDetails = laneDetails
        self.creditsLabel = creditsLabel
        self.incidentLabel = incidentLabel
        self.stale = stale
        self.isShowingCachedData = isShowingCachedData
        self.isRefreshing = isRefreshing
        self.error = error
        self.globalIssueLabel = globalIssueLabel
        self.historyFractions = historyFractions
        self.claudeFactors = claudeFactors
        self.adjunct = adjunct
    }
}

public struct OverviewMenuProjection: Sendable {
    public var items: [ProviderMenuProjection]
    public var combinedCostLabel: String
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
