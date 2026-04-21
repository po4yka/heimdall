import Foundation
import HeimdallDomain
import HeimdallServices
import Testing
@testable import HeimdallPlatformMac

struct ProviderCredentialInspectorTests {
    @Test
    func reportsMissingCredentialsWhenProviderFilesDoNotExist() throws {
        let temp = try Self.makeTempDirectory()
        let inspector = ProviderCredentialInspector(
            homeDirectoryProvider: { temp },
            codexHomeProvider: { temp.appendingPathComponent(".codex", isDirectory: true) }
        )

        #expect(inspector.credentialPresence(for: .claude) == .missing)
        #expect(inspector.credentialPresence(for: .codex) == .missing)
    }

    @Test
    func reportsPresentCredentialsWhenProviderFilesExist() throws {
        let temp = try Self.makeTempDirectory()
        let claudeDirectory = temp.appendingPathComponent(".claude", isDirectory: true)
        let codexDirectory = temp.appendingPathComponent(".codex", isDirectory: true)
        try FileManager.default.createDirectory(at: claudeDirectory, withIntermediateDirectories: true)
        try FileManager.default.createDirectory(at: codexDirectory, withIntermediateDirectories: true)
        FileManager.default.createFile(
            atPath: claudeDirectory.appendingPathComponent(".credentials.json").path,
            contents: Data("{}".utf8)
        )
        FileManager.default.createFile(
            atPath: codexDirectory.appendingPathComponent("auth.json").path,
            contents: Data("{}".utf8)
        )

        let inspector = ProviderCredentialInspector(
            homeDirectoryProvider: { temp },
            codexHomeProvider: { codexDirectory }
        )

        #expect(inspector.credentialPresence(for: .claude) == .present)
        #expect(inspector.credentialPresence(for: .codex) == .present)
    }

    private static func makeTempDirectory() throws -> URL {
        let url = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(at: url, withIntermediateDirectories: true)
        return url
    }
}
