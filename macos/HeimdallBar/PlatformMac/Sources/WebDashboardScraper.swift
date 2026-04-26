import Foundation
import HeimdallDomain
import WebKit

@MainActor
public final class WebDashboardScraper {
    public struct ScrapeResult: Sendable {
        public var statusText: String
        public var headline: String
        public var detailLines: [String]
        public var webExtras: DashboardWebExtras?
        public var isLoginRequired: Bool
        public var fetchedAt: String
    }

    private struct CachedScrapeEntry: Codable {
        var provider: ProviderID
        var sessionFingerprint: String
        var result: ScrapeResultPayload
        var savedAt: String
    }

    private struct ScrapeResultPayload: Codable {
        var statusText: String
        var headline: String
        var detailLines: [String]
        var webExtras: DashboardWebExtras?
        var isLoginRequired: Bool
        var fetchedAt: String
    }

    private final class NavigationDelegate: NSObject, WKNavigationDelegate {
        private var continuation: CheckedContinuation<Void, Error>?

        func wait() async throws {
            try await withCheckedThrowingContinuation { continuation in
                self.continuation = continuation
            }
        }

        func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
            self.continuation?.resume()
            self.continuation = nil
        }

        func webView(
            _ webView: WKWebView,
            didFail navigation: WKNavigation!,
            withError error: Error
        ) {
            self.continuation?.resume(throwing: error)
            self.continuation = nil
        }

