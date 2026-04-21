import Foundation
import HeimdallDomain

public enum CLICommand: String, Sendable, Equatable {
    case usage
    case cost
    case configValidate
    case configDump
    case authStatus
    case authDoctor
    case authLogin
}

public enum CLIHelpTopic: String, Sendable, Equatable {
    case root
    case usage
    case cost
    case config
    case auth
}

public enum CLIOutputFormat: String, Codable, Sendable, Equatable {
    case text
    case json
}

public struct CLIOptions: Sendable, Equatable {
    public var providers: [ProviderID] = ProviderID.allCases
    public var preferredSource: UsageSourcePreference = .auto
    public var format: CLIOutputFormat = .text
    public var pretty: Bool = false
    public var includeStatus: Bool = false
    public var refresh: Bool = false
    public var deviceAuth: Bool = false

    public init(
        providers: [ProviderID] = ProviderID.allCases,
        preferredSource: UsageSourcePreference = .auto,
        format: CLIOutputFormat = .text,
        pretty: Bool = false,
        includeStatus: Bool = false,
        refresh: Bool = false,
        deviceAuth: Bool = false
    ) {
        self.providers = providers
        self.preferredSource = preferredSource
        self.format = format
        self.pretty = pretty
        self.includeStatus = includeStatus
        self.refresh = refresh
        self.deviceAuth = deviceAuth
    }
}

public struct CLIInvocation: Sendable {
    public var command: CLICommand?
    public var options: CLIOptions
    public var helpTopic: CLIHelpTopic?

    public init(command: CLICommand?, options: CLIOptions = CLIOptions(), helpTopic: CLIHelpTopic? = nil) {
        self.command = command
        self.options = options
        self.helpTopic = helpTopic
    }
}

public enum CLIArgumentError: Error, LocalizedError, Sendable {
    case invalidArguments(String)

    public var errorDescription: String? {
        switch self {
        case .invalidArguments(let message):
            return message
        }
    }
}

private struct CLIOptionAllowance {
    var provider: Bool
    var source: Bool
    var format: Bool
    var pretty: Bool
    var status: Bool
    var refresh: Bool
    var device: Bool

    static let usage = CLIOptionAllowance(provider: true, source: true, format: true, pretty: true, status: true, refresh: true, device: false)
    static let cost = CLIOptionAllowance(provider: true, source: true, format: true, pretty: true, status: true, refresh: true, device: false)
    static let configValidate = CLIOptionAllowance(provider: false, source: false, format: true, pretty: true, status: false, refresh: false, device: false)
    static let configDump = CLIOptionAllowance(provider: false, source: false, format: true, pretty: true, status: false, refresh: false, device: false)
    static let authStatus = CLIOptionAllowance(provider: true, source: false, format: true, pretty: true, status: false, refresh: true, device: false)
    static let authDoctor = CLIOptionAllowance(provider: true, source: false, format: true, pretty: true, status: false, refresh: true, device: false)
    static let authLogin = CLIOptionAllowance(provider: true, source: false, format: false, pretty: false, status: false, refresh: false, device: true)
}

