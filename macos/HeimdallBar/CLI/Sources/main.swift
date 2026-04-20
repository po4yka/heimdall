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

enum CLIOutputFormat: String {
    case text
    case json
}

struct CLIOptions {
    var providers: [ProviderID] = ProviderID.allCases
    var preferredSource: UsageSourcePreference = .auto
    var format: CLIOutputFormat = .text
    var pretty: Bool = false
    var includeStatus: Bool = false
    var refresh: Bool = false
}

struct HeimdallBarCLI {
    static func run(arguments: [String]) async throws {
        let command = arguments.dropFirst().first ?? "usage"
        let rest = Array(arguments.dropFirst(2))

        switch command {
        case "usage":
            try await self.runUsage(arguments: rest)
        case "cost":
            try await self.runCost(arguments: rest)
        case "config":
            try self.runConfig(arguments: rest)
        default:
            throw CLIError.invalidArguments("unknown command: \(command)")
        }
    }

    private static func runUsage(arguments: [String]) async throws {
        let options = try self.parseOptions(arguments)
        let config = ConfigStore.shared.load()
        let client = HeimdallAPIClient(port: config.helperPort)
        let envelope = if options.refresh {
            try await client.refresh(provider: options.providers.count == 1 ? options.providers.first : nil)
        } else {
            try await client.fetchSnapshots()
        }

        let snapshots = envelope.providers.filter { snapshot in
            guard let provider = snapshot.providerID else { return false }
            return options.providers.contains(provider)
        }

        if options.format == .json {
            try self.writeJSON([
                "preferred_source": options.preferredSource.rawValue,
                "providers": snapshots.map(self.snapshotDictionary(includeStatus: options.includeStatus)),
            ], pretty: options.pretty)
            return
        }

        for snapshot in snapshots {
            print("== \(snapshot.provider.capitalized) (\(snapshot.sourceUsed)) ==")
            print("Preferred source: \(options.preferredSource.rawValue)")
            if let primary = snapshot.primary {
                print("Session: \(Int((100 - primary.usedPercent).rounded()))% left")
            }
            if let secondary = snapshot.secondary {
                print("Weekly: \(Int((100 - secondary.usedPercent).rounded()))% left")
            }
            if let tertiary = snapshot.tertiary {
                print("Extra: \(Int((100 - tertiary.usedPercent).rounded()))% left")
            }
            if let credits = snapshot.credits {
                print("Credits: \(String(format: "%.2f", credits))")
            }
            if options.includeStatus, let status = snapshot.status {
                print("Status: [\(status.indicator)] \(status.description)")
            }
            print("Today: $\(String(format: "%.2f", snapshot.costSummary.todayCostUSD))")
            print("Last 30d: $\(String(format: "%.2f", snapshot.costSummary.last30DaysCostUSD))")
            print("")
        }
    }

    private static func runCost(arguments: [String]) async throws {
        let options = try self.parseOptions(arguments)
        let config = ConfigStore.shared.load()
        let client = HeimdallAPIClient(port: config.helperPort)

        var summaries = [[String: Any]]()
        for provider in options.providers {
            let summary = try await client.fetchCostSummary(provider: provider)
            summaries.append([
                "provider": summary.provider,
                "today_cost_usd": summary.summary.todayCostUSD,
                "today_tokens": summary.summary.todayTokens,
                "last_30_days_cost_usd": summary.summary.last30DaysCostUSD,
                "last_30_days_tokens": summary.summary.last30DaysTokens,
                "preferred_source": options.preferredSource.rawValue,
            ])
        }

        if options.format == .json {
            try self.writeJSON(["providers": summaries], pretty: options.pretty)
            return
        }

        for summary in summaries {
            print("== \((summary["provider"] as? String ?? "").capitalized) ==")
            print("Preferred source: \(summary["preferred_source"] as? String ?? "auto")")
            print(String(format: "Today: $%.2f", summary["today_cost_usd"] as? Double ?? 0))
            print("Today tokens: \(summary["today_tokens"] as? Int ?? 0)")
            print(String(format: "Last 30d: $%.2f", summary["last_30_days_cost_usd"] as? Double ?? 0))
            print("Last 30d tokens: \(summary["last_30_days_tokens"] as? Int ?? 0)")
            print("")
        }
    }

