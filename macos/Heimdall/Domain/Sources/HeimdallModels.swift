import Foundation

// MARK: - Browser session import types

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

// MARK: - Dashboard web extras

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

// MARK: - Source resolution and presentation state

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

// MARK: - Widget snapshot types

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

// MARK: - Dashboard adjunct snapshot

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

// MARK: - Visual state and trend

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

// MARK: - Menu projection types

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
    public var derivedDataRevision: String
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
    public var dailyByModel: [ProviderDailyModelRow]

    public var id: String { self.provider.rawValue }

    public init(
        provider: ProviderID,
        derivedDataRevision: String = "",
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
        versionBreakdown: [ProviderVersionRow] = [],
        dailyByModel: [ProviderDailyModelRow] = []
    ) {
        self.provider = provider
        self.derivedDataRevision = derivedDataRevision
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
        self.dailyByModel = dailyByModel
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
