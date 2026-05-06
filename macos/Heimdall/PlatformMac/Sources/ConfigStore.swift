import Foundation
import HeimdallDomain
import HeimdallServices

public final class ConfigStore: @unchecked Sendable, SettingsStore {
    public static let shared = ConfigStore()

    public let url: URL
    private let encoder = JSONEncoder()
    private let decoder = JSONDecoder()

    public init(fileManager: FileManager = .default) {
        let base = fileManager.homeDirectoryForCurrentUser
            .appendingPathComponent(".config", isDirectory: true)
            .appendingPathComponent("heimdall", isDirectory: true)
        self.url = base.appendingPathComponent("config.json", isDirectory: false)
        self.encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        self.decoder.keyDecodingStrategy = .convertFromSnakeCase
        self.encoder.keyEncodingStrategy = .convertToSnakeCase
    }

    /// Initializer for testing: supply an explicit file URL instead of
    /// defaulting to `~/.config/heimdall/config.json`.
    public init(url: URL) {
        self.url = url
        self.encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        self.decoder.keyDecodingStrategy = .convertFromSnakeCase
        self.encoder.keyEncodingStrategy = .convertToSnakeCase
    }

    public func load() -> HeimdallConfig {
        guard let data = try? Data(contentsOf: self.url),
              let config = try? self.decoder.decode(HeimdallConfig.self, from: data) else {
            return .default
        }
        return config
    }

    public func save(_ config: HeimdallConfig) throws {
        let dir = self.url.deletingLastPathComponent()
        try FileManager.default.createDirectory(
            at: dir,
            withIntermediateDirectories: true,
            attributes: nil
        )

        // Encode the Swift config to its snake_case dictionary form.
        let swiftData = try self.encoder.encode(config)
        guard let swiftDict = try JSONSerialization.jsonObject(with: swiftData) as? [String: Any] else {
            throw CocoaError(.fileWriteUnknown)
        }

        // Load the existing on-disk JSON (if any) so we can preserve keys that
        // the Swift type doesn't model (e.g. Rust-only fields like `statusline`,
        // `webhooks`, `pricing`). Invalid or missing files are treated as empty.
        var onDiskDict: [String: Any]
        if let existingData = try? Data(contentsOf: self.url),
           let parsed = try? JSONSerialization.jsonObject(with: existingData) as? [String: Any] {
            onDiskDict = parsed
        } else {
            onDiskDict = [:]
        }

        // Merge: Swift-known keys overwrite on-disk values; all other on-disk
        // keys are left untouched. Shallow merge at the top level — when Swift
        // models a section it owns that section entirely.
        for (key, value) in swiftDict {
            onDiskDict[key] = value
        }

        // Re-serialize with stable formatting (pretty + sorted keys) that
        // matches the existing encoder output.
        let mergedData = try JSONSerialization.data(
            withJSONObject: onDiskDict,
            options: [.prettyPrinted, .sortedKeys]
        )
        try mergedData.write(to: self.url, options: .atomic)
    }

    public func validate() throws {
        _ = self.load()
    }
}
