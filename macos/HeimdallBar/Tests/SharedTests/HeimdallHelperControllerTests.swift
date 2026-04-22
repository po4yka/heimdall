import Foundation
import Testing
@testable import HeimdallPlatformMac

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
    func liveProvidersPayloadCompatibilityRequiresMatchingContractVersion() {
        let compatible = Data(#"{"contract_version":1,"providers":[{"provider":"claude"},{"provider":"codex"}]}"#.utf8)
        let incompatible = Data(#"{"contract_version":999,"providers":[{"provider":"claude"},{"provider":"codex"}]}"#.utf8)

        #expect(HeimdallHelperController.liveProvidersPayloadIsCompatible(compatible))
        #expect(!HeimdallHelperController.liveProvidersPayloadIsCompatible(incompatible))
    }

    @Test
    func compatibleHealthyServerCanBeReusedAcrossLocalBuildPaths() {
        #expect(
            HeimdallHelperController.canReuseExistingServer(
                isHealthy: true,
                compatibility: .compatible
            )
        )
        #expect(
            !HeimdallHelperController.canReuseExistingServer(
                isHealthy: true,
                compatibility: .incompatible
            )
        )
        #expect(
            !HeimdallHelperController.canReuseExistingServer(
                isHealthy: false,
                compatibility: .compatible
            )
        )
    }

    @Test
    func trustedListenerRequiresMatchingExecutablePathAndFingerprint() throws {
        let temp = try Self.makeTempDirectory()
        let executable = temp.appendingPathComponent("claude-usage-tracker")
        FileManager.default.createFile(atPath: executable.path, contents: Data("trusted".utf8))
        try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: executable.path)
        let fingerprint = try #require(HeimdallHelperController.fingerprint(for: executable))

        let trusted = HeimdallHelperController.hasTrustedListener(
            on: 8787,
            expectedExecutable: executable,
            expectedFingerprint: fingerprint,
            pidsProvider: { _ in [42] },
            pathProvider: { _ in executable.path }
        )
        let untrusted = HeimdallHelperController.hasTrustedListener(
            on: 8787,
            expectedExecutable: executable,
            expectedFingerprint: fingerprint,
            pidsProvider: { _ in [42] },
            pathProvider: { _ in "/tmp/other-process" }
        )

        #expect(trusted)
        #expect(!untrusted)
    }

    private static func makeTempDirectory() throws -> URL {
        let base = FileManager.default.temporaryDirectory
        let url = base.appendingPathComponent(UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(at: url, withIntermediateDirectories: true)
        return url
    }
}
