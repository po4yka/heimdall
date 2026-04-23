import Foundation
import HeimdallDomain
import HeimdallServices
import Observation

enum MobileRefreshReason: Sendable, Equatable {
    case startup
    case foreground
    case manual
    case shareAccepted
}

enum MobileAccountScope: Sendable, Equatable, Hashable, Codable {
    case all
    case account(String)
    case alias(String)

    private enum CodingKeys: String, CodingKey {
        case kind
        case value
    }

    private enum Kind: String, Codable {
        case all
        case account
        case alias
    }

    var id: String {
        switch self {
        case .all:
            return "all"
        case .account(let key):
            return "account:\(key)"
        case .alias(let aliasID):
            return "alias:\(aliasID)"
        }
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        switch try container.decode(Kind.self, forKey: .kind) {
        case .all:
            self = .all
        case .account:
            self = .account(try container.decode(String.self, forKey: .value))
        case .alias:
            self = .alias(try container.decode(String.self, forKey: .value))
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .all:
            try container.encode(Kind.all, forKey: .kind)
        case .account(let key):
            try container.encode(Kind.account, forKey: .kind)
            try container.encode(key, forKey: .value)
        case .alias(let aliasID):
            try container.encode(Kind.alias, forKey: .kind)
            try container.encode(aliasID, forKey: .value)
        }
    }
}

enum MobileCompressionPreference: String, Sendable, Equatable, Codable, CaseIterable, Identifiable {
    case compact
    case expanded

    var id: String { self.rawValue }

    var title: String {
        switch self {
        case .compact:
            return "Compact"
        case .expanded:
            return "Expanded"
        }
    }
}

struct MobileAccountAlias: Sendable, Equatable, Codable, Identifiable {
    var id: String
    var title: String
    var sourceLabelKeys: [String]

    init(
        id: String = UUID().uuidString.lowercased(),
        title: String,
        sourceLabelKeys: [String]
    ) {
        self.id = id
        self.title = title
        self.sourceLabelKeys = Array(Set(sourceLabelKeys)).sorted()
    }
}

struct MobileAccountSourceLabel: Sendable, Equatable, Hashable, Identifiable {
    var key: String
    var displayTitle: String
    var providerID: ProviderID?
    var providerName: String
    var sourceLabel: String

    var id: String { self.key }
}

struct MobileAccountOption: Sendable, Equatable, Identifiable {
    var scope: MobileAccountScope
    var displayTitle: String
    var providerID: ProviderID?
    var sourceLabelKeys: [String]
    var isAliasBacked: Bool

    var id: String { self.scope.id }

    static let allAccounts = MobileAccountOption(
        scope: .all,
        displayTitle: "All Accounts",
        providerID: nil,
        sourceLabelKeys: [],
        isAliasBacked: false
    )
}

struct MobileAccountSummary: Sendable, Identifiable {
    var option: MobileAccountOption
    var totals: MobileSnapshotTotals
    var installationIDs: [String]
    var hasCurrentLimitData: Bool
    var isStale: Bool

    var id: String { self.option.id }
}

struct MobileScopedAggregateProjection: Sendable {
    var scopedAggregate: SyncedAggregateEnvelope?
    var accountSources: [MobileAccountSourceLabel]
    var accountOptions: [MobileAccountOption]
    var accountSummaries: [MobileAccountSummary]
    var visibleAccountSummaries: [MobileAccountSummary]
    var rolledUpAccountSummary: MobileAccountSummary?
    var visibleInstallations: [SyncedInstallationSnapshot]
    var rolledUpInstallations: [SyncedInstallationSnapshot]
    var selectedScopeTitle: String
    var selectedScopeDescription: String

    static let empty = MobileScopedAggregateProjection(
        scopedAggregate: nil,
        accountSources: [],
        accountOptions: [.allAccounts],
        accountSummaries: [],
        visibleAccountSummaries: [],
        rolledUpAccountSummary: nil,
        visibleInstallations: [],
        rolledUpInstallations: [],
        selectedScopeTitle: "All Accounts",
        selectedScopeDescription: "All synced accounts across all shared installations."
    )

    var providerSnapshots: [ProviderSnapshot] {
        self.scopedAggregate?.aggregateProviderViews.map(\.providerSnapshot) ?? []
    }

    var visibleProviders: [ProviderID] {
        self.scopedAggregate?.aggregateProviderViews.compactMap(\.providerID) ?? []
    }

    var hasScopedData: Bool {
        self.scopedAggregate != nil
    }
}

struct PersistedMobileDashboardPreferences: Sendable, Equatable, Codable {
    var selectedProvider: ProviderID
    var selectedAccountScope: MobileAccountScope
    var compressionPreference: MobileCompressionPreference
    var aliases: [MobileAccountAlias]
    var collapsedSectionIDs: [String]

