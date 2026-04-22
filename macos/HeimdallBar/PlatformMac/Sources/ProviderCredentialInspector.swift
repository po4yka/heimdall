import Foundation
import HeimdallDomain
import HeimdallServices

public struct ProviderCredentialInspector: ProviderCredentialInspecting, Sendable {
    private let fileExistsAtPath: @Sendable (String) -> Bool
    private let homeDirectoryProvider: @Sendable () -> URL
    private let codexHomeProvider: @Sendable () -> URL
    private let claudeKeychainCredentialProvider: @Sendable () -> Bool
    private let codexCredentialStoreProvider: @Sendable () -> String?

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
        },
        claudeKeychainCredentialProvider: @escaping @Sendable () -> Bool = {
            Self.keychainContainsClaudeCredentials()
        },
        codexCredentialStoreProvider: @escaping @Sendable () -> String? = {
            Self.codexCredentialStore(codexHome: {
                if let customCodexHome = ProcessInfo.processInfo.environment["CODEX_HOME"], !customCodexHome.isEmpty {
                    return URL(fileURLWithPath: customCodexHome, isDirectory: true)
                }
                return URL(fileURLWithPath: NSHomeDirectory(), isDirectory: true)
                    .appendingPathComponent(".codex", isDirectory: true)
            }())
        }
    ) {
        self.fileExistsAtPath = fileExistsAtPath
        self.homeDirectoryProvider = homeDirectoryProvider
        self.codexHomeProvider = codexHomeProvider
        self.claudeKeychainCredentialProvider = claudeKeychainCredentialProvider
        self.codexCredentialStoreProvider = codexCredentialStoreProvider
    }

    public func credentialPresence(for provider: ProviderID) -> ProviderCredentialPresence {
        switch provider {
        case .claude:
            let credentialsURL = self.homeDirectoryProvider()
                .appendingPathComponent(".claude", isDirectory: true)
                .appendingPathComponent(".credentials.json", isDirectory: false)
            if self.claudeKeychainCredentialProvider() || self.fileExistsAtPath(credentialsURL.path) {
                return .present
            }
            return .missing
        case .codex:
            let authURL = self.codexHomeProvider().appendingPathComponent("auth.json", isDirectory: false)
            let store = self.codexCredentialStoreProvider()?.lowercased()
            if store == "keyring" || store == "auto" {
                return .present
            }
            return self.fileExistsAtPath(authURL.path) ? .present : .missing
        }
    }

    public static func keychainContainsClaudeCredentials() -> Bool {
        for service in ["Claude Code-credentials", "Claude Code"] {
            let process = Process()
            process.executableURL = URL(fileURLWithPath: "/usr/bin/security")
            process.arguments = ["find-generic-password", "-s", service]
            let output = Pipe()
            process.standardOutput = output
            process.standardError = output
            do {
                try process.run()
                process.waitUntilExit()
                if process.terminationStatus == 0 {
                    return true
                }
            } catch {
                return false
            }
        }
        return false
    }

    public static func codexCredentialStore(codexHome: URL) -> String? {
        let configURL = codexHome.appendingPathComponent("config.toml", isDirectory: false)
        guard let contents = try? String(contentsOf: configURL, encoding: .utf8) else {
            return nil
        }
        let pattern = #"(?m)^\s*cli_auth_credentials_store\s*=\s*"?([A-Za-z-]+)"?\s*$"#
        guard let regex = try? NSRegularExpression(pattern: pattern) else {
            return nil
        }
        let range = NSRange(contents.startIndex..<contents.endIndex, in: contents)
        guard let match = regex.firstMatch(in: contents, options: [], range: range),
              let storeRange = Range(match.range(at: 1), in: contents) else {
            return nil
        }
        return String(contents[storeRange])
    }
}