public enum CLIArgumentParser {
    public static func parse(arguments: [String]) throws -> CLIInvocation {
        let args = Array(arguments.dropFirst())
        guard let first = args.first else {
            return CLIInvocation(command: .usage)
        }

        if Self.isHelpFlag(first) {
            return CLIInvocation(command: nil, helpTopic: .root)
        }

        switch first {
        case "usage":
            if args.dropFirst().first.map(Self.isHelpFlag) == true {
                return CLIInvocation(command: nil, helpTopic: .usage)
            }
            let options = try Self.parseOptions(Array(args.dropFirst()), allowance: .usage)
            return CLIInvocation(command: .usage, options: options)
        case "cost":
            if args.dropFirst().first.map(Self.isHelpFlag) == true {
                return CLIInvocation(command: nil, helpTopic: .cost)
            }
            let options = try Self.parseOptions(Array(args.dropFirst()), allowance: .cost)
            return CLIInvocation(command: .cost, options: options)
        case "config":
            let rest = Array(args.dropFirst())
            if rest.first.map(Self.isHelpFlag) != false {
                return CLIInvocation(command: nil, helpTopic: .config)
            }
            let action = rest.first ?? "validate"
            let optionArgs = Array(rest.dropFirst())
            switch action {
            case "validate":
                let options = try Self.parseOptions(optionArgs, allowance: .configValidate)
                return CLIInvocation(command: .configValidate, options: options)
            case "dump":
                var options = try Self.parseOptions(optionArgs, allowance: .configDump)
                if !optionArgs.contains("--format") {
                    options.format = .json
                } else if options.format == .text {
                    throw CLIArgumentError.invalidArguments("config dump only supports --format json")
                }
                return CLIInvocation(command: .configDump, options: options)
            default:
                throw CLIArgumentError.invalidArguments("config expects validate|dump; run `heimdallbar config --help`")
            }
        case "auth":
            let rest = Array(args.dropFirst())
            if rest.first.map(Self.isHelpFlag) != false {
                return CLIInvocation(command: nil, helpTopic: .auth)
            }
            let action = rest.first ?? "status"
            let optionArgs = Array(rest.dropFirst())
            switch action {
            case "status":
                let options = try Self.parseOptions(optionArgs, allowance: .authStatus)
                return CLIInvocation(command: .authStatus, options: options)
            case "doctor":
                let options = try Self.parseOptions(optionArgs, allowance: .authDoctor)
                return CLIInvocation(command: .authDoctor, options: options)
            case "login":
                let options = try Self.parseOptions(optionArgs, allowance: .authLogin)
                guard options.providers.count == 1 else {
                    throw CLIArgumentError.invalidArguments("auth login requires --provider claude|codex")
                }
                if options.deviceAuth, options.providers.first != .codex {
                    throw CLIArgumentError.invalidArguments("--device is only valid for codex login")
                }
                return CLIInvocation(command: .authLogin, options: options)
            default:
                throw CLIArgumentError.invalidArguments("auth expects status|doctor|login; run `heimdallbar auth --help`")
            }
        default:
            throw CLIArgumentError.invalidArguments("unknown command: \(first); run `heimdallbar --help`")
        }
    }

    public static func helpText(for topic: CLIHelpTopic) -> String {
        switch topic {
        case .root:
            return """
            Usage:
              heimdallbar usage [options]
              heimdallbar cost [options]
              heimdallbar auth status [options]
              heimdallbar auth doctor [options]
              heimdallbar auth login --provider claude|codex [--device]
              heimdallbar config validate [--format text|json] [--pretty]
              heimdallbar config dump [--format json] [--pretty]

            Commands:
              usage   Show live quota lanes, source selection, status, and local cost context.
              cost    Show local cost summaries with the same source-resolution metadata.
              auth    Show auth diagnosis, recovery guidance, or run provider login flows.
              config  Validate or dump HeimdallBar configuration.

            Common options:
              --provider claude|codex|both
              --source auto|oauth|web|cli
              --format text|json
              --pretty
              --status
              --refresh
              --help
            """
        case .usage:
            return """
            Usage:
              heimdallbar usage [options]

            Options:
              --provider claude|codex|both   Restrict output to one provider or both.
              --source auto|oauth|web|cli   Override the requested source-selection mode.
              --format text|json            Render human text or stable JSON.
              --pretty                      Pretty-print JSON output.
              --status                      Include provider status / incident detail.
              --refresh                     Force a live refresh before rendering.
              --help                        Show this help.
            """
        case .cost:
            return """
            Usage:
              heimdallbar cost [options]

            Options:
              --provider claude|codex|both   Restrict output to one provider or both.
              --source auto|oauth|web|cli   Override the requested source-selection mode.
              --format text|json            Render human text or stable JSON.
              --pretty                      Pretty-print JSON output.
              --status                      Include provider status / incident detail.
              --refresh                     Force a live refresh before rendering.
              --help                        Show this help.
            """
        case .config:
            return """
            Usage:
              heimdallbar config validate [--format text|json] [--pretty]
              heimdallbar config dump [--format json] [--pretty]

            Subcommands:
              validate   Validate the stored HeimdallBar configuration.
              dump       Emit the current HeimdallBar configuration as JSON.
            """
        case .auth:
            return """
            Usage:
              heimdallbar auth status [options]
              heimdallbar auth doctor [options]
              heimdallbar auth login --provider claude|codex [--device]

            Auth commands:
              status   Show the current login method, credential store, compatibility, and next actions.
              doctor   Show the same diagnosis with diagnostic codes and recovery details.
              login    Run the official Claude or Codex login flow.

            Status/doctor options:
              --provider claude|codex|both
              --format text|json
              --pretty
              --refresh

            Login options:
              --provider claude|codex
              --device   Run `codex login --device-auth`.
            """
        }
    }