    private static func runConfig(arguments: [String]) throws {
        let action = arguments.first ?? "validate"
        let options = try self.parseOptions(Array(arguments.dropFirst()), allowProvider: false)
        let store = ConfigStore.shared

        switch action {
        case "validate":
            try store.validate()
            if options.format == .json {
                try self.writeJSON(["valid": true], pretty: options.pretty)
            } else {
                print("valid")
            }
        case "dump":
            let config = store.load()
            let encoder = JSONEncoder()
            encoder.outputFormatting = options.pretty ? [.prettyPrinted, .sortedKeys] : [.sortedKeys]
            encoder.keyEncodingStrategy = .convertToSnakeCase
            let data = try encoder.encode(config)
            FileHandle.standardOutput.write(data)
            FileHandle.standardOutput.write(Data("\n".utf8))
        default:
            throw CLIError.invalidArguments("config expects validate or dump")
        }
    }

    private static func parseOptions(
        _ arguments: [String],
        allowProvider: Bool = true
    ) throws -> CLIOptions {
        var options = CLIOptions()
        var index = 0

        while index < arguments.count {
            let argument = arguments[index]
            switch argument {
            case "--provider":
                guard allowProvider else {
                    throw CLIError.invalidArguments("--provider is not valid for this command")
                }
                index += 1
                guard index < arguments.count else {
                    throw CLIError.invalidArguments("--provider requires a value")
                }
                switch arguments[index] {
                case "claude":
                    options.providers = [.claude]
                case "codex":
                    options.providers = [.codex]
                case "both":
                    options.providers = ProviderID.allCases
                default:
                    throw CLIError.invalidArguments("unsupported provider: \(arguments[index])")
                }
            case "--source":
                index += 1
                guard index < arguments.count,
                      let source = UsageSourcePreference(rawValue: arguments[index]) else {
                    throw CLIError.invalidArguments("--source requires auto|oauth|web|cli")
                }
                options.preferredSource = source
            case "--format":
                index += 1
                guard index < arguments.count,
                      let format = CLIOutputFormat(rawValue: arguments[index]) else {
                    throw CLIError.invalidArguments("--format requires text|json")
                }
                options.format = format
            case "--pretty":
                options.pretty = true
                options.format = .json
            case "--status":
                options.includeStatus = true
            case "--refresh":
                options.refresh = true
            default:
                throw CLIError.invalidArguments("unknown option: \(argument)")
            }
            index += 1
        }

        return options
    }

    private static func snapshotDictionary(includeStatus: Bool) -> (ProviderSnapshot) -> [String: Any] {
        { snapshot in
            var value: [String: Any] = [
                "provider": snapshot.provider,
                "available": snapshot.available,
                "source_used": snapshot.sourceUsed,
                "today_cost_usd": snapshot.costSummary.todayCostUSD,
                "last_30_days_cost_usd": snapshot.costSummary.last30DaysCostUSD,
                "today_tokens": snapshot.costSummary.todayTokens,
                "last_30_days_tokens": snapshot.costSummary.last30DaysTokens,
                "stale": snapshot.stale,
            ]
            if let credits = snapshot.credits {
                value["credits"] = credits
            }
            if includeStatus, let status = snapshot.status {
                value["status"] = [
                    "indicator": status.indicator,
                    "description": status.description,
                    "page_url": status.pageURL,
                ]
            }
            return value
        }
    }

    private static func writeJSON(_ object: Any, pretty: Bool) throws {
        let options: JSONSerialization.WritingOptions = pretty ? [.prettyPrinted, .sortedKeys] : [.sortedKeys]
        let data = try JSONSerialization.data(withJSONObject: object, options: options)
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
