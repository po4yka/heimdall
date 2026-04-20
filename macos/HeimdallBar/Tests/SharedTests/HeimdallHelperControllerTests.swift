import Foundation
import Testing
@testable import HeimdallBarShared

struct HeimdallHelperControllerTests {
    @Test
    func resolveExecutablePrefersBundledHelperBeforePATHFallback() throws {
        let temp = try Self.makeTempDirectory()
        let bundleURL = temp.appendingPathComponent("HeimdallBar.app", isDirectory: true)
        let bundled = HeimdallHelperController.bundledHelperURL(bundleURL: bundleURL)
        try FileManager.default.createDirectory(at: bundled.deletingLastPathComponent(), withIntermediateDirectories: true)
        FileManager.default.createFile(atPath: bundled.path, contents: Data("bundled".utf8))
        try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: bundled.path)

        let pathDir = temp.appendingPathComponent("bin", isDirectory: true)
        try FileManager.default.createDirectory(at: pathDir, withIntermediateDirectories: true)
        let pathHelper = pathDir.appendingPathComponent("claude-usage-tracker")
        FileManager.default.createFile(atPath: pathHelper.path, contents: Data("path".utf8))
        try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: pathHelper.path)

        let resolved = HeimdallHelperController.resolveExecutable(
            bundleURL: bundleURL,
            pathEnv: pathDir.path
        )

        #expect(resolved?.path == bundled.path)
    }

    @Test
    func resolveExecutableUsesPATHOnlyForDevelopmentBundles() throws {
        let temp = try Self.makeTempDirectory()
        let releaseBundle = temp.appendingPathComponent("Applications/HeimdallBar.app", isDirectory: true)
        let derivedBundle = temp.appendingPathComponent(".derived/Build/Products/Debug/HeimdallBar.app", isDirectory: true)
        let pathDir = temp.appendingPathComponent("bin", isDirectory: true)
        try FileManager.default.createDirectory(at: pathDir, withIntermediateDirectories: true)
        let pathHelper = pathDir.appendingPathComponent("claude-usage-tracker")
        FileManager.default.createFile(atPath: pathHelper.path, contents: Data("path".utf8))
        try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: pathHelper.path)

        let releaseResolved = HeimdallHelperController.resolveExecutable(
            bundleURL: releaseBundle,
            pathEnv: pathDir.path
        )
        let developmentResolved = HeimdallHelperController.resolveExecutable(
            bundleURL: derivedBundle,
            pathEnv: pathDir.path
        )

        #expect(releaseResolved == nil)
        #expect(developmentResolved?.path == pathHelper.path)
    }

    @Test
    func fingerprintTracksPathSizeAndModificationTime() throws {
        let temp = try Self.makeTempDirectory()
        let executable = temp.appendingPathComponent("claude-usage-tracker")
        FileManager.default.createFile(atPath: executable.path, contents: Data("12345".utf8))
        try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: executable.path)

        let fingerprint = HeimdallHelperController.fingerprint(for: executable)

        #expect(fingerprint?.path == executable.path)
        #expect(fingerprint?.size == 5)
        #expect(fingerprint?.modificationTimeInterval != nil)
    }

    @Test
    func liveProvidersPayloadCompatibilityRequiresAuthForEveryProvider() {
        let compatible = Data(#"{"providers":[{"provider":"claude","auth":{"diagnostic_code":"ok"}},{"provider":"codex","auth":{"diagnostic_code":"ok"}}]}"#.utf8)
        let incompatible = Data(#"{"providers":[{"provider":"claude"},{"provider":"codex","auth":{"diagnostic_code":"ok"}}]}"#.utf8)

        #expect(HeimdallHelperController.liveProvidersPayloadIncludesAuth(compatible))
        #expect(!HeimdallHelperController.liveProvidersPayloadIncludesAuth(incompatible))
    }

    private static func makeTempDirectory() throws -> URL {
        let base = FileManager.default.temporaryDirectory
        let url = base.appendingPathComponent(UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(at: url, withIntermediateDirectories: true)
        return url
    }
}
