import Foundation
import WebKit

@MainActor
public final class WebDashboardScraper {
    private lazy var webView: WKWebView = {
        let configuration = WKWebViewConfiguration()
        configuration.websiteDataStore = .nonPersistent()
        return WKWebView(frame: .zero, configuration: configuration)
    }()

    public init() {}

    public func warm() {
        _ = self.webView
    }

    public func statusMessage(provider: ProviderID, hasStoredSession: Bool) -> String {
        guard hasStoredSession else {
            switch provider {
            case .claude:
                return "Claude web dashboard login required"
            case .codex:
                return "OpenAI web dashboard login required"
            }
        }

        switch provider {
        case .claude:
            return "Claude web dashboard session available"
        case .codex:
            return "OpenAI web dashboard session available"
        }
    }
}
