import Foundation
import HeimdallDomain

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