    init(
        selectedProvider: ProviderID = .claude,
        selectedAccountScope: MobileAccountScope = .all,
        compressionPreference: MobileCompressionPreference = .compact,
        aliases: [MobileAccountAlias] = [],
        collapsedSectionIDs: [String] = []
    ) {
        self.selectedProvider = selectedProvider
        self.selectedAccountScope = selectedAccountScope
        self.compressionPreference = compressionPreference
        self.aliases = aliases
        self.collapsedSectionIDs = collapsedSectionIDs
    }
}

protocol MobileDashboardPreferencesPersisting: Sendable {
    func loadPreferences() -> PersistedMobileDashboardPreferences?
    func savePreferences(_ preferences: PersistedMobileDashboardPreferences)
}

final class UserDefaultsMobileDashboardPreferencesStore: @unchecked Sendable, MobileDashboardPreferencesPersisting {
    private enum Keys {
        static let preferences = "heimdall.mobile.dashboard.preferences"
    }

    private let defaults: UserDefaults
    private let encoder = JSONEncoder()
    private let decoder = JSONDecoder()

    init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
    }

    func loadPreferences() -> PersistedMobileDashboardPreferences? {
        guard let data = self.defaults.data(forKey: Keys.preferences) else {
            return nil
        }
        return try? self.decoder.decode(PersistedMobileDashboardPreferences.self, from: data)
    }

    func savePreferences(_ preferences: PersistedMobileDashboardPreferences) {
        guard let data = try? self.encoder.encode(preferences) else {
            return
        }
        self.defaults.set(data, forKey: Keys.preferences)
    }
}

enum MobileDashboardProjectionBuilder {
    static func build(
        aggregate: SyncedAggregateEnvelope?,
        selectedScope: MobileAccountScope,
        aliases: [MobileAccountAlias],
        compressionPreference: MobileCompressionPreference
    ) -> MobileScopedAggregateProjection {
        guard let aggregate else {
            return .empty
        }

        let rows = self.providerRows(from: aggregate)
        let accountSources = Array(Set(rows.map(\.accountSource))).sorted { lhs, rhs in
            if lhs.providerName == rhs.providerName {
                return lhs.sourceLabel.localizedCaseInsensitiveCompare(rhs.sourceLabel) == .orderedAscending
            }
            return lhs.providerName.localizedCaseInsensitiveCompare(rhs.providerName) == .orderedAscending
        }
        let normalizedAliases = self.normalizedAliases(aliases)
        let accountOptions = self.accountOptions(
            accountSources: accountSources,
            aliases: normalizedAliases
        )
        let scopedAggregate = self.scopedAggregate(
            for: selectedScope,
            rows: rows,
            aggregate: aggregate,
            aliases: normalizedAliases
        )
        let accountSummaries = self.accountSummaries(
            rows: rows,
            accountOptions: accountOptions,
            aggregate: aggregate
        )
        let visibleAccountSummaries: [MobileAccountSummary]
        let rolledUpAccountSummary: MobileAccountSummary?
        if compressionPreference == .compact {
            (visibleAccountSummaries, rolledUpAccountSummary) = self.compressedAccountSummaries(accountSummaries)
        } else {
            visibleAccountSummaries = accountSummaries
            rolledUpAccountSummary = nil
        }
        let visibleInstallations: [SyncedInstallationSnapshot]
        let rolledUpInstallations: [SyncedInstallationSnapshot]
        if compressionPreference == .compact {
            (visibleInstallations, rolledUpInstallations) = self.compressedInstallations(
                scopedAggregate?.installations ?? []
            )
        } else {
            visibleInstallations = scopedAggregate?.installations ?? []
            rolledUpInstallations = []
        }

        return MobileScopedAggregateProjection(
            scopedAggregate: scopedAggregate,
            accountSources: accountSources,
            accountOptions: accountOptions,
            accountSummaries: accountSummaries,
            visibleAccountSummaries: visibleAccountSummaries,
            rolledUpAccountSummary: rolledUpAccountSummary,
            visibleInstallations: visibleInstallations,
            rolledUpInstallations: rolledUpInstallations,
            selectedScopeTitle: self.scopeTitle(selectedScope, accountOptions: accountOptions),
            selectedScopeDescription: self.scopeDescription(selectedScope, accountOptions: accountOptions)
        )
    }

    private static func providerRows(from aggregate: SyncedAggregateEnvelope) -> [ScopedProviderRow] {
        aggregate.installations.flatMap { installation in
            installation.providers.map { snapshot in
                ScopedProviderRow(
                    installationID: installation.installationID,
                    sourceDevice: installation.sourceDevice,
                    publishedAt: installation.publishedAt,
                    snapshot: snapshot,
                    history: installation.history90d.first(where: { $0.provider == snapshot.provider }),
                    accountSource: self.accountSource(
                        sourceDevice: installation.sourceDevice,
                        snapshot: snapshot
                    )
                )
            }
        }
    }

