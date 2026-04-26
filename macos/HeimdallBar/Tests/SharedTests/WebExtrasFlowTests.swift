import Foundation
import Testing
@testable import HeimdallDomain

/// Phase 13 acceptance test: when a Codex web-imported session feeds
/// `DashboardAdjunctSnapshot.webExtras` into the data model, the credits and
/// lane data must surface through the three independent presentation
/// pipelines used by the menu bar, widget, and CLI.
///
/// Until this file landed, every existing test set `adjunct: nil`, so the
/// `webExtras` fallback chain in `ProviderPresentationState`,
/// `MenuProjectionBuilder`, and `WidgetSnapshotBuilder` was untested. A
/// silent regression in any of those merge points would reproduce the
/// pre-Phase-13 symptom: the scraper produces data, but no UI surface ever
/// displays it.
struct WebExtrasFlowTests {
    // MARK: ProviderPresentationState (shared accessor layer)

    /// `displayCredits` is read by both the menu (`creditsLabel`) and the CLI
    /// (`webExtrasPayload`). When `effectiveSource == .web` and the snapshot
    /// has no native credits, the accessor must return the web value.
    @Test
    func presentationDisplayCreditsPrefersWebExtrasOnWebSource() {
        let presentation = Self.webExtrasOnlyPresentation()
        #expect(presentation.displayCredits == 14.75)
        #expect(presentation.primary?.usedPercent == 36)
        #expect(presentation.secondary?.usedPercent == 59)
        #expect(presentation.tertiary == nil)
        // Identity falls through to webExtras.signedInEmail/accountPlan when
        // the live snapshot has no identity.
        #expect(presentation.displayIdentityLabel == "fixture@example.com · Pro")
    }

    // MARK: MenuProjectionBuilder

    @Test
    func menuProjectionExposesWebExtrasCreditsAndLanesUnderWebSource() {
        let presentation = Self.webExtrasOnlyPresentation()
        let projection = MenuProjectionBuilder.projection(
            from: presentation,
            config: Self.config(),
            isRefreshing: false,
            lastGlobalError: nil
        )

        // Web-source rendering uses the "Web credits:" prefix to disambiguate
        // from native OAuth credits — locks in the user-visible distinction.
        #expect(projection.creditsLabel == "Web credits: 14.75")

        // Lane details come from the webExtras quota lanes via
        // `presentation.primary` / `.secondary` fallback.
        #expect(projection.laneDetails.count == 2)
        #expect(projection.laneDetails.first?.summary.contains("64% left") == true)
        #expect(projection.laneDetails.dropFirst().first?.summary.contains("41% left") == true)

        // Identity surfaces via the menu's identityLabel.
        #expect(projection.identityLabel == "fixture@example.com · Pro")
    }

    // MARK: WidgetSnapshotBuilder

    @Test
    func widgetSnapshotExposesWebExtrasCreditsAndLanesUnderWebSource() {
        let presentation = Self.webExtrasOnlyPresentation()
        let widget = WidgetSnapshotBuilder.providerSnapshot(
            provider: presentation.provider,
            config: Self.config().codex,
            snapshot: presentation.snapshot,
            adjunct: presentation.adjunct
        )

        // The widget's lanes array is built from `presentation.primary` etc,
        // which fall back to webExtras lanes when the snapshot is absent.
        #expect(widget.lanes.count == 2)
        #expect(widget.credits == 14.75)

        // Adjunct passthrough flag the widget uses to render login-required
        // / "web extras present" indicators stays consistent.
        #expect(widget.adjunct?.hasWebExtras == true)
        #expect(widget.adjunct?.isLoginRequired == false)
    }

    // MARK: Fixtures

    /// Builds a presentation state where the live snapshot is absent and
    /// the only data source is the web-scraped adjunct, with a forced
    /// `effectiveSource = .web` resolution. Mirrors the runtime state when
    /// a user has opted into Codex web extras and OAuth/CLI are unavailable.
    private static func webExtrasOnlyPresentation() -> ProviderPresentationState {
        let webExtras = DashboardWebExtras(
            signedInEmail: "fixture@example.com",
            accountPlan: "Pro",
            creditsRemaining: 14.75,
            creditsPurchaseURL: "https://chatgpt.com/codex/cloud/credits",
            quotaLanes: [
                DashboardWebQuotaLane(
                    title: "5-hour",
                    window: ProviderRateWindow(
                        usedPercent: 36,
                        resetsAt: nil,
                        resetsInMinutes: 18,
                        windowMinutes: 300,
                        resetLabel: nil
                    )
                ),
                DashboardWebQuotaLane(
                    title: "Weekly",
                    window: ProviderRateWindow(
                        usedPercent: 59,
                        resetsAt: nil,
                        resetsInMinutes: 2 * 24 * 60,
                        windowMinutes: 7 * 24 * 60,
                        resetLabel: nil
                    )
                ),
            ],
            sourceURL: "https://chatgpt.com/codex/cloud/settings/analytics",
            fetchedAt: "2026-04-26T10:00:00Z"
        )
        let adjunct = DashboardAdjunctSnapshot(
            provider: .codex,
            source: .web,
            headline: "Codex web extras",
            detailLines: ["fixture@example.com", "Pro"],
            webExtras: webExtras,
            statusText: "ready",
            isLoginRequired: false,
            lastUpdated: "2026-04-26T10:00:00Z"
        )
        let resolution = ProviderSourceResolution(
            provider: .codex,
            requestedSource: .web,
            effectiveSource: .web,
            effectiveSourceDetail: "web",
            sourceLabel: "Source: web",
            explanation: "Web-imported session is the active live source.",
            warnings: [],
            fallbackChain: ["web"],
            usageAvailable: true,
            isUnsupported: false,
            requiresLogin: false,
            usesFallback: false
        )
        return ProviderPresentationState(
            provider: .codex,
            snapshot: nil,
            adjunct: adjunct,
            resolution: resolution
        )
    }

    private static func config() -> HeimdallBarConfig {
        HeimdallBarConfig(
            claude: ProviderConfig(
                enabled: true,
                source: .oauth,
                cookieSource: .auto,
                dashboardExtrasEnabled: false
            ),
            codex: ProviderConfig(
                enabled: true,
                source: .web,
                cookieSource: .web,
                dashboardExtrasEnabled: true
            ),
            mergeIcons: false,
            showUsedValues: false,
            refreshIntervalSeconds: 120,
            resetDisplayMode: .countdown,
            checkProviderStatus: true,
            localNotificationsEnabled: false,
            helperPort: 8787
        )
    }
}