        func webView(
            _ webView: WKWebView,
            didFailProvisionalNavigation navigation: WKNavigation!,
            withError error: Error
        ) {
            self.continuation?.resume(throwing: error)
            self.continuation = nil
        }
    }

    public init() {}

    public func warm() {
        _ = self.makeWebView()
    }

    public func statusMessage(provider: ProviderID, sessionStatus: String) -> String {
        guard sessionStatus == "ready" else {
            switch provider {
            case .claude:
                return sessionStatus == "expired" ? "Claude web dashboard session expired" : "Claude web dashboard login required"
            case .codex:
                return sessionStatus == "expired" ? "OpenAI web dashboard session expired" : "OpenAI web dashboard login required"
            }
        }

        switch provider {
        case .claude:
            return "Claude web dashboard session available"
        case .codex:
            return "OpenAI web dashboard session available"
        }
    }

    public func fetch(
        provider: ProviderID,
        importedSession: ImportedBrowserSession?,
        force: Bool,
        allowLiveNavigation: Bool = true
    ) async -> ScrapeResult {
        let nowStamp = ISO8601DateFormatter().string(from: Date())
        guard let importedSession else {
            return ScrapeResult(
                statusText: "missing",
                headline: self.statusMessage(provider: provider, sessionStatus: "missing"),
                detailLines: ["Import browser cookies to unlock web-only dashboard details."],
                webExtras: nil,
                isLoginRequired: true,
                fetchedAt: nowStamp
            )
        }

        if importedSession.expired {
            return ScrapeResult(
                statusText: "expired",
                headline: self.statusMessage(provider: provider, sessionStatus: "expired"),
                detailLines: ["Stored browser session appears expired and needs refresh."],
                webExtras: nil,
                isLoginRequired: true,
                fetchedAt: nowStamp
            )
        }

        if importedSession.loginRequired {
            return ScrapeResult(
                statusText: "login-required",
                headline: self.statusMessage(provider: provider, sessionStatus: "login-required"),
                detailLines: ["Stored browser session is missing an active auth cookie."],
                webExtras: nil,
                isLoginRequired: true,
                fetchedAt: nowStamp
            )
        }

        if provider == .claude {
            // Phase 13 decision: keep the Claude path as a documented stub.
            //
            // Roadmap line 47 ("Claude web fallback logic where parity needs
            // it") was scoped to fields the OAuth + admin-API + local-DB
            // pipeline cannot reach.  Candidate web-only fields surveyed
            // during Phase 13 planning:
            //
            //   • Subscription billing (next renewal, card last4) on
            //     claude.ai/settings/billing — narrowly useful, but not
            //     covered by the menu/widget/CLI surfaces today.  Adding it
            //     would require new presentation slots without an obvious
            //     home.
            //   • Pre-paid credit balance on console.anthropic.com — already
            //     reachable via the admin API for users who provide an
            //     ANTHROPIC_ADMIN_KEY.
            //   • Per-day usage history with caching breakdown — the local
            //     DB scanner already produces this; web scraping would
            //     duplicate, not augment.
            //   • Recent organization activity (team plans) — possibly
            //     web-only, but plan-conditional and currently out of
            //     scope for the menu-bar app.
            //
            // The accept-cost / reject-value calculus did not justify a
            // full WKWebView path for any single candidate.  When a concrete
            // user-visible field surfaces that OAuth + admin + local-DB
            // genuinely cannot deliver, replace this branch with a real
            // scrapeClaude(importedSession:) symmetric to scrapeCodex.
            return ScrapeResult(
                statusText: "ready",
                headline: "Claude web fallback is standing by",
                detailLines: ["Claude live data already comes from OAuth and local history.", "A Claude WebKit fetch is skipped until it exposes fields the live helper cannot provide."],
                webExtras: nil,
                isLoginRequired: false,
                fetchedAt: nowStamp
            )
        }

        let sessionFingerprint = self.sessionFingerprint(importedSession)
        if !force, let cached = Self.loadCached(provider: provider, sessionFingerprint: sessionFingerprint) {
            let result = Self.result(from: cached.result)
            return ScrapeResult(
                statusText: result.statusText,
                headline: result.headline,
                detailLines: result.detailLines + ["Using cached web dashboard scrape."],
                webExtras: result.webExtras,
                isLoginRequired: result.isLoginRequired,
                fetchedAt: result.fetchedAt
            )
        }

        if !allowLiveNavigation {
            return ScrapeResult(
                statusText: "ready",
                headline: "OpenAI web extras are waiting for an app refresh",
                detailLines: ["Live web scraping runs only inside HeimdallBar.app.", "Open the app or refresh the menu bar to populate cached web dashboard extras for CLI reads."],
                webExtras: nil,
                isLoginRequired: false,
                fetchedAt: nowStamp
            )
        }

        for attempt in 1...2 {
            do {
                let result = try await self.scrapeCodex(importedSession: importedSession)
                Self.saveCached(
                    provider: provider,
                    sessionFingerprint: sessionFingerprint,
                    result: ScrapeResultPayload(
                        statusText: result.statusText,
                        headline: result.headline,
                        detailLines: result.detailLines,
                        webExtras: result.webExtras,
                        isLoginRequired: result.isLoginRequired,
                        fetchedAt: result.fetchedAt
                    )
                )
                return result
            } catch {
                if attempt == 2 {
                    let message = error.localizedDescription
                    return ScrapeResult(
                        statusText: "error",
                        headline: "OpenAI web dashboard refresh failed",
                        detailLines: ["WebKit could not extract dashboard extras.", message],
                        webExtras: nil,
                        isLoginRequired: false,
                        fetchedAt: nowStamp
                    )
                }
            }
        }

        return ScrapeResult(
            statusText: "error",
            headline: "OpenAI web dashboard refresh failed",
            detailLines: ["WebKit could not extract dashboard extras."],
            webExtras: nil,
            isLoginRequired: false,
            fetchedAt: nowStamp
        )
    }

    private func scrapeCodex(importedSession: ImportedBrowserSession) async throws -> ScrapeResult {
        let url = URL(string: "https://chatgpt.com/codex/cloud/settings/analytics#usage")!
        let webView = self.makeWebView()
        try await self.installCookies(importedSession.cookies, into: webView.configuration.websiteDataStore.httpCookieStore)
        let delegate = NavigationDelegate()
        webView.navigationDelegate = delegate
        webView.load(URLRequest(url: url))
        try await self.withTimeout(seconds: 15) {
            try await delegate.wait()
        }

        for _ in 0..<8 {
            let payload = try await self.samplePage(webView: webView)
            if payload.loginRequired {
                return ScrapeResult(
                    statusText: "login-required",
                    headline: "OpenAI web dashboard login required",
                    detailLines: ["Imported cookies no longer satisfy the OpenAI dashboard login."],
                    webExtras: nil,
                    isLoginRequired: true,
                    fetchedAt: ISO8601DateFormatter().string(from: Date())
                )
            }

            let extras = self.parseCodexDashboardDocument(
                html: payload.html,
                bodyText: payload.bodyText,
                sourceURL: payload.url,
                purchaseURL: payload.purchaseURL
            )
            if extras.creditsRemaining != nil || !extras.quotaLanes.isEmpty || extras.signedInEmail != nil {
                var detailLines = [String]()
                if let email = extras.signedInEmail {
                    detailLines.append("Signed in as \(email).")
                }
                if let plan = extras.accountPlan {
                    detailLines.append("Plan: \(plan).")
                }
                if let credits = extras.creditsRemaining {
                    detailLines.append(String(format: "Web credits remaining: %.2f.", credits))
                }
                detailLines.append(contentsOf: extras.quotaLanes.map { lane in
                    self.laneSummary(title: lane.title, window: lane.window)
                })
                if let purchaseURL = extras.creditsPurchaseURL {
                    detailLines.append("Credits purchase link detected: \(purchaseURL).")
                }

                return ScrapeResult(
                    statusText: "ready",
                    headline: "OpenAI web dashboard extras refreshed",
                    detailLines: detailLines,
                    webExtras: extras,
                    isLoginRequired: false,
                    fetchedAt: extras.fetchedAt
                )
            }

            try? await Task.sleep(for: .milliseconds(750))
        }

        return ScrapeResult(
            statusText: "partial",
            headline: "OpenAI web dashboard loaded without extra fields",
            detailLines: ["Dashboard page loaded, but no credits or quota lanes were extracted yet."],
            webExtras: DashboardWebExtras(
                signedInEmail: nil,
                accountPlan: nil,
                creditsRemaining: nil,
                creditsPurchaseURL: nil,
                quotaLanes: [],
                sourceURL: url.absoluteString,
                fetchedAt: ISO8601DateFormatter().string(from: Date())
            ),
            isLoginRequired: false,
            fetchedAt: ISO8601DateFormatter().string(from: Date())
        )
    }

    private func makeWebView() -> WKWebView {
        let configuration = WKWebViewConfiguration()
        configuration.websiteDataStore = .nonPersistent()
        return WKWebView(frame: .zero, configuration: configuration)
    }

    private func installCookies(
        _ importedCookies: [ImportedSessionCookie],
        into store: WKHTTPCookieStore
    ) async throws {
        let cookies = importedCookies.compactMap(self.httpCookie(from:))
        for cookie in cookies {
            await withCheckedContinuation { continuation in
                store.setCookie(cookie) {
                    continuation.resume()
                }
            }
        }
    }

    private func httpCookie(from importedCookie: ImportedSessionCookie) -> HTTPCookie? {
        guard let value = importedCookie.value, !value.isEmpty else { return nil }
        var properties: [HTTPCookiePropertyKey: Any] = [
            .domain: importedCookie.domain,
            .path: importedCookie.path,
            .name: importedCookie.name,
            .value: value,
        ]
        if importedCookie.secure {
            properties[.secure] = true
        }
        if let expiresAt = importedCookie.expiresAt,
           let expiresDate = ISO8601DateFormatter().date(from: expiresAt)
        {
            properties[.expires] = expiresDate
        }
        return HTTPCookie(properties: properties)
    }

    private func samplePage(webView: WKWebView) async throws -> (bodyText: String, html: String, url: String?, purchaseURL: String?, loginRequired: Bool) {
        let script = """
        (() => {
          const textOf = value => typeof value === 'string' ? value : '';
          const bodyText = document.body ? String(document.body.innerText || document.body.textContent || '') : '';
          const html = document.documentElement ? document.documentElement.outerHTML : '';
          const purchaseLink = Array.from(document.querySelectorAll('a[href], button'))
            .map(node => {
              const text = String(node.innerText || node.textContent || '').toLowerCase();
              const href = node.getAttribute && node.getAttribute('href');
              if (text.includes('credit') && (text.includes('buy') || text.includes('add') || text.includes('purchase'))) {
                return href || null;
              }
              return null;
            })
            .find(Boolean);
          const href = purchaseLink
            ? (purchaseLink.startsWith('http') ? purchaseLink : `${location.origin}${purchaseLink.startsWith('/') ? '' : '/'}${purchaseLink}`)
            : null;
          const lower = bodyText.toLowerCase();
          const loginRequired =
            location.href.includes('/auth/login') ||
            lower.includes('sign in') ||
            lower.includes('log in') ||
            lower.includes('continue with google') ||
            lower.includes('continue with apple');
          return {
            bodyText,
            html,
            url: location.href,
            purchaseURL: href,
            loginRequired
          };
        })();
        """
        guard let payload = try await webView.evaluateJavaScript(script) as? [String: Any] else {
            throw URLError(.cannotParseResponse)
        }
        return (
            bodyText: payload["bodyText"] as? String ?? "",
            html: payload["html"] as? String ?? "",
            url: payload["url"] as? String,
            purchaseURL: payload["purchaseURL"] as? String,
            loginRequired: payload["loginRequired"] as? Bool ?? false
        )
    }

    func parseCodexDashboardDocument(
        html: String,
        bodyText: String,
        sourceURL: String?,
        purchaseURL: String?
    ) -> DashboardWebExtras {
        DashboardWebExtras(
            signedInEmail: self.parseSignedInEmail(from: html),
            accountPlan: self.parsePlan(from: html),
            creditsRemaining: self.parseCreditsRemaining(from: bodyText),
            creditsPurchaseURL: purchaseURL,
            quotaLanes: self.parseQuotaLanes(from: bodyText),
            sourceURL: sourceURL,
            fetchedAt: ISO8601DateFormatter().string(from: Date())
        )
    }

    private func parseSignedInEmail(from html: String) -> String? {
        let patterns = [
            #""email"\s*:\s*"([^"]+@[^"]+)""#,
            #""account_email"\s*:\s*"([^"]+@[^"]+)""#,
        ]
        return patterns.compactMap { pattern in
            self.firstMatch(in: html, pattern: pattern)
        }.first
    }

    private func parsePlan(from html: String) -> String? {
        let patterns = [
            #""plan"\s*:\s*"([^"]+)""#,
            #""accountPlan"\s*:\s*"([^"]+)""#,
        ]
        return patterns.compactMap { pattern in
            self.firstMatch(in: html, pattern: pattern)
        }.first
    }

    private func parseCreditsRemaining(from bodyText: String) -> Double? {
        let patterns = [
            #"credits\s*remaining[^0-9]*([0-9][0-9.,]*)"#,
            #"credit\s*balance[^0-9]*([0-9][0-9.,]*)"#,
            #"remaining\s*credits[^0-9]*([0-9][0-9.,]*)"#,
        ]
        for pattern in patterns {
            if let capture = self.firstMatch(in: bodyText, pattern: pattern),
               let value = Double(capture.replacingOccurrences(of: ",", with: ""))
            {
                return value
            }
        }
        return nil
    }

    private func parseQuotaLanes(from bodyText: String) -> [DashboardWebQuotaLane] {
        let lines = bodyText
            .replacingOccurrences(of: "\r", with: "\n")
            .split(whereSeparator: \.isNewline)
            .map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
            .filter { !$0.isEmpty }

        let descriptors: [(String, [String], Int?)] = [
            ("Session", ["5-hour", "5 hour", "session"], 5 * 60),
            ("Weekly", ["weekly"], 7 * 24 * 60),
            ("Code Review", ["code review", "core review"], nil),
        ]

        return descriptors.compactMap { descriptor in
            self.parseLane(title: descriptor.0, keywords: descriptor.1, defaultWindowMinutes: descriptor.2, lines: lines)
        }
    }

    private func parseLane(
        title: String,
        keywords: [String],
        defaultWindowMinutes: Int?,
        lines: [String]
    ) -> DashboardWebQuotaLane? {
        for index in lines.indices {
            let lower = lines[index].lowercased()
            guard keywords.contains(where: { lower.contains($0) }) else { continue }
            let endIndex = min(lines.count - 1, index + 5)
            let window = Array(lines[index...endIndex]).joined(separator: " ")
            guard let percent = self.firstPercent(in: window) else { continue }
            let remaining = {
                let lowerWindow = window.lowercased()
                return lowerWindow.contains("remaining") || lowerWindow.contains("% left")
            }()
            let usedPercent = remaining ? max(0, 100 - percent) : percent
            let resetMinutes = self.resetMinutes(in: window)
            let resetsAt = resetMinutes.map { minutes in
                ISO8601DateFormatter().string(from: Date().addingTimeInterval(TimeInterval(minutes * 60)))
            }
            return DashboardWebQuotaLane(
                title: title,
                window: ProviderRateWindow(
                    usedPercent: usedPercent,
                    resetsAt: resetsAt,
                    resetsInMinutes: resetMinutes,
                    windowMinutes: defaultWindowMinutes,
                    resetLabel: resetMinutes.map { "resets in \($0)m" }
                )
            )
        }
        return nil
    }

    private func firstPercent(in text: String) -> Double? {
        guard let capture = self.firstMatch(in: text, pattern: #"([0-9]{1,3})\s*%"#) else { return nil }
        return Double(capture)
    }

    private func resetMinutes(in text: String) -> Int? {
        let lower = text.lowercased()
        if let capture = self.firstMatch(in: lower, pattern: #"([0-9]{1,4})\s*(minute|minutes|min|hour|hours|day|days|week|weeks)"#),
           let value = Int(capture)
        {
            let unit = self.secondMatch(in: lower, pattern: #"([0-9]{1,4})\s*(minute|minutes|min|hour|hours|day|days|week|weeks)"#) ?? "minutes"
            switch unit {
            case "minute", "minutes", "min":
                return value
            case "hour", "hours":
                return value * 60
            case "day", "days":
                return value * 24 * 60
            case "week", "weeks":
                return value * 7 * 24 * 60
            default:
                return value
            }
        }
        return nil
    }

    private func firstMatch(in text: String, pattern: String) -> String? {
        guard let regex = try? NSRegularExpression(pattern: pattern, options: [.caseInsensitive]) else { return nil }
        let range = NSRange(text.startIndex..<text.endIndex, in: text)
        guard let match = regex.firstMatch(in: text, options: [], range: range),
              match.numberOfRanges > 1,
              let captureRange = Range(match.range(at: 1), in: text)
        else {
            return nil
        }
        return String(text[captureRange]).trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private func secondMatch(in text: String, pattern: String) -> String? {
        guard let regex = try? NSRegularExpression(pattern: pattern, options: [.caseInsensitive]) else { return nil }
        let range = NSRange(text.startIndex..<text.endIndex, in: text)
        guard let match = regex.firstMatch(in: text, options: [], range: range),
              match.numberOfRanges > 2,
              let captureRange = Range(match.range(at: 2), in: text)
        else {
            return nil
        }
        return String(text[captureRange]).trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private func laneSummary(title: String, window: ProviderRateWindow) -> String {
        let left = max(0, Int((100 - window.usedPercent).rounded()))
        if let resetsInMinutes = window.resetsInMinutes {
            return "\(title): \(left)% left · resets in \(resetsInMinutes)m"
        }
        return "\(title): \(left)% left"
    }

    private func sessionFingerprint(_ session: ImportedBrowserSession) -> String {
        [
            session.provider.rawValue,
            session.browserSource.rawValue,
            session.profileName,
            session.importedAt,
            "\(session.cookies.count)",
        ].joined(separator: "|")
    }

    private func withTimeout<T: Sendable>(
        seconds: TimeInterval,
        operation: @escaping @Sendable () async throws -> T
    ) async throws -> T {
        try await withThrowingTaskGroup(of: T.self) { group in
            group.addTask {
                try await operation()
            }
            group.addTask {
                try await Task.sleep(for: .seconds(seconds))
                throw URLError(.timedOut)
            }
            let value = try await group.next()!
            group.cancelAll()
            return value
        }
    }

    private static func result(from payload: ScrapeResultPayload) -> ScrapeResult {
        ScrapeResult(
            statusText: payload.statusText,
            headline: payload.headline,
            detailLines: payload.detailLines,
            webExtras: payload.webExtras,
            isLoginRequired: payload.isLoginRequired,
            fetchedAt: payload.fetchedAt
        )
    }

    private static func loadCached(
        provider: ProviderID,
        sessionFingerprint: String
    ) -> CachedScrapeEntry? {
        guard let url = self.cacheURL(for: provider),
              let data = try? Data(contentsOf: url),
              let entry = try? JSONDecoder().decode(CachedScrapeEntry.self, from: data),
              entry.sessionFingerprint == sessionFingerprint
        else {
            return nil
        }
        guard let savedAt = ISO8601DateFormatter().date(from: entry.savedAt),
              Date().timeIntervalSince(savedAt) < 600
        else {
            return nil
        }
        return entry
    }

    private static func saveCached(
        provider: ProviderID,
        sessionFingerprint: String,
        result: ScrapeResultPayload
    ) {
        guard let url = self.cacheURL(for: provider) else { return }
        let entry = CachedScrapeEntry(
            provider: provider,
            sessionFingerprint: sessionFingerprint,
            result: self.redacted(result),
            savedAt: ISO8601DateFormatter().string(from: Date())
        )
        do {
            try FileManager.default.createDirectory(
                at: url.deletingLastPathComponent(),
                withIntermediateDirectories: true
            )
            let data = try JSONEncoder().encode(entry)
            try data.write(to: url, options: [.atomic])
        } catch {
            // Best-effort cache only.
        }
    }

    private static func redacted(_ result: ScrapeResultPayload) -> ScrapeResultPayload {
        let redactedLines = result.detailLines.filter { !$0.lowercased().hasPrefix("signed in as ") }
        let redactedExtras = result.webExtras.map { extras in
            DashboardWebExtras(
                signedInEmail: nil,
                accountPlan: extras.accountPlan,
                creditsRemaining: extras.creditsRemaining,
                creditsPurchaseURL: extras.creditsPurchaseURL,
                quotaLanes: extras.quotaLanes,
                sourceURL: extras.sourceURL,
                fetchedAt: extras.fetchedAt
            )
        }
        return ScrapeResultPayload(
            statusText: result.statusText,
            headline: result.headline,
            detailLines: redactedLines,
            webExtras: redactedExtras,
            isLoginRequired: result.isLoginRequired,
            fetchedAt: result.fetchedAt
        )
    }

    private static func cacheURL(for provider: ProviderID) -> URL? {
        guard let root = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first else {
            return nil
        }
        return root
            .appendingPathComponent("dev.heimdall.HeimdallBar", isDirectory: true)
            .appendingPathComponent("web-dashboard-\(provider.rawValue).json", isDirectory: false)
    }
}