    private static func normalizedAliases(_ aliases: [MobileAccountAlias]) -> [MobileAccountAlias] {
        aliases.map { alias in
            MobileAccountAlias(
                id: alias.id,
                title: alias.title.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
                    ? "Untitled Group"
                    : alias.title.trimmingCharacters(in: .whitespacesAndNewlines),
                sourceLabelKeys: alias.sourceLabelKeys
            )
        }
        .sorted { lhs, rhs in
            lhs.title.localizedCaseInsensitiveCompare(rhs.title) == .orderedAscending
        }
    }

    private static func accountOptions(
        accountSources: [MobileAccountSourceLabel],
        aliases: [MobileAccountAlias]
    ) -> [MobileAccountOption] {
        let aliasedKeys = Set(aliases.flatMap(\.sourceLabelKeys))
        let aliasOptions = aliases.map { alias in
            MobileAccountOption(
                scope: .alias(alias.id),
                displayTitle: alias.title,
                providerID: nil,
                sourceLabelKeys: alias.sourceLabelKeys,
                isAliasBacked: true
            )
        }
        let rawOptions = accountSources
            .filter { !aliasedKeys.contains($0.key) }
            .map { source in
                MobileAccountOption(
                    scope: .account(source.key),
                    displayTitle: source.displayTitle,
                    providerID: source.providerID,
                    sourceLabelKeys: [source.key],
                    isAliasBacked: false
                )
            }

        return [MobileAccountOption.allAccounts] + aliasOptions + rawOptions
    }

    private static func scopedAggregate(
        for selectedScope: MobileAccountScope,
        rows: [ScopedProviderRow],
        aggregate: SyncedAggregateEnvelope,
        aliases: [MobileAccountAlias]
    ) -> SyncedAggregateEnvelope? {
        switch selectedScope {
        case .all:
            return aggregate
        case .account(let key):
            return self.aggregateFromRows(
                rows.filter { $0.accountSource.key == key },
                generatedAt: aggregate.generatedAt
            )
        case .alias(let aliasID):
            let keys = aliases.first(where: { $0.id == aliasID })?.sourceLabelKeys ?? []
            return self.aggregateFromRows(
                rows.filter { keys.contains($0.accountSource.key) },
                generatedAt: aggregate.generatedAt
            )
        }
    }

    private static func accountSummaries(
        rows: [ScopedProviderRow],
        accountOptions: [MobileAccountOption],
        aggregate: SyncedAggregateEnvelope
    ) -> [MobileAccountSummary] {
        accountOptions
            .filter { $0.scope != .all }
            .compactMap { option in
                guard let scopedAggregate = self.aggregateFromRows(
                    rows.filter { option.sourceLabelKeys.contains($0.accountSource.key) },
                    generatedAt: aggregate.generatedAt
                ) else {
                    return nil
                }
                let currentLimitInstallationIDs = Set(scopedAggregate.aggregateProviderViews.flatMap(\.currentLimitInstallationIDs))
                let staleInstallationIDs = Set(scopedAggregate.staleInstallations)
                return MobileAccountSummary(
                    option: option,
                    totals: scopedAggregate.aggregateTotals,
                    installationIDs: scopedAggregate.installations.map(\.installationID),
                    hasCurrentLimitData: !currentLimitInstallationIDs.isEmpty,
                    isStale: !staleInstallationIDs.isEmpty
                )
            }
            .sorted(by: self.compareAccountSummaries)
    }

    private static func compressedAccountSummaries(
        _ summaries: [MobileAccountSummary]
    ) -> ([MobileAccountSummary], MobileAccountSummary?) {
        guard summaries.count > 3 else {
            return (summaries, nil)
        }

        let pinned = summaries.filter { $0.hasCurrentLimitData || $0.isStale || $0.option.isAliasBacked }
        let pinnedIDs = Set(pinned.map(\.id))
        let remaining = summaries.filter { !pinnedIDs.contains($0.id) }
        let explicitCount = max(3 - pinned.count, 0)
        let explicit = (pinned + remaining.prefix(explicitCount)).sorted(by: self.compareAccountSummaries)
        let explicitIDs = Set(explicit.map(\.id))
        let rolledUp = summaries.filter { !explicitIDs.contains($0.id) }
        guard !rolledUp.isEmpty else {
            return (explicit, nil)
        }

        let rolledUpTotals = rolledUp.reduce(
            MobileSnapshotTotals(todayTokens: 0, todayCostUSD: 0, last90DaysTokens: 0, last90DaysCostUSD: 0)
        ) { partial, summary in
            partial.merging(summary.totals)
        }

        let other = MobileAccountSummary(
            option: MobileAccountOption(
                scope: .all,
                displayTitle: "Other Accounts",
                providerID: nil,
                sourceLabelKeys: [],
                isAliasBacked: false
            ),
            totals: rolledUpTotals,
            installationIDs: Array(Set(rolledUp.flatMap(\.installationIDs))).sorted(),
            hasCurrentLimitData: false,
            isStale: false
        )

        return (explicit, other)
    }

