import Foundation

public actor DashboardAdjunctController {
    private let keychainStore: KeychainStore

    public init(keychainStore: KeychainStore = KeychainStore()) {
        self.keychainStore = keychainStore
    }

    public func loadAdjunct(
        provider: ProviderID,
        config: ProviderConfig,
        snapshot: ProviderSnapshot?
    ) async -> DashboardAdjunctSnapshot? {
        guard config.dashboardExtrasEnabled else { return nil }

        let account = "\(provider.rawValue).web-session"
        let storedSession = self.keychainStore.load(account: account)
        let source = config.cookieSource == .auto ? .web : config.cookieSource
        let hasStoredSession = storedSession != nil
        let headline = await MainActor.run { () -> String in
            let scraper = WebDashboardScraper()
            scraper.warm()
            return scraper.statusMessage(provider: provider, hasStoredSession: hasStoredSession)
        }

        var detailLines = [String]()
        if let snapshot {
            detailLines.append("Live source: \(snapshot.sourceUsed)")
            if let credits = snapshot.credits {
                detailLines.append(String(format: "Credits: %.2f", credits))
            }
        }

        if hasStoredSession {
            detailLines.append("Stored browser session found in Keychain.")
            detailLines.append("Hidden WebKit refresh pipeline is ready for provider extras.")
        } else {
            detailLines.append("Import browser cookies to unlock web-only dashboard details.")
            detailLines.append("Web extras stay opt-in and local to this machine.")
        }

        return DashboardAdjunctSnapshot(
            provider: provider,
            source: source,
            headline: headline,
            detailLines: detailLines,
            statusText: hasStoredSession ? "ready" : "login-required",
            isLoginRequired: !hasStoredSession,
            lastUpdated: ISO8601DateFormatter().string(from: Date())
        )
    }

    public func importBrowserSession(provider: ProviderID, payload: Data) throws {
        try self.keychainStore.save(payload, account: "\(provider.rawValue).web-session")
    }
}
