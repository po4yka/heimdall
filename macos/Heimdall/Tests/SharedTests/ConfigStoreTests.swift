import Foundation
import HeimdallDomain
import HeimdallPlatformMac
import Testing

/// Tests for `ConfigStore.save(_:)` merge semantics.
///
/// Core invariant: saving a `HeimdallConfig` must preserve any top-level JSON
/// keys that the Swift type does not model (e.g. Rust-only keys like
/// `statusline`, `webhooks`, `pricing`).  A naive `JSONEncoder.encode` +
/// `Data.write` would silently drop those keys on every save.
struct ConfigStoreTests {

    // MARK: - Helpers

    /// Returns a URL inside a unique temp directory.  The directory is created;
    /// the file itself does not exist yet.
    private func makeTempURL(filename: String = "config.json") throws -> URL {
        let dir = FileManager.default.temporaryDirectory
            .appendingPathComponent("ConfigStoreTests-\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        return dir.appendingPathComponent(filename)
    }

    /// Writes raw JSON bytes to `url`.
    private func writeJSON(_ object: Any, to url: URL) throws {
        let data = try JSONSerialization.data(withJSONObject: object, options: [.prettyPrinted, .sortedKeys])
        try data.write(to: url, options: .atomic)
    }

    /// Reads the file at `url` and parses it as a top-level JSON dictionary.
    private func readJSON(at url: URL) throws -> [String: Any] {
        let data = try Data(contentsOf: url)
        guard let dict = try JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            Issue.record("Expected a JSON dictionary at \(url.path)")
            return [:]
        }
        return dict
    }

    // MARK: - testSavePreservesUnknownTopLevelKeys

    /// The critical regression guard: Rust-only keys must survive a Swift save.
    ///
    /// Pre-condition on disk (simulates a file previously written by the Rust
    /// binary):
    ///   {
    ///     "merge_icons": true,
    ///     "statusline": {"context_low_threshold": 0.5},
    ///     "macos_only_thing": {"foo": "bar"}
    ///   }
    ///
    /// After loading and flipping `mergeIcons` to `false` via the Swift type,
    /// saving must:
    ///  - apply `merge_icons = false`              (Swift key updated)
    ///  - keep `statusline.context_low_threshold`  (Rust-only key preserved)
    ///  - keep `macos_only_thing.foo`              (unknown key preserved)
    @Test
    func savePreservesUnknownTopLevelKeys() throws {
        let url = try makeTempURL()

        // Write the "Rust-side" file with mixed known + unknown keys.
        let initial: [String: Any] = [
            "merge_icons": true,
            "statusline": ["context_low_threshold": 0.5],
            "macos_only_thing": ["foo": "bar"],
        ]
        try writeJSON(initial, to: url)

        let store = ConfigStore(url: url)

        // Load picks up the existing `merge_icons = true`.
        var config = store.load()
        #expect(config.mergeIcons == true)

        // Flip the Swift-known field.
        config.mergeIcons = false
        try store.save(config)

        // Read back the raw on-disk JSON.
        let result = try readJSON(at: url)

        // Swift-side change must be applied.
        #expect(result["merge_icons"] as? Bool == false)

        // Rust-only sub-object must survive untouched.
        let statusline = result["statusline"] as? [String: Any]
        #expect(statusline != nil)
        #expect(statusline?["context_low_threshold"] as? Double == 0.5)

        // Completely unknown key must survive.
        let macosOnlyThing = result["macos_only_thing"] as? [String: Any]
        #expect(macosOnlyThing != nil)
        #expect(macosOnlyThing?["foo"] as? String == "bar")
    }

    // MARK: - testSaveCreatesFileIfMissing

    /// Saving to a non-existent path must create the file with valid JSON.
    @Test
    func saveCreatesFileIfMissing() throws {
        let url = try makeTempURL()
        // File must not exist before the test.
        #expect(!FileManager.default.fileExists(atPath: url.path))

        let store = ConfigStore(url: url)
        try store.save(.default)

        #expect(FileManager.default.fileExists(atPath: url.path))
        let data = try Data(contentsOf: url)
        #expect(!data.isEmpty)
        // Must parse as valid JSON.
        let parsed = try JSONSerialization.jsonObject(with: data)
        #expect(parsed is [String: Any])
    }

    // MARK: - testSaveOverwritesExistingSwiftKeys

    /// Saving an updated config must overwrite the matching Swift-modeled key.
    @Test
    func saveOverwritesExistingSwiftKeys() throws {
        let url = try makeTempURL()

        let store = ConfigStore(url: url)

        // First save: mergeIcons = true.
        var config = HeimdallConfig.default
        config.mergeIcons = true
        try store.save(config)

        // Second save: mergeIcons = false.
        config.mergeIcons = false
        try store.save(config)

        let result = try readJSON(at: url)
        #expect(result["merge_icons"] as? Bool == false)
    }

    // MARK: - testSaveAtomicityViaCorruptOnDisk

    /// When the on-disk file contains garbage (non-JSON) bytes, save must
    /// succeed by treating the corrupt file as an empty dictionary — the merge
    /// fallback path.  The resulting file must contain valid JSON.
    @Test
    func saveAtomicityViaCorruptOnDisk() throws {
        let url = try makeTempURL()

        // Write non-JSON garbage.
        let garbage = Data("NOT_VALID_JSON!!!@#$%".utf8)
        try garbage.write(to: url, options: .atomic)

        let store = ConfigStore(url: url)

        // Save must not throw — corrupt file is treated as empty dict.
        #expect(throws: Never.self) {
            try store.save(.default)
        }

        // Result must be parseable JSON.
        let data = try Data(contentsOf: url)
        let parsed = try JSONSerialization.jsonObject(with: data)
        #expect(parsed is [String: Any])
    }
}