    private static func compressedInstallations(
        _ installations: [SyncedInstallationSnapshot]
    ) -> ([SyncedInstallationSnapshot], [SyncedInstallationSnapshot]) {
        guard installations.count > 3 else {
            return (installations, [])
        }

        let pinned = installations.filter { installation in
            installation.isStale || installation.providers.contains(where: { $0.available && !$0.stale })
        }
        let pinnedIDs = Set(pinned.map(\.installationID))
        let remaining = installations.filter { !pinnedIDs.contains($0.installationID) }
        let explicitCount = max(3 - pinned.count, 0)
        let explicit = (pinned + remaining.prefix(explicitCount)).sorted(by: self.compareInstallations)
        let explicitIDs = Set(explicit.map(\.installationID))
        let rolledUp = installations.filter { !explicitIDs.contains($0.installationID) }
        return (explicit, rolledUp)
    }

    private static func aggregateFromRows(
        _ rows: [ScopedProviderRow],
        generatedAt: String
    ) -> SyncedAggregateEnvelope? {
        guard !rows.isEmpty else {
            return nil
        }

        let grouped = Dictionary(grouping: rows, by: \.installationID)
        let installations = grouped.values.compactMap { installationRows in
            guard let first = installationRows.first else {
                return nil
            }
            let providers = installationRows.map(\.snapshot).sorted { lhs, rhs in
                lhs.provider < rhs.provider
            }
            let histories = self.histories(from: installationRows)
            return SyncedInstallationSnapshot(
                installationID: first.installationID,
                sourceDevice: first.sourceDevice,
                publishedAt: first.publishedAt,
                providers: providers,
                history90d: histories,
                totals: self.installationTotals(providers: providers, histories: histories),
                freshness: MobileSnapshotFreshness(
                    newestProviderRefresh: providers.map(\.lastRefresh).max(),
                    oldestProviderRefresh: providers.map(\.lastRefresh).min(),
                    staleProviders: providers.filter { $0.stale }.map(\.provider).sorted(),
                    hasStaleProviders: providers.contains(where: { $0.stale })
                )
            )
        }
        .sorted(by: self.compareInstallations)

        guard !installations.isEmpty else {
            return nil
        }
        return SyncedAggregateEnvelope.aggregate(
            installations: installations,
            generatedAt: generatedAt
        )
    }

    private static func histories(from rows: [ScopedProviderRow]) -> [MobileProviderHistorySeries] {
        let grouped = Dictionary(grouping: rows.compactMap { row in
            row.history.map { (row.snapshot.provider, $0) }
        }, by: \.0)
        return grouped.keys.sorted().compactMap { provider in
            let seriesList = grouped[provider]?.map(\.1) ?? []
            guard !seriesList.isEmpty else {
                return nil
            }
            let groupedPoints = Dictionary(grouping: seriesList.flatMap(\.daily), by: \.day)
            let daily = groupedPoints.map { day, points in
                CostHistoryPoint(
                    day: day,
                    totalTokens: points.reduce(0) { partial, point in partial + point.totalTokens },
                    costUSD: points.reduce(0) { partial, point in partial + point.costUSD },
                    breakdown: TokenBreakdown.sum(points.compactMap(\.breakdown))
                )
            }
            .sorted { lhs, rhs in lhs.day < rhs.day }
            return MobileProviderHistorySeries(
                provider: provider,
                daily: daily,
                totalTokens: seriesList.reduce(0) { partial, series in partial + series.totalTokens },
                totalCostUSD: seriesList.reduce(0) { partial, series in partial + series.totalCostUSD }
            )
        }
    }

    private static func installationTotals(
        providers: [ProviderSnapshot],
        histories: [MobileProviderHistorySeries]
    ) -> MobileSnapshotTotals {
        MobileSnapshotTotals(
            todayTokens: providers.reduce(0) { partial, snapshot in partial + snapshot.costSummary.todayTokens },
            todayCostUSD: providers.reduce(0) { partial, snapshot in partial + snapshot.costSummary.todayCostUSD },
            last90DaysTokens: histories.reduce(0) { partial, series in partial + series.totalTokens },
            last90DaysCostUSD: histories.reduce(0) { partial, series in partial + series.totalCostUSD },
            todayBreakdown: TokenBreakdown.sum(providers.compactMap(\.costSummary.todayBreakdown)),
            last90DaysBreakdown: TokenBreakdown.sum(histories.flatMap { $0.daily.compactMap(\.breakdown) })
        )
    }

