import Foundation
import HeimdallDomain
import HeimdallServices

public actor DashboardAdjunctController: DashboardAdjunctLoading {
    private let sessionManager: any BrowserSessionManaging

    public init(
        sessionManager: any BrowserSessionManaging
    ) {
        self.sessionManager = sessionManager
    }

    public func loadAdjunct(
        provider: ProviderID,
        config: ProviderConfig,
        snapshot: ProviderSnapshot?,
        forceRefresh: Bool = false,
        allowLiveNavigation: Bool = true
    ) async -> DashboardAdjunctSnapshot? {
        guard config.dashboardExtrasEnabled else { return nil }

        let importedSession = await self.sessionManager.importedSession(provider: provider)
        let source = config.cookieSource == .auto ? .web : config.cookieSource
        let scraper = await MainActor.run { WebDashboardScraper() }
        await MainActor.run {
            scraper.warm()
        }
        let scrapeResult = await scraper.fetch(
            provider: provider,
            importedSession: importedSession,
            force: forceRefresh,
            allowLiveNavigation: allowLiveNavigation
        )

        var detailLines = [String]()
        if let snapshot {
            detailLines.append("Live source: \(snapshot.sourceUsed)")
            if let credits = snapshot.credits {
                detailLines.append(String(format: "Credits: %.2f", credits))
            }
        }

        if let importedSession {
            detailLines.append("Imported from \(importedSession.browserSource.title) · \(importedSession.profileName).")
            detailLines.append("Stored auth cookies: \(importedSession.cookies.count) from \(importedSession.storageKind).")
            detailLines.append("Imported at \(relativeLabel(importedSession.importedAt)).")
        }
        detailLines.append(contentsOf: scrapeResult.detailLines)

        return DashboardAdjunctSnapshot(
            provider: provider,
            source: source,
            headline: scrapeResult.headline,
            detailLines: detailLines,
            webExtras: scrapeResult.webExtras,
            statusText: scrapeResult.statusText,
            isLoginRequired: scrapeResult.isLoginRequired,
            lastUpdated: scrapeResult.fetchedAt
        )
    }
    private func relativeLabel(_ timestamp: String) -> String {
        let formatter = ISO8601DateFormatter()
        guard let date = formatter.date(from: timestamp) else { return timestamp }
        let delta = max(0, Int(Date().timeIntervalSince(date)))
        if delta < 60 {
            return "\(delta)s ago"
        }
        if delta < 3600 {
            return "\(delta / 60)m ago"
        }
        if delta < 86_400 {
            return "\(delta / 3600)h ago"
        }
        return "\(delta / 86_400)d ago"
    }
}
