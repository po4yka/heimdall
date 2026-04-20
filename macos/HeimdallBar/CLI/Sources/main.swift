import Darwin
import Foundation
import HeimdallBarShared

enum CLIError: Error, LocalizedError {
    case invalidArguments(String)

    var errorDescription: String? {
        switch self {
        case .invalidArguments(let message):
            return message
        }
    }
}

struct CLILiveState {
    var envelope: ProviderSnapshotEnvelope
    var sections: [CLIProviderSection]
}

struct CLIJSONRefresh: Encodable {
    var requestedRefresh: Bool
    var responseScope: String
    var requestedProvider: String?
    var refreshedProviders: [String]
    var cacheHit: Bool
    var fetchedAt: String
}

struct CLIJSONSource: Encodable {
    var requestedSource: String
    var effectiveSource: String?
    var effectiveSourceDetail: String?
    var sourceLabel: String
    var sourceExplanation: String
    var sourceWarnings: [String]
    var sourceFallbackChain: [String]
    var usageAvailable: Bool
    var requiresLogin: Bool
    var isUnsupported: Bool
    var usesFallback: Bool
}

struct CLIJSONWindow: Encodable {
    var usedPercent: Double
    var resetsAt: String?
    var resetsInMinutes: Int?
    var windowMinutes: Int?
    var resetLabel: String?
}

struct CLIJSONStatus: Encodable {
    var indicator: String
    var description: String
    var pageURL: String
}

struct CLIJSONWebQuotaLane: Encodable {
    var title: String
    var window: CLIJSONWindow
}

struct CLIJSONWebExtras: Encodable {
    var signedInEmail: String?
    var accountPlan: String?
    var creditsRemaining: Double?
    var creditsPurchaseURL: String?
    var sourceURL: String?
    var fetchedAt: String
    var quotaLanes: [CLIJSONWebQuotaLane]
}

struct CLIJSONUsageProvider: Encodable {
    var provider: String
    var available: Bool
    var sourceUsed: String?
    var source: CLIJSONSource
    var stateLabel: String
    var refreshLabel: String
    var warnings: [String]
    var primary: CLIJSONWindow?
    var secondary: CLIJSONWindow?
    var tertiary: CLIJSONWindow?
    var credits: Double?
    var status: CLIJSONStatus?
    var webExtras: CLIJSONWebExtras?
    var todayCostUSD: Double
    var last30DaysCostUSD: Double
    var todayTokens: Int
    var last30DaysTokens: Int
}

struct CLIJSONUsageResponse: Encodable {
    var command: String
    var preferredSource: String
    var refresh: CLIJSONRefresh
    var providers: [CLIJSONUsageProvider]
}

typealias CLIJSONCostProvider = CLIJSONUsageProvider

struct CLIJSONCostResponse: Encodable {
    var command: String
    var preferredSource: String
    var refresh: CLIJSONRefresh
    var providers: [CLIJSONCostProvider]
}

struct HeimdallBarCLI {
    static func run(arguments: [String]) async throws {
        let invocation = try CLIArgumentParser.parse(arguments: arguments)

        if let helpTopic = invocation.helpTopic {
            print(CLIArgumentParser.helpText(for: helpTopic))
            return
        }

        guard let command = invocation.command else {
            throw CLIError.invalidArguments("missing command")
        }

        switch command {
        case .usage:
            try await self.runUsage(options: invocation.options)
        case .cost:
            try await self.runCost(options: invocation.options)
        case .configValidate:
            try self.runConfigValidate(options: invocation.options)
        case .configDump:
            try self.runConfigDump(options: invocation.options)
        }
    }

    private static func runUsage(options: CLIOptions) async throws {
        let config = ConfigStore.shared.load()
        let state = try await self.loadLiveState(options: options, config: config)
        let refreshMetadata = self.refreshMetadata(from: state.envelope, requestedRefresh: options.refresh)

        if options.format == .json {
            let response = CLIJSONUsageResponse(
                command: "usage",
                preferredSource: options.preferredSource.rawValue,
                refresh: refreshMetadata,
                providers: state.sections.map { self.usageJSONProvider(from: $0, includeStatus: options.includeStatus) }
            )
            try self.writeJSON(response, pretty: options.pretty)
            return
        }

        print(CLITextFormatter.usageText(
            sections: state.sections,
            refresh: self.refreshMetadataValue(from: refreshMetadata),
            includeStatus: options.includeStatus
        ))
    }