    private static func accountSource(
        sourceDevice: String,
        snapshot: ProviderSnapshot
    ) -> MobileAccountSourceLabel {
        let providerName = snapshot.providerID?.title ?? snapshot.provider.capitalized
        if let email = snapshot.identity?.accountEmail?.trimmingCharacters(in: .whitespacesAndNewlines), !email.isEmpty {
            let normalized = email.lowercased()
            return MobileAccountSourceLabel(
                key: "\(snapshot.provider)|email|\(normalized)",
                displayTitle: "\(providerName): \(email)",
                providerID: snapshot.providerID,
                providerName: providerName,
                sourceLabel: email
            )
        }
        if let organization = snapshot.identity?.accountOrganization?.trimmingCharacters(in: .whitespacesAndNewlines), !organization.isEmpty {
            let normalized = organization.lowercased()
            return MobileAccountSourceLabel(
                key: "\(snapshot.provider)|organization|\(normalized)",
                displayTitle: "\(providerName): \(organization)",
                providerID: snapshot.providerID,
                providerName: providerName,
                sourceLabel: organization
            )
        }
        let normalized = sourceDevice.lowercased()
        return MobileAccountSourceLabel(
            key: "\(snapshot.provider)|device|\(normalized)",
            displayTitle: "\(providerName): \(sourceDevice)",
            providerID: snapshot.providerID,
            providerName: providerName,
            sourceLabel: sourceDevice
        )
    }

    private static func scopeTitle(
        _ scope: MobileAccountScope,
        accountOptions: [MobileAccountOption]
    ) -> String {
        accountOptions.first(where: { $0.scope == scope })?.displayTitle ?? MobileAccountOption.allAccounts.displayTitle
    }

    private static func scopeDescription(
        _ scope: MobileAccountScope,
        accountOptions: [MobileAccountOption]
    ) -> String {
        switch scope {
        case .all:
            return "All synced accounts across all shared installations."
        case .account, .alias:
            return accountOptions.first(where: { $0.scope == scope })?.displayTitle ?? "Selected account scope."
        }
    }

    private static func compareAccountSummaries(
        _ lhs: MobileAccountSummary,
        _ rhs: MobileAccountSummary
    ) -> Bool {
        if lhs.totals.last90DaysCostUSD == rhs.totals.last90DaysCostUSD {
            if lhs.totals.last90DaysTokens == rhs.totals.last90DaysTokens {
                return lhs.option.displayTitle.localizedCaseInsensitiveCompare(rhs.option.displayTitle) == .orderedAscending
            }
            return lhs.totals.last90DaysTokens > rhs.totals.last90DaysTokens
        }
        return lhs.totals.last90DaysCostUSD > rhs.totals.last90DaysCostUSD
    }

    private static func compareInstallations(
        _ lhs: SyncedInstallationSnapshot,
        _ rhs: SyncedInstallationSnapshot
    ) -> Bool {
        if lhs.totals.last90DaysCostUSD == rhs.totals.last90DaysCostUSD {
            if lhs.totals.last90DaysTokens == rhs.totals.last90DaysTokens {
                return lhs.installationID < rhs.installationID
            }
            return lhs.totals.last90DaysTokens > rhs.totals.last90DaysTokens
        }
        return lhs.totals.last90DaysCostUSD > rhs.totals.last90DaysCostUSD
    }
}

private struct ScopedProviderRow {
    var installationID: String
    var sourceDevice: String
    var publishedAt: String
    var snapshot: ProviderSnapshot
    var history: MobileProviderHistorySeries?
    var accountSource: MobileAccountSourceLabel
}

@MainActor
@Observable
final class MobileDashboardModel {
    var aggregate: SyncedAggregateEnvelope?
    var isLoading = false
    var lastRefreshError: String?
    var selectedProvider: ProviderID
    var selectedAccountScope: MobileAccountScope
    var compressionPreference: MobileCompressionPreference
    var cloudSyncState = CloudSyncSpaceState()
    var lastSuccessfulRefreshAt: String?
    var isAliasEditorPresented = false
    private(set) var aliases: [MobileAccountAlias]
    private(set) var collapsedSectionIDs: Set<String>
    private var projection = MobileScopedAggregateProjection.empty

    private let store: any SnapshotSyncStore
    private let cache: any SyncedAggregateCaching
    private let preferencesStore: any MobileDashboardPreferencesPersisting
    private let widgetSnapshotCoordinator: WidgetSnapshotCoordinator?
    private let now: @Sendable () -> Date
    private let foregroundRefreshThrottle: TimeInterval
    private let widgetRefreshIntervalSeconds: Int

