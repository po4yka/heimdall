import Foundation
import HeimdallDomain
import HeimdallPlatformMac
import Testing

/// Tests for `HeimdallConfig` Codable implementation — M6 parity additions.
///
/// Core invariants:
///  - Decoding `{}` yields the documented defaults for every M6 field.
///  - Round-tripping a non-default config preserves all values.
///  - The `aggregator` Swift property serializes under the `status_aggregator`
///    JSON wire key (mirrors Rust `#[serde(rename = "status_aggregator")]`).
///  - Saving via `ConfigStore` preserves unknown top-level keys (collision
///    survival across the new M6 schema additions).
struct HeimdallConfigCodingTests {

    // MARK: - Helpers

    private func makeTempURL(filename: String = "config.json") throws -> URL {
        let dir = FileManager.default.temporaryDirectory
            .appendingPathComponent("HeimdallConfigCodingTests-\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        return dir.appendingPathComponent(filename)
    }

    private func writeJSON(_ object: Any, to url: URL) throws {
        let data = try JSONSerialization.data(withJSONObject: object, options: [.prettyPrinted, .sortedKeys])
        try data.write(to: url, options: .atomic)
    }

    private func readJSON(at url: URL) throws -> [String: Any] {
        let data = try Data(contentsOf: url)
        guard let dict = try JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            Issue.record("Expected a JSON dictionary at \(url.path)")
            return [:]
        }
        return dict
    }

    private func makeDecoder() -> JSONDecoder {
        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        return decoder
    }

    private func makeEncoder() -> JSONEncoder {
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        return encoder
    }

    // MARK: - decodesEmptyJSONToDefaults

    /// Decoding `{}` must succeed and apply default values for all M6 fields.
    /// Existing macOS users with old config files must not crash.
    @Test
    func decodesEmptyJSONToDefaults() throws {
        let json = "{}".data(using: .utf8)!
        let config = try self.makeDecoder().decode(HeimdallConfig.self, from: json)

        // M6 sub-types fall back to their documented defaults.
        #expect(config.display.currency == nil)
        #expect(config.display.locale == nil)
        #expect(config.display.compact == nil)
        #expect(config.oauth.enabled == true)
        #expect(config.oauth.refreshInterval == 60)
        #expect(config.claudeAdmin.enabled == true)
        #expect(config.claudeAdmin.refreshInterval == 300)
        #expect(config.claudeAdmin.lookbackDays == 30)
        #expect(config.openai.enabled == true)
        #expect(config.openai.refreshInterval == 300)
        #expect(config.openai.lookbackDays == 30)
        #expect(config.agentStatus.enabled == true)
        #expect(config.agentStatus.refreshInterval == 60)
        #expect(config.agentStatus.claudeEnabled == true)
        #expect(config.agentStatus.openaiEnabled == true)
        #expect(config.agentStatus.alertMinSeverity == .major)
        #expect(config.aggregator.enabled == false)
        #expect(config.aggregator.refreshInterval == 300)
        #expect(config.aggregator.spikeWebhook == true)
        #expect(config.blocks.tokenLimit == nil)
        #expect(config.blocks.sessionLengthHours == nil)
        #expect(config.statusline.contextLowThreshold == 0.5)
        #expect(config.statusline.contextMediumThreshold == 0.8)
        #expect(config.statusline.burnRateNormalMax == 4000)
        #expect(config.statusline.burnRateModerateMax == 10000)
        #expect(config.webhooks.url == nil)
        #expect(config.webhooks.costThreshold == nil)
        #expect(config.webhooks.sessionDepleted == false)
        #expect(config.webhooks.agentStatus == true)
        #expect(config.webhooks.spikeWebhook == true)
        #expect(config.webhooks.capChanges == true)
        #expect(config.webhooks.agentStopReason == true)
        #expect(config.webhooks.agentStopReasonFilter == nil)
        #expect(config.projectAliases.isEmpty)
        #expect(config.pricing.isEmpty)

        // Pre-existing macOS-only fields keep their original defaults.
        #expect(config.mergeIcons == HeimdallConfig.default.mergeIcons)
        #expect(config.helperPort == HeimdallConfig.default.helperPort)
    }

    // MARK: - roundTripsAllNewFields