    private static func runCost(options: CLIOptions) async throws {
        let config = ConfigStore.shared.load()
        let state = try await self.loadLiveState(options: options, config: config)
        let client = HeimdallAPIClient(port: config.helperPort)
        let summaryMap = try await self.fetchCostSummaries(client: client, providers: options.providers)
        let sections = state.sections.map { section in
            var updated = section
            updated.costSummary = summaryMap[section.provider] ?? section.costSummary
            return updated
        }
        let refreshMetadata = self.refreshMetadata(from: state.envelope, requestedRefresh: options.refresh)

        if options.format == .json {
            let response = CLIJSONCostResponse(
                command: "cost",
                preferredSource: options.preferredSource.rawValue,
                refresh: refreshMetadata,
                providers: sections.map { self.usageJSONProvider(from: $0, includeStatus: options.includeStatus) }
            )
            try self.writeJSON(response, pretty: options.pretty)
            return
        }

        print(CLITextFormatter.costText(
            sections: sections,
            refresh: self.refreshMetadataValue(from: refreshMetadata),
            includeStatus: options.includeStatus
        ))
    }

    private static func runConfigValidate(options: CLIOptions) throws {
        try ConfigStore.shared.validate()
        if options.format == .json {
            try self.writeJSON(["valid": true], pretty: options.pretty)
        } else {
            print("valid")
        }
    }

    private static func runConfigDump(options: CLIOptions) throws {
        let config = ConfigStore.shared.load()
        try self.writeJSON(config, pretty: options.pretty)
    }

    private static func loadLiveState(
        options: CLIOptions,
        config: HeimdallBarConfig
    ) async throws -> CLILiveState {
        let client = HeimdallAPIClient(port: config.helperPort)
        let envelope = if options.refresh {
            try await client.refresh(provider: options.providers.count == 1 ? options.providers.first : nil)
        } else {
            try await client.fetchSnapshots()
        }

        let adjunctController = DashboardAdjunctController()
        let snapshotsByProvider = Dictionary(uniqueKeysWithValues: envelope.providers.compactMap { snapshot in
            snapshot.providerID.map { ($0, snapshot) }
        })

        var sections = [CLIProviderSection]()
        for provider in options.providers {
            let providerConfig = Self.configuredProviderConfig(
                config: config,
                provider: provider,
                overrideSource: options.preferredSource
            )
            let snapshot = snapshotsByProvider[provider]
            let adjunct = await adjunctController.loadAdjunct(
                provider: provider,
                config: providerConfig,
                snapshot: snapshot,
                forceRefresh: options.refresh,
                allowLiveNavigation: false
            )
            let presentation = SourceResolver.presentation(
                for: provider,
                config: providerConfig,
                snapshot: snapshot,
                adjunct: adjunct
            )
            let projection = MenuProjectionBuilder.projection(
                from: presentation,
                config: config,
                isRefreshing: false,
                lastGlobalError: nil
            )
            sections.append(
                CLIProviderSection(
                    provider: provider,
                    requestedSource: options.preferredSource,
                    presentation: presentation,
                    projection: projection,
                    costSummary: snapshot?.costSummary ?? ProviderCostSummary(
                        todayTokens: 0,
                        todayCostUSD: 0,
                        last30DaysTokens: 0,
                        last30DaysCostUSD: 0,
                        daily: []
                    )
                )
            )
        }

        return CLILiveState(envelope: envelope, sections: sections)
    }

    private static func fetchCostSummaries(
        client: HeimdallAPIClient,
        providers: [ProviderID]
    ) async throws -> [ProviderID: ProviderCostSummary] {
        var summaries = [ProviderID: ProviderCostSummary]()
        for provider in providers {
            let response = try await client.fetchCostSummary(provider: provider)
            summaries[provider] = response.summary
        }
        return summaries
    }

    private static func configuredProviderConfig(
        config: HeimdallBarConfig,
        provider: ProviderID,
        overrideSource: UsageSourcePreference
    ) -> ProviderConfig {
        var providerConfig = config.providerConfig(for: provider)
        providerConfig.source = overrideSource
        return providerConfig
    }

    private static func usageJSONProvider(
        from section: CLIProviderSection,
        includeStatus: Bool
    ) -> CLIJSONUsageProvider {
        let snapshot = section.presentation.snapshot
        return CLIJSONUsageProvider(
            provider: section.provider.rawValue,
            available: snapshot?.available ?? false,
            sourceUsed: snapshot?.sourceUsed,
            source: self.sourceMetadata(section.presentation),
            stateLabel: section.projection.stateLabel,
            refreshLabel: section.projection.refreshStatusLabel,
            warnings: section.projection.warningLabels,
            primary: self.windowPayload(section.presentation.primary),
            secondary: self.windowPayload(section.presentation.secondary),
            tertiary: self.windowPayload(section.presentation.tertiary),
            credits: section.presentation.displayCredits,
            status: includeStatus ? self.statusPayload(snapshot?.status) : nil,
            webExtras: self.webExtrasPayload(section.presentation.webExtras),
            todayCostUSD: section.costSummary.todayCostUSD,
            last30DaysCostUSD: section.costSummary.last30DaysCostUSD,
            todayTokens: section.costSummary.todayTokens,
            last30DaysTokens: section.costSummary.last30DaysTokens
        )
    }