    init(
        store: any SnapshotSyncStore,
        cache: any SyncedAggregateCaching = NoopSyncedAggregateCache(),
        preferencesStore: any MobileDashboardPreferencesPersisting = UserDefaultsMobileDashboardPreferencesStore(),
        widgetSnapshotCoordinator: WidgetSnapshotCoordinator? = nil,
        now: @escaping @Sendable () -> Date = Date.init,
        foregroundRefreshThrottle: TimeInterval = 60,
        widgetRefreshIntervalSeconds: Int = 900
    ) {
        let persistedPreferences = preferencesStore.loadPreferences()
        let initialCompressionPreference = persistedPreferences?.compressionPreference ?? .compact
        let initialCollapsedSectionIDs = persistedPreferences?.collapsedSectionIDs ?? []
        self.store = store
        self.cache = cache
        self.preferencesStore = preferencesStore
        self.widgetSnapshotCoordinator = widgetSnapshotCoordinator
        self.now = now
        self.foregroundRefreshThrottle = foregroundRefreshThrottle
        self.widgetRefreshIntervalSeconds = widgetRefreshIntervalSeconds
        self.selectedProvider = persistedPreferences?.selectedProvider ?? .claude
        self.selectedAccountScope = persistedPreferences?.selectedAccountScope ?? .all
        self.compressionPreference = initialCompressionPreference
        self.aliases = persistedPreferences?.aliases ?? []
        if initialCollapsedSectionIDs.isEmpty {
            self.collapsedSectionIDs = Self.defaultCollapsedSectionIDs(for: initialCompressionPreference)
        } else {
            self.collapsedSectionIDs = Set(initialCollapsedSectionIDs)
        }
    }

    var providerSnapshots: [ProviderSnapshot] {
        self.projection.providerSnapshots
    }

    var selectedProviderSnapshot: ProviderSnapshot? {
        self.providerSnapshots.first(where: { $0.providerID == self.selectedProvider })
            ?? self.providerSnapshots.first
    }

    var selectedHistorySeries: MobileProviderHistorySeries? {
        self.projection.scopedAggregate?.aggregateHistorySeries(for: self.selectedProvider)
            ?? self.projection.scopedAggregate?.aggregateHistory90d().first
    }

    var installations: [SyncedInstallationSnapshot] {
        self.projection.scopedAggregate?.installations ?? []
    }

    var visibleInstallations: [SyncedInstallationSnapshot] {
        self.projection.visibleInstallations
    }

    var rolledUpInstallations: [SyncedInstallationSnapshot] {
        self.projection.rolledUpInstallations
    }

    var accountSources: [MobileAccountSourceLabel] {
        self.projection.accountSources
    }

    var accountOptions: [MobileAccountOption] {
        self.projection.accountOptions
    }

    var aliasAccountOptions: [MobileAccountOption] {
        self.accountOptions.filter(\.isAliasBacked)
    }

    var rawAccountOptions: [MobileAccountOption] {
        self.accountOptions.filter { !$0.isAliasBacked && $0.scope != .all }
    }

    var accountSummaries: [MobileAccountSummary] {
        self.projection.visibleAccountSummaries
    }

    var rolledUpAccountSummary: MobileAccountSummary? {
        self.projection.rolledUpAccountSummary
    }

    var scopedAggregate: SyncedAggregateEnvelope? {
        self.projection.scopedAggregate
    }

    var scopedTotals: MobileSnapshotTotals? {
        self.projection.scopedAggregate?.aggregateTotals
    }

    var visibleProviders: [ProviderID] {
        self.projection.visibleProviders
    }

    var selectedScopeTitle: String {
        self.projection.selectedScopeTitle
    }

    var selectedScopeDescription: String {
        self.projection.selectedScopeDescription
    }

    var hasScopedData: Bool {
        self.projection.hasScopedData
    }

    var hasSnapshot: Bool {
        self.aggregate != nil
    }

    var lastError: String? {
        self.lastRefreshError
    }

    var staleSnapshotWarning: String? {
        guard self.aggregate != nil else { return nil }
        return self.lastRefreshError
    }

    var scopedEmptyStateTitle: String {
        "No Data In \(self.selectedScopeTitle)"
    }

    var scopedEmptyStateMessage: String {
        switch self.selectedAccountScope {
        case .all:
            return "No synced account data is available yet."
        case .account, .alias:
            return "The selected account scope does not currently have synced provider data."
        }
    }

    var cloudSyncStatusTitle: String {
        switch self.cloudSyncState.status {
        case .participantJoined:
            return "Joined"
        case .ownerReady, .inviteReady:
            return "Configured"
        case .notConfigured:
            return "Not configured"
        case .iCloudUnavailable:
            return "iCloud unavailable"
        case .sharingBlocked:
            return "Sharing blocked"
        }
    }

