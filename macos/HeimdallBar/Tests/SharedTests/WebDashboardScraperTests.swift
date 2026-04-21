import Foundation
import Testing
import HeimdallDomain
@testable import HeimdallPlatformMac

@MainActor
struct WebDashboardScraperTests {
    @Test
    func missingSessionRequiresLogin() async {
        let scraper = WebDashboardScraper()

        let result = await scraper.fetch(
            provider: .codex,
            importedSession: nil,
            force: true,
            allowLiveNavigation: false
        )

        #expect(result.isLoginRequired)
        #expect(result.statusText == "missing")
        #expect(result.headline == "OpenAI web dashboard login required")
    }

    @Test
    func expiredSessionRequiresLogin() async {
        let scraper = WebDashboardScraper()

        let result = await scraper.fetch(
            provider: .codex,
            importedSession: Self.session(expired: true, loginRequired: false),
            force: true,
            allowLiveNavigation: false
        )

        #expect(result.isLoginRequired)
        #expect(result.statusText == "expired")
        #expect(result.detailLines.contains("Stored browser session appears expired and needs refresh."))
    }

    @Test
    func claudeSessionUsesStandingByFallback() async {
        let scraper = WebDashboardScraper()

        let result = await scraper.fetch(
            provider: .claude,
            importedSession: Self.session(expired: false, loginRequired: false),
            force: true,
            allowLiveNavigation: false
        )

        #expect(!result.isLoginRequired)
        #expect(result.statusText == "ready")
        #expect(result.headline == "Claude web fallback is standing by")
    }

    @Test
    func codexSessionWithoutAppRefreshStaysCacheOnly() async {
        let scraper = WebDashboardScraper()

        let result = await scraper.fetch(
            provider: .codex,
            importedSession: Self.session(expired: false, loginRequired: false),
            force: true,
            allowLiveNavigation: false
        )

        #expect(!result.isLoginRequired)
        #expect(result.headline == "OpenAI web extras are waiting for an app refresh")
        #expect(result.detailLines.contains("Live web scraping runs only inside HeimdallBar.app."))
    }

    @Test
    func codexDashboardFixtureExtractsCreditsIdentityAndQuotaLanes() throws {
        let scraper = WebDashboardScraper()
        let html = try FixtureLoader.string("tests/fixtures/codex/dashboard_usage.html")

        let extras = scraper.parseCodexDashboardDocument(
            html: html,
            bodyText: html,
            sourceURL: "https://chatgpt.com/codex/cloud/settings/analytics#usage",
            purchaseURL: "https://chatgpt.com/settings/billing/credits"
        )

        #expect(extras.signedInEmail == "fixture@example.com")
        #expect(extras.accountPlan == "Pro")
        #expect(extras.creditsRemaining == 14.75)
        #expect(extras.creditsPurchaseURL == "https://chatgpt.com/settings/billing/credits")
        #expect(extras.quotaLanes.count == 2)
        #expect(extras.primaryLane?.usedPercent.rounded() == 36)
        #expect(extras.secondaryLane?.usedPercent.rounded() == 59)
    }

    @Test
    func importedSessionFlagsStillDriveLoginRequiredState() async throws {
        let scraper = WebDashboardScraper()
        _ = try FixtureLoader.string("tests/fixtures/codex/dashboard_login_required.html")

        let result = await scraper.fetch(
            provider: .codex,
            importedSession: Self.session(expired: false, loginRequired: true),
            force: true,
            allowLiveNavigation: false
        )

        #expect(result.isLoginRequired)
        #expect(result.statusText == "login-required")
        #expect(result.headline == "OpenAI web dashboard login required")
    }

    private static func session(expired: Bool, loginRequired: Bool) -> ImportedBrowserSession {
        ImportedBrowserSession(
            provider: .codex,
            browserSource: .chrome,
            profileName: "Default",
            importedAt: ISO8601DateFormatter().string(from: Date()),
            storageKind: "chromium-sqlite",
            cookies: [
                ImportedSessionCookie(
                    domain: ".chatgpt.com",
                    name: "__Secure-next-auth.session-token",
                    value: "token",
                    path: "/",
                    expiresAt: nil,
                    secure: true,
                    httpOnly: true
                )
            ],
            loginRequired: loginRequired,
            expired: expired,
            lastValidatedAt: nil
        )
    }
}
