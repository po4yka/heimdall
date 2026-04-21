import Foundation
import HeimdallDomain
import HeimdallServices

public struct ProviderCredentialInspector: ProviderCredentialInspecting, Sendable {
    private let fileExistsAtPath: @Sendable (String) -> Bool
    private let homeDirectoryProvider: @Sendable () -> URL
    private let codexHomeProvider: @Sendable () -> URL

    public init(
        fileExistsAtPath: @escaping @Sendable (String) -> Bool = { path in
            FileManager.default.fileExists(atPath: path)
        },
        homeDirectoryProvider: @escaping @Sendable () -> URL = {
            URL(fileURLWithPath: NSHomeDirectory(), isDirectory: true)
        },
        codexHomeProvider: @escaping @Sendable () -> URL = {
            if let customCodexHome = ProcessInfo.processInfo.environment["CODEX_HOME"], !customCodexHome.isEmpty {
                return URL(fileURLWithPath: customCodexHome, isDirectory: true)
            }
            return URL(fileURLWithPath: NSHomeDirectory(), isDirectory: true)
                .appendingPathComponent(".codex", isDirectory: true)
        }
    ) {
        self.fileExistsAtPath = fileExistsAtPath
        self.homeDirectoryProvider = homeDirectoryProvider
        self.codexHomeProvider = codexHomeProvider
    }

    public func credentialPresence(for provider: ProviderID) -> ProviderCredentialPresence {
        let url = switch provider {
        case .claude:
            self.homeDirectoryProvider()
                .appendingPathComponent(".claude", isDirectory: true)
                .appendingPathComponent(".credentials.json", isDirectory: false)
        case .codex:
            self.codexHomeProvider()
                .appendingPathComponent("auth.json", isDirectory: false)
        }
        return self.fileExistsAtPath(url.path) ? .present : .missing
    }
}