    var cloudSyncStatusDetail: String {
        if let statusMessage = self.cloudSyncState.statusMessage, !statusMessage.isEmpty {
            return statusMessage
        }
        switch self.cloudSyncState.status {
        case .participantJoined:
            return "This iPhone is connected to the shared Heimdall sync space."
        case .ownerReady, .inviteReady:
            return "Cloud Sync is configured for this Apple account."
        case .notConfigured:
            return "Open a Heimdall share link from macOS to join Cloud Sync on this iPhone."
        case .iCloudUnavailable:
            return "Sign in to iCloud on this iPhone to refresh synced usage data."
        case .sharingBlocked:
            return "CloudKit sharing is restricted on this iPhone."
        }
    }

    var newestPublishedAt: String? {
        self.projection.scopedAggregate?.installations.map(\.publishedAt).max()
            ?? self.aggregate?.installations.map(\.publishedAt).max()
    }

    func load() async {
        await self.refresh(reason: .startup)
    }

    func refresh(reason: MobileRefreshReason) async {
        if reason == .startup {
            await self.primeFromCache()
            // Let SwiftUI render cached content before the live CloudKit fetch continues.
            await Task.yield()
        }

        guard self.shouldRefresh(for: reason) else { return }

        self.isLoading = true
        defer { self.isLoading = false }

        do {
            self.cloudSyncState = try await self.store.loadCloudSyncSpaceState()
        } catch {
            if self.lastRefreshError == nil {
                self.lastRefreshError = error.localizedDescription
            }
        }

        do {
            if let liveAggregate = try await self.store.loadLiveAggregateSnapshot() {
                await self.applyFreshAggregate(liveAggregate)
                return
            }

            if await self.restoreCachedAggregate(errorMessage: nil) {
                return
            }

            if let fallbackAggregate = try await self.store.loadAggregateSnapshot() {
                await self.applyFreshAggregate(fallbackAggregate)
                return
            }

            self.aggregate = nil
            self.lastRefreshError = nil
        } catch {
            let message = error.localizedDescription
            if await self.restoreCachedAggregate(errorMessage: message) {
                return
            }
            if let fallbackAggregate = try? await self.store.loadAggregateSnapshot() {
                await self.applyFreshAggregate(fallbackAggregate)
                self.lastRefreshError = message
                return
            }
            self.lastRefreshError = message
            self.aggregate = nil
        }
    }

    func acceptShareURL(_ url: URL) async {
        self.isLoading = true
        defer { self.isLoading = false }

        do {
            self.cloudSyncState = try await self.store.acceptShareURL(url)
            self.lastRefreshError = nil
        } catch {
            self.lastRefreshError = error.localizedDescription
            return
        }

        await self.refresh(reason: .shareAccepted)
    }

    func selectProvider(_ provider: ProviderID) {
        self.selectedProvider = provider
        self.persistPreferences()
    }

    func selectAccountScope(_ scope: MobileAccountScope) {
        self.selectedAccountScope = scope
        self.rebuildProjection()
    }

    func setCompressionPreference(_ preference: MobileCompressionPreference) {
        guard self.compressionPreference != preference else { return }
        self.compressionPreference = preference
        self.collapsedSectionIDs = Self.defaultCollapsedSectionIDs(for: preference)
        self.rebuildProjection()
    }

    func replaceAliases(_ aliases: [MobileAccountAlias]) {
        self.aliases = aliases
        self.rebuildProjection()
    }

    func presentAliasEditor() {
        self.isAliasEditorPresented = true
    }

    func dismissAliasEditor() {
        self.isAliasEditorPresented = false
    }

    func isSectionCollapsed(_ sectionID: String) -> Bool {
        self.collapsedSectionIDs.contains(sectionID)
    }

    func toggleSectionCollapsed(_ sectionID: String) {
        if self.collapsedSectionIDs.contains(sectionID) {
            self.collapsedSectionIDs.remove(sectionID)
        } else {
            self.collapsedSectionIDs.insert(sectionID)
        }
        self.persistPreferences()
    }

    private func shouldRefresh(for reason: MobileRefreshReason) -> Bool {
        switch reason {
        case .startup, .manual, .shareAccepted:
            return true
        case .foreground:
            guard let lastSuccessfulRefreshAt else { return true }
            guard let lastRefreshDate = Self.isoFormatter.date(from: lastSuccessfulRefreshAt) else {
                return true
            }
            return self.now().timeIntervalSince(lastRefreshDate) >= self.foregroundRefreshThrottle
        }
    }

    private func primeFromCache() async {
        guard let cached = try? await self.cache.loadCachedAggregate() else {
            return
        }
        self.aggregate = cached.aggregate
        self.lastSuccessfulRefreshAt = cached.lastSuccessfulRefreshAt
        self.rebuildProjection()
    }