    /// Encode -> decode round-trip preserves every M6 field with non-default values.
    @Test
    func roundTripsAllNewFields() throws {
        var config = HeimdallConfig.default
        config.display = HeimdallDisplay(currency: "EUR", locale: "fr-FR", compact: true)
        config.oauth = HeimdallOAuthConfig(enabled: false, refreshInterval: 120)
        config.claudeAdmin = HeimdallClaudeAdminConfig(enabled: false, refreshInterval: 600, lookbackDays: 60)
        config.openai = HeimdallOpenAiConfig(enabled: false, refreshInterval: 600, lookbackDays: 90)
        config.agentStatus = HeimdallAgentStatusConfig(
            enabled: false,
            refreshInterval: 30,
            claudeEnabled: false,
            openaiEnabled: false,
            alertMinSeverity: .critical
        )
        config.aggregator = HeimdallAggregatorConfig(enabled: true, refreshInterval: 600, spikeWebhook: false)
        config.blocks = HeimdallBlocksConfig(tokenLimit: 500_000, sessionLengthHours: 5.5)
        config.statusline = HeimdallStatuslineConfig(
            contextLowThreshold: 0.4,
            contextMediumThreshold: 0.7,
            burnRateNormalMax: 5000,
            burnRateModerateMax: 12000
        )
        config.webhooks = HeimdallWebhookConfig(
            url: "https://example.test/webhook",
            costThreshold: 25.5,
            sessionDepleted: true,
            agentStatus: false,
            spikeWebhook: false,
            capChanges: false,
            agentStopReason: false,
            agentStopReasonFilter: ["error", "max_tokens"]
        )
        config.projectAliases = [
            "-Users-foo-bar": "Bar Project",
            "-Users-foo-baz": "Baz Project",
        ]
        config.pricing = [
            "claude-sonnet-4-5": HeimdallPricingOverride(
                input: 3.0, output: 15.0, cacheWrite: 3.75, cacheRead: 0.30
            ),
            "gpt-4o-mini": HeimdallPricingOverride(input: 0.15, output: 0.60),
        ]

        let data = try self.makeEncoder().encode(config)
        let decoded = try self.makeDecoder().decode(HeimdallConfig.self, from: data)

        #expect(decoded.display.currency == "EUR")
        #expect(decoded.display.locale == "fr-FR")
        #expect(decoded.display.compact == true)
        #expect(decoded.oauth == config.oauth)
        #expect(decoded.claudeAdmin == config.claudeAdmin)
        #expect(decoded.openai == config.openai)
        #expect(decoded.agentStatus == config.agentStatus)
        #expect(decoded.aggregator == config.aggregator)
        #expect(decoded.blocks == config.blocks)
        #expect(decoded.statusline == config.statusline)
        #expect(decoded.webhooks == config.webhooks)
        #expect(decoded.projectAliases == config.projectAliases)
        #expect(decoded.pricing == config.pricing)
    }

    // MARK: - aggregatorJSONKeyIsStatusAggregator

    /// Rust uses `#[serde(rename = "status_aggregator")]`, so the on-disk JSON
    /// key MUST be `status_aggregator`, NOT `aggregator`.
    @Test
    func aggregatorJSONKeyIsStatusAggregator() throws {
        var config = HeimdallConfig.default
        config.aggregator = HeimdallAggregatorConfig(enabled: true, refreshInterval: 300, spikeWebhook: true)

        let data = try self.makeEncoder().encode(config)
        guard let dict = try JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            Issue.record("Expected JSON dict")
            return
        }

        #expect(dict["status_aggregator"] != nil, "Expected top-level wire key 'status_aggregator'")
        #expect(dict["aggregator"] == nil, "Must not emit a bare 'aggregator' key")

        let nested = dict["status_aggregator"] as? [String: Any]
        #expect(nested?["enabled"] as? Bool == true)
        #expect((nested?["refresh_interval"] as? NSNumber)?.intValue == 300)
        #expect(nested?["spike_webhook"] as? Bool == true)
    }

    // MARK: - unknownTopLevelKeysSurviveSaveLoad

    /// Extends the M1 collision-survival guarantee for the M6 schema additions.
    /// A Rust-only field MUST survive a Swift-side save+reload round-trip.
    @Test
    func unknownTopLevelKeysSurviveSaveLoad() throws {
        let url = try self.makeTempURL()

        // Pre-write a file simulating a Rust-written config with a key that
        // Swift's HeimdallConfig does not model.
        let initial: [String: Any] = [
            "merge_icons": true,
            "projects_dirs": ["/tmp/foo", "/tmp/bar"],
            "status_aggregator": [
                "enabled": true,
                "refresh_interval": 600,
                "spike_webhook": false,
            ],
            "macos_unknown_section": ["nested": ["k": "v"]],
        ]
        try self.writeJSON(initial, to: url)

        let store = ConfigStore(url: url)

        // Load picks up the Swift-known fields, including the new M6 ones.
        var config = store.load()
        #expect(config.aggregator.enabled == true)
        #expect(config.aggregator.refreshInterval == 600)
        #expect(config.aggregator.spikeWebhook == false)

        // Mutate a Swift-known M6 field and save.
        config.display.currency = "GBP"
        config.projectAliases = ["-Users-foo": "Foo"]
        try store.save(config)

        // Read back raw on-disk JSON and verify unknowns persist alongside
        // the new Swift-modeled M6 sections.
        let result = try self.readJSON(at: url)

        // Unknown Rust-only keys preserved.
        let projectsDirs = result["projects_dirs"] as? [String]
        #expect(projectsDirs == ["/tmp/foo", "/tmp/bar"])

        let macosUnknown = result["macos_unknown_section"] as? [String: Any]
        #expect(macosUnknown != nil)
        let nested = macosUnknown?["nested"] as? [String: Any]
        #expect(nested?["k"] as? String == "v")

        // Swift-mutated M6 fields applied via the `status_aggregator` wire key.
        let statusAggregator = result["status_aggregator"] as? [String: Any]
        #expect(statusAggregator != nil)
        #expect(statusAggregator?["enabled"] as? Bool == true)

        let display = result["display"] as? [String: Any]
        #expect(display?["currency"] as? String == "GBP")

        let aliases = result["project_aliases"] as? [String: Any]
        #expect(aliases?["-Users-foo"] as? String == "Foo")

        // Reload via the store: M6 fields decode correctly and unknowns survive
        // (we cannot verify unknowns through the typed config, but the raw
        // dict assertions above already proved they're on disk).
        let reloaded = store.load()
        #expect(reloaded.display.currency == "GBP")
        #expect(reloaded.aggregator.enabled == true)
        #expect(reloaded.projectAliases["-Users-foo"] == "Foo")
    }
}