    private static func sourceMetadata(_ presentation: ProviderPresentationState) -> CLIJSONSource {
        CLIJSONSource(
            requestedSource: presentation.resolution.requestedSource.rawValue,
            effectiveSource: presentation.resolution.effectiveSource?.rawValue,
            effectiveSourceDetail: presentation.resolution.effectiveSourceDetail,
            sourceLabel: presentation.resolution.sourceLabel,
            sourceExplanation: presentation.resolution.explanation,
            sourceWarnings: presentation.resolution.warnings,
            sourceFallbackChain: presentation.resolution.fallbackChain,
            usageAvailable: presentation.resolution.usageAvailable,
            requiresLogin: presentation.resolution.requiresLogin,
            isUnsupported: presentation.resolution.isUnsupported,
            usesFallback: presentation.resolution.usesFallback
        )
    }

    private static func windowPayload(_ window: ProviderRateWindow?) -> CLIJSONWindow? {
        guard let window else { return nil }
        return CLIJSONWindow(
            usedPercent: window.usedPercent,
            resetsAt: window.resetsAt,
            resetsInMinutes: window.resetsInMinutes,
            windowMinutes: window.windowMinutes,
            resetLabel: window.resetLabel
        )
    }

    private static func statusPayload(_ status: ProviderStatusSummary?) -> CLIJSONStatus? {
        guard let status else { return nil }
        return CLIJSONStatus(
            indicator: status.indicator,
            description: status.description,
            pageURL: status.pageURL
        )
    }

    private static func webExtrasPayload(_ extras: DashboardWebExtras?) -> CLIJSONWebExtras? {
        guard let extras else { return nil }
        return CLIJSONWebExtras(
            signedInEmail: extras.signedInEmail,
            accountPlan: extras.accountPlan,
            creditsRemaining: extras.creditsRemaining,
            creditsPurchaseURL: extras.creditsPurchaseURL,
            sourceURL: extras.sourceURL,
            fetchedAt: extras.fetchedAt,
            quotaLanes: extras.quotaLanes.map { lane in
                CLIJSONWebQuotaLane(
                    title: lane.title,
                    window: CLIJSONWindow(
                        usedPercent: lane.window.usedPercent,
                        resetsAt: lane.window.resetsAt,
                        resetsInMinutes: lane.window.resetsInMinutes,
                        windowMinutes: lane.window.windowMinutes,
                        resetLabel: lane.window.resetLabel
                    )
                )
            }
        )
    }

    private static func refreshMetadata(
        from envelope: ProviderSnapshotEnvelope,
        requestedRefresh: Bool
    ) -> CLIJSONRefresh {
        CLIJSONRefresh(
            requestedRefresh: requestedRefresh,
            responseScope: envelope.responseScope,
            requestedProvider: envelope.requestedProvider,
            refreshedProviders: envelope.refreshedProviders,
            cacheHit: envelope.cacheHit,
            fetchedAt: envelope.fetchedAt
        )
    }

    private static func refreshMetadataValue(from metadata: CLIJSONRefresh) -> CLIRefreshMetadata {
        CLIRefreshMetadata(
            requestedRefresh: metadata.requestedRefresh,
            responseScope: metadata.responseScope,
            requestedProvider: metadata.requestedProvider,
            refreshedProviders: metadata.refreshedProviders,
            cacheHit: metadata.cacheHit,
            fetchedAt: metadata.fetchedAt
        )
    }

    private static func writeJSON<T: Encodable>(_ object: T, pretty: Bool) throws {
        let encoder = JSONEncoder()
        encoder.outputFormatting = pretty ? [.prettyPrinted, .sortedKeys] : [.sortedKeys]
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let data = try encoder.encode(object)
        FileHandle.standardOutput.write(data)
        FileHandle.standardOutput.write(Data("\n".utf8))
    }
}

do {
    try await HeimdallBarCLI.run(arguments: CommandLine.arguments)
} catch {
    fputs("\(error.localizedDescription)\n", stderr)
    Darwin.exit(1)
}