    private func restoreCachedAggregate(errorMessage: String?) async -> Bool {
        guard let cached = try? await self.cache.loadCachedAggregate() else {
            return false
        }
        self.aggregate = cached.aggregate
        self.lastSuccessfulRefreshAt = cached.lastSuccessfulRefreshAt
        self.lastRefreshError = errorMessage
        self.rebuildProjection()
        return true
    }

    private func applyFreshAggregate(_ aggregate: SyncedAggregateEnvelope) async {
        self.aggregate = aggregate
        self.lastRefreshError = nil
        self.lastSuccessfulRefreshAt = Self.isoFormatter.string(from: self.now())
        self.rebuildProjection()

        if let lastSuccessfulRefreshAt {
            let cached = CachedSyncedAggregateEnvelope(
                aggregate: aggregate,
                cachedAt: Self.isoFormatter.string(from: self.now()),
                lastSuccessfulRefreshAt: lastSuccessfulRefreshAt
            )
            try? await self.cache.saveCachedAggregate(cached)
        }

        if let widgetSnapshotCoordinator {
            let widgetSnapshot = WidgetSnapshotBuilder.snapshot(
                aggregate: aggregate,
                defaultRefreshIntervalSeconds: self.widgetRefreshIntervalSeconds
            )
            try? widgetSnapshotCoordinator.persist(widgetSnapshot)
        }
    }

    private func rebuildProjection() {
        self.projection = MobileDashboardProjectionBuilder.build(
            aggregate: self.aggregate,
            selectedScope: self.selectedAccountScope,
            aliases: self.aliases,
            compressionPreference: self.compressionPreference
        )

        let validScopes = Set(self.projection.accountOptions.map(\.scope))
        if !validScopes.contains(self.selectedAccountScope) {
            self.selectedAccountScope = .all
            self.projection = MobileDashboardProjectionBuilder.build(
                aggregate: self.aggregate,
                selectedScope: self.selectedAccountScope,
                aliases: self.aliases,
                compressionPreference: self.compressionPreference
            )
        }

        if !self.projection.visibleProviders.contains(self.selectedProvider) {
            self.selectedProvider = self.projection.visibleProviders.first ?? .claude
        }

        self.persistPreferences()
    }

    private func persistPreferences() {
        self.preferencesStore.savePreferences(
            PersistedMobileDashboardPreferences(
                selectedProvider: self.selectedProvider,
                selectedAccountScope: self.selectedAccountScope,
                compressionPreference: self.compressionPreference,
                aliases: self.aliases,
                collapsedSectionIDs: Array(self.collapsedSectionIDs).sorted()
            )
        )
    }

    private static func defaultCollapsedSectionIDs(for preference: MobileCompressionPreference) -> Set<String> {
        switch preference {
        case .compact:
            return [
                "overview.accounts",
                "overview.installations",
                "history.days",
                "freshness.installations",
            ]
        case .expanded:
            return []
        }
    }

    private static let isoFormatter = ISO8601DateFormatter()
}

actor FileBackedSyncedAggregateCache: SyncedAggregateCaching {
    private static let filename = "mobile-synced-aggregate.json"
    private let baseURLOverride: URL?
    private let encoder: JSONEncoder
    private let decoder: JSONDecoder

    init(baseURLOverride: URL? = nil) {
        self.baseURLOverride = baseURLOverride
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        self.encoder = encoder
        self.decoder = JSONDecoder()
    }

    func loadCachedAggregate() async throws -> CachedSyncedAggregateEnvelope? {
        let url = try self.cacheURL(createDirectory: false)
        guard FileManager.default.fileExists(atPath: url.path) else {
            return nil
        }
        let data = try Data(contentsOf: url)
        return try self.decoder.decode(CachedSyncedAggregateEnvelope.self, from: data)
    }

    func saveCachedAggregate(_ cached: CachedSyncedAggregateEnvelope) async throws {
        let url = try self.cacheURL(createDirectory: true)
        let data = try self.encoder.encode(cached)
        try data.write(to: url, options: .atomic)
    }

    private func cacheURL(createDirectory: Bool) throws -> URL {
        let baseURL = self.baseURLOverride ?? (
            FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first
            ?? FileManager.default.temporaryDirectory
        )
        let directory = baseURL
            .appendingPathComponent("HeimdallMobile", isDirectory: true)
        if createDirectory {
            try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        }
        return directory.appendingPathComponent(Self.filename, isDirectory: false)
    }
}

private actor NoopSyncedAggregateCache: SyncedAggregateCaching {
    func loadCachedAggregate() async throws -> CachedSyncedAggregateEnvelope? {
        nil
    }

    func saveCachedAggregate(_: CachedSyncedAggregateEnvelope) async throws {}
}