    private static func parseOptions(
        _ arguments: [String],
        allowance: CLIOptionAllowance
    ) throws -> CLIOptions {
        var options = CLIOptions()
        var index = 0

        while index < arguments.count {
            let argument = arguments[index]
            switch argument {
            case "--provider":
                guard allowance.provider else {
                    throw CLIArgumentError.invalidArguments("--provider is not valid for this command")
                }
                index += 1
                guard index < arguments.count else {
                    throw CLIArgumentError.invalidArguments("--provider requires claude|codex|both")
                }
                switch arguments[index] {
                case "claude":
                    options.providers = [.claude]
                case "codex":
                    options.providers = [.codex]
                case "both":
                    options.providers = ProviderID.allCases
                default:
                    throw CLIArgumentError.invalidArguments("unsupported provider: \(arguments[index])")
                }
            case "--source":
                guard allowance.source else {
                    throw CLIArgumentError.invalidArguments("--source is not valid for this command")
                }
                index += 1
                guard index < arguments.count,
                      let source = UsageSourcePreference(rawValue: arguments[index]) else {
                    throw CLIArgumentError.invalidArguments("--source requires auto|oauth|web|cli")
                }
                options.preferredSource = source
            case "--format":
                guard allowance.format else {
                    throw CLIArgumentError.invalidArguments("--format is not valid for this command")
                }
                index += 1
                guard index < arguments.count,
                      let format = CLIOutputFormat(rawValue: arguments[index]) else {
                    throw CLIArgumentError.invalidArguments("--format requires text|json")
                }
                options.format = format
            case "--pretty":
                guard allowance.pretty else {
                    throw CLIArgumentError.invalidArguments("--pretty is not valid for this command")
                }
                options.pretty = true
                options.format = .json
            case "--status":
                guard allowance.status else {
                    throw CLIArgumentError.invalidArguments("--status is not valid for this command")
                }
                options.includeStatus = true
            case "--refresh":
                guard allowance.refresh else {
                    throw CLIArgumentError.invalidArguments("--refresh is not valid for this command")
                }
                options.refresh = true
            case "--device":
                guard allowance.device else {
                    throw CLIArgumentError.invalidArguments("--device is not valid for this command")
                }
                options.deviceAuth = true
            case let helpFlag where Self.isHelpFlag(helpFlag):
                throw CLIArgumentError.invalidArguments("help flags must appear directly after the command")
            default:
                throw CLIArgumentError.invalidArguments("unknown option: \(argument)")
            }
            index += 1
        }

        return options
    }

    private static func isHelpFlag(_ value: String) -> Bool {
        value == "--help" || value == "-h" || value == "help"
    }
}

public struct CLIRefreshMetadata: Sendable {
    public var requestedRefresh: Bool
    public var responseScope: String
    public var requestedProvider: String?
    public var refreshedProviders: [String]
    public var cacheHit: Bool
    public var fetchedAt: String

    public init(
        requestedRefresh: Bool,
        responseScope: String,
        requestedProvider: String?,
        refreshedProviders: [String],
        cacheHit: Bool,
        fetchedAt: String
    ) {
        self.requestedRefresh = requestedRefresh
        self.responseScope = responseScope
        self.requestedProvider = requestedProvider
        self.refreshedProviders = refreshedProviders
        self.cacheHit = cacheHit
        self.fetchedAt = fetchedAt
    }
}

public struct CLIProviderSection: Sendable {
    public var provider: ProviderID
    public var requestedSource: UsageSourcePreference
    public var presentation: ProviderPresentationState
    public var projection: ProviderMenuProjection
    public var costSummary: ProviderCostSummary
    public var auth: ProviderAuthHealth?

    public init(
        provider: ProviderID,
        requestedSource: UsageSourcePreference,
        presentation: ProviderPresentationState,
        projection: ProviderMenuProjection,
        costSummary: ProviderCostSummary,
        auth: ProviderAuthHealth? = nil
    ) {
        self.provider = provider
        self.requestedSource = requestedSource
        self.presentation = presentation
        self.projection = projection
        self.costSummary = costSummary
        self.auth = auth ?? presentation.auth
    }
}

public enum CLITextFormatter {
    public static func usageText(
        sections: [CLIProviderSection],
        refresh: CLIRefreshMetadata?,
        includeStatus: Bool
    ) -> String {
        self.render(
            commandLabel: "Usage",
            sections: sections,
            refresh: refresh,
            includeStatus: includeStatus
        )
    }

    public static func costText(
        sections: [CLIProviderSection],
        refresh: CLIRefreshMetadata?,
        includeStatus: Bool
    ) -> String {
        self.render(
            commandLabel: "Cost",
            sections: sections,
            refresh: refresh,
            includeStatus: includeStatus
        )
    }

    public static func authStatusText(
        sections: [CLIProviderSection],
        refresh: CLIRefreshMetadata?
    ) -> String {
        self.renderAuth(
            commandLabel: "Auth status",
            sections: sections,
            refresh: refresh,
            verbose: false
        )
    }

    public static func authDoctorText(
        sections: [CLIProviderSection],
        refresh: CLIRefreshMetadata?
    ) -> String {
        self.renderAuth(
            commandLabel: "Auth doctor",
            sections: sections,
            refresh: refresh,
            verbose: true
        )
    }

    private static func render(
        commandLabel: String,
        sections: [CLIProviderSection],
        refresh: CLIRefreshMetadata?,
        includeStatus: Bool
    ) -> String {
        var lines = [String]()

        if let refresh {
            lines.append("\(commandLabel) snapshot: \(refresh.requestedRefresh ? "forced refresh" : "cached fetch")")
            lines.append("Scope: \(refresh.responseScope)")
            if let requestedProvider = refresh.requestedProvider {
                lines.append("Requested provider: \(requestedProvider)")
            }
            if !refresh.refreshedProviders.isEmpty {
                lines.append("Refreshed providers: \(refresh.refreshedProviders.joined(separator: ", "))")
            }
            lines.append("Cache: \(refresh.cacheHit ? "hit" : "miss")")
            lines.append("Fetched at: \(refresh.fetchedAt)")
            lines.append("")
        }

        for (index, section) in sections.enumerated() {
            if index > 0 {
                lines.append("")
            }

            lines.append("== \(section.provider.title) ==")
            lines.append("Requested source: \(section.requestedSource.rawValue)")
            lines.append(section.projection.sourceLabel)
            if let explanation = section.projection.sourceExplanationLabel {
                lines.append(explanation)
            }
            if !section.presentation.resolution.fallbackChain.isEmpty {
                lines.append("Fallbacks: \(section.presentation.resolution.fallbackChain.joined(separator: " -> "))")
            }
            if section.presentation.resolution.requiresLogin {
                lines.append("Login: browser session required for requested source")
            }
            for warning in section.projection.warningLabels {
                lines.append("Warning: \(warning)")
            }
            if includeStatus || self.shouldShowStatus(for: section.projection) {
                lines.append(self.statusLine(for: section))
            }
            lines.append("Refresh: \(section.projection.refreshStatusLabel)")

            if section.projection.laneDetails.isEmpty {
                lines.append("Usage: unavailable for requested source")
            } else {
                for lane in section.projection.laneDetails {
                    lines.append("Lane: \(self.laneSummary(lane))")
                }
            }

            if let creditsLabel = section.projection.creditsLabel {
                lines.append("Credits: \(creditsLabel)")
            }
            if let webAccountLine = self.webAccountLine(for: section.presentation) {
                lines.append("Web: \(webAccountLine)")
            }
            lines.append(String(format: "Today: $%.2f · %d tokens", section.costSummary.todayCostUSD, section.costSummary.todayTokens))
            lines.append(String(format: "30d: $%.2f · %d tokens", section.costSummary.last30DaysCostUSD, section.costSummary.last30DaysTokens))
        }

        return lines.joined(separator: "\n")
    }

    private static func renderAuth(
        commandLabel: String,
        sections: [CLIProviderSection],
        refresh: CLIRefreshMetadata?,
        verbose: Bool
    ) -> String {
        var lines = [String]()

        if let refresh {
            lines.append("\(commandLabel): \(refresh.requestedRefresh ? "forced refresh" : "cached fetch")")
            lines.append("Scope: \(refresh.responseScope)")
            lines.append("Fetched at: \(refresh.fetchedAt)")
            lines.append("")
        }

        for (index, section) in sections.enumerated() {
            if index > 0 {
                lines.append("")
            }
            lines.append("== \(section.provider.title) ==")
            lines.append("Headline: \(section.projection.authHeadline ?? "No auth diagnosis")")
            if let summary = section.projection.authSummaryLabel {
                lines.append("Summary: \(summary)")
            }
            if let detail = section.projection.authDetail {
                lines.append("Detail: \(detail)")
            }
            if let auth = section.auth {
                lines.append("Authenticated: \(auth.isAuthenticated ? "yes" : "no")")
                lines.append("Source compatible: \(auth.isSourceCompatible ? "yes" : "no")")
                lines.append("Requires re-login: \(auth.requiresRelogin ? "yes" : "no")")
                if verbose {
                    if let loginMethod = auth.loginMethod {
                        lines.append("Login method: \(loginMethod)")
                    }
                    if let backend = auth.credentialBackend {
                        lines.append("Credential store: \(backend)")
                    }
                    if let authMode = auth.authMode {
                        lines.append("Auth mode: \(authMode)")
                    }
                    if let diagnostic = auth.diagnosticCode {
                        lines.append("Diagnostic: \(diagnostic)")
                    }
                    if let restriction = auth.managedRestriction {
                        lines.append("Managed restriction: \(restriction)")
                    }
                    if let validated = auth.lastValidatedAt {
                        lines.append("Last validated: \(validated)")
                    }
                }
            }
            if section.projection.authRecoveryActions.isEmpty {
                lines.append("Recovery: none suggested")
            } else {
                for action in section.projection.authRecoveryActions {
                    let suffix = verbose ? action.detail.map { " (\($0))" } ?? "" : ""
                    lines.append("Recovery: \(action.label)\(suffix)")
                }
            }
        }

        return lines.joined(separator: "\n")
    }

    private static func shouldShowStatus(for projection: ProviderMenuProjection) -> Bool {
        switch projection.visualState {
        case .healthy, .refreshing:
            return projection.incidentLabel != nil
        case .stale, .degraded, .incident, .error:
            return true
        }
    }

    private static func statusLine(for section: CLIProviderSection) -> String {
        let headline = section.projection.statusLabel
            ?? section.projection.incidentLabel
            ?? section.projection.stateLabel
        if let providerStatus = section.presentation.snapshot?.status {
            return "Status: [\(providerStatus.indicator)] \(providerStatus.description)"
        }
        return "Status: \(headline)"
    }

    private static func laneSummary(_ lane: LaneDetailProjection) -> String {
        [
            lane.title,
            lane.summary,
            lane.paceLabel.map { "pace \($0.lowercased())" },
            lane.resetDetail,
        ]
        .compactMap { $0 }
        .joined(separator: " · ")
    }

    private static func webAccountLine(for presentation: ProviderPresentationState) -> String? {
        let parts = [
            presentation.webExtras?.signedInEmail,
            presentation.webExtras?.accountPlan,
        ]
        .compactMap { $0 }
        return parts.isEmpty ? nil : parts.joined(separator: " · ")
    }
}
