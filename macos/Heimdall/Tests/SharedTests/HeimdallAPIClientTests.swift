import Foundation
import Testing
import HeimdallDomain
@testable import HeimdallPlatformMac

struct HeimdallAPIClientTests {
    @Test
    func fetchStartupSnapshotsUsesStartupQuery() async throws {
        StubURLProtocol.reset()
        StubURLProtocol.handler = { request, _ in
            #expect(request.httpMethod == "GET")
            #expect(request.url?.path == "/api/live-providers")
            #expect(request.url?.query == "startup=true")
            return Self.jsonResponse(
                url: try #require(request.url),
                body: Self.snapshotEnvelopeJSON(provider: "claude")
            )
        }

        let client = Self.makeClient()
        let envelope = try await client.fetchStartupSnapshots()

        #expect(envelope.providers.count == 1)
        #expect(envelope.providers.first?.provider == "claude")
        #expect(StubURLProtocol.requestCount == 1)
    }

    @Test
    func fetchSnapshotsRetriesTransientConnectionRefused() async throws {
        StubURLProtocol.reset()
        StubURLProtocol.handler = { request, attempt in
            #expect(request.httpMethod == "GET")
            if attempt == 1 {
                throw URLError(.cannotConnectToHost)
            }
            return Self.jsonResponse(
                url: try #require(request.url),
                body: Self.snapshotEnvelopeJSON(provider: "claude")
            )
        }

        let client = Self.makeClient()
        let envelope = try await client.fetchSnapshots()

        #expect(envelope.providers.count == 1)
        #expect(envelope.providers.first?.provider == "claude")
        #expect(envelope.providers.first?.depletionForecast?.primarySignal.kind == "primary_window")
        #expect(envelope.providers.first?.quotaSuggestions?.populationCount == 9)
        #expect(envelope.providers.first?.predictiveInsights?.rollingHourBurn?.tier == "moderate")
        #expect(envelope.localNotificationState?.conditions.first?.id == "claude-session-depleted")
        #expect(StubURLProtocol.requestCount == 2)
    }

    @Test
    func fetchStartupSnapshotsUsesStartupQueryAndShorterTimeout() async throws {
        StubURLProtocol.reset()
        StubURLProtocol.handler = { request, attempt in
            #expect(request.httpMethod == "GET")
            #expect(request.url?.path == "/api/live-providers")
            #expect(request.url?.query == "startup=true")
            #expect(request.timeoutInterval == 1.5)
            if attempt == 1 {
                throw URLError(.cannotConnectToHost)
            }
            return Self.jsonResponse(
                url: try #require(request.url),
                body: Self.snapshotEnvelopeJSON(provider: "claude")
            )
        }

        let client = Self.makeClient()
        let envelope = try await client.fetchStartupSnapshots()

        #expect(envelope.providers.count == 1)
        #expect(envelope.providers.first?.provider == "claude")
        #expect(StubURLProtocol.requestCount == 2)
    }

    @Test
    func refreshRetriesTimeoutAndPreservesPOSTRequest() async throws {
        StubURLProtocol.reset()
        StubURLProtocol.handler = { request, attempt in
            #expect(request.httpMethod == "POST")
            #expect(request.url?.query?.contains("provider=codex") == true)
            #expect(request.timeoutInterval == 45)
            if attempt == 1 {
                throw URLError(.timedOut)
            }
            return Self.jsonResponse(
                url: try #require(request.url),
                body: Self.snapshotEnvelopeJSON(provider: "codex")
            )
        }

        let client = Self.makeClient()
        let envelope = try await client.refresh(provider: .codex)

        #expect(envelope.providers.count == 1)
        #expect(envelope.providers.first?.provider == "codex")
        #expect(StubURLProtocol.requestCount == 2)
    }

    @Test
    func fetchSnapshotsDoesNotRetryNonRetryableTransportErrors() async {
        StubURLProtocol.reset()
        StubURLProtocol.handler = { _, _ in
            throw URLError(.userAuthenticationRequired)
        }

        let client = Self.makeClient()

        await #expect(throws: URLError.self) {
            _ = try await client.fetchSnapshots()
        }
        #expect(StubURLProtocol.requestCount == 1)
    }

    @Test
    func fetchLiveMonitorDecodesSharedContract() async throws {
        StubURLProtocol.reset()
        StubURLProtocol.handler = { request, _ in
            #expect(request.url?.path == "/api/live-monitor")
            #expect(request.url?.query?.contains("tz_offset_min=") == true)
            #expect(request.timeoutInterval == 30)
            return Self.jsonResponse(
                url: try #require(request.url),
                body: """
                {
                  "contract_version": 2,
                  "generated_at": "2026-04-22T10:00:00Z",
                  "default_focus": "all",
                  "global_issue": "1 provider needs attention",
                  "freshness": {
                    "newest_provider_refresh": "2026-04-22T10:00:00Z",
                    "oldest_provider_refresh": "2026-04-22T09:59:00Z",
                    "stale_providers": [],
                    "has_stale_providers": false,
                    "refresh_state": "current"
                  },
                  "providers": [
                    {
                      "provider": "claude",
                      "title": "Claude",
                      "visual_state": "healthy",
                      "source_label": "Source: oauth",
                      "warnings": [],
                      "identity_label": "pro",
                      "primary": {
                        "used_percent": 25,
                        "resets_at": null,
                        "resets_in_minutes": 10,
                        "window_minutes": 300,
                        "reset_label": "resets in 10m"
                      },
                      "secondary": null,
                      "today_cost_usd": 3.25,
                      "projected_weekly_spend_usd": 22.75,
                      "last_refresh": "2026-04-22T10:00:00Z",
                      "last_refresh_label": "Updated just now",
                      "active_block": null,
                      "context_window": null,
                      "recent_session": null,
                      "depletion_forecast": {
                        "primary_signal": {
                          "kind": "billing_block",
                          "title": "Billing block",
                          "used_percent": 58,
                          "projected_percent": 92,
                          "remaining_tokens": 420000,
                          "remaining_percent": 42,
                          "end_time": "2026-04-22T12:00:00Z"
                        },
                        "secondary_signals": [
                          {
                            "kind": "primary_window",
                            "title": "Primary window",
                            "used_percent": 25,
                            "remaining_percent": 75,
                            "resets_in_minutes": 10,
                            "pace_label": "Comfortable",
                            "end_time": "2026-04-22T10:10:00Z"
                          }
                        ],
                        "summary_label": "Billing block projected to reach 92% before reset",
                        "severity": "danger"
                      },
                      "quota_suggestions": {
                        "sample_count": 4,
                        "population_count": 9,
                        "sample_strategy": "trailing_blocks",
                        "sample_label": "4 of 9 completed blocks",
                        "recommended_key": "p90",
                        "levels": [
                          { "key": "p90", "label": "P90", "limit_tokens": 800000 },
                          { "key": "p95", "label": "P95", "limit_tokens": 900000 },
                          { "key": "max", "label": "Max", "limit_tokens": 950000 }
                        ],
                        "note": "Based on fewer than 10 completed blocks."
                      },
                      "predictive_insights": {
                        "rolling_hour_burn": {
                          "tokens_per_min": 12450,
                          "cost_per_hour_nanos": 1450000000,
                          "coverage_minutes": 42,
                          "tier": "moderate"
                        },
                        "historical_envelope": {
                          "sample_count": 9,
                          "tokens": {
                            "average": 186000,
                            "p50": 171000,
                            "p75": 209000,
                            "p90": 248000,
                            "p95": 264000
                          },
                          "cost_usd": {
                            "average": 1.82,
                            "p50": 1.53,
                            "p75": 1.94,
                            "p90": 2.41,
                            "p95": 2.66
                          },
                          "turns": {
                            "average": 23,
                            "p50": 21,
                            "p75": 28,
                            "p90": 34,
                            "p95": 38
                          }
                        },
                        "limit_hit_analysis": {
                          "sample_count": 9,
                          "hit_count": 2,
                          "hit_rate": 0.22,
                          "threshold_tokens": 900000,
                          "threshold_percent": 90,
                          "active_current_hit": false,
                          "active_projected_hit": true,
                          "risk_level": "medium",
                          "summary_label": "Projected to hit the suggested quota in 2 of the last 9 windows."
                        }
                      }
                    }
                  ]
                }
                """
            )
        }

        let client = Self.makeClient()
        let envelope = try await client.fetchLiveMonitor()

        #expect(envelope.contractVersion == LiveMonitorContract.version)
        #expect(envelope.defaultFocus == .all)
        #expect(envelope.providers.first?.providerID == .claude)
        #expect(envelope.providers.first?.quotaSuggestions?.recommendedKey == "p90")
        #expect(envelope.providers.first?.quotaSuggestions?.sampleLabel == "4 of 9 completed blocks")
        #expect(envelope.providers.first?.depletionForecast?.primarySignal.kind == "billing_block")
        #expect(envelope.providers.first?.predictiveInsights?.historicalEnvelope?.sampleCount == 9)
        #expect(envelope.providers.first?.predictiveInsights?.limitHitAnalysis?.activeProjectedHit == true)
    }

    private static func makeClient() -> HeimdallAPIClient {
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [StubURLProtocol.self]
        let session = URLSession(configuration: configuration)
        return HeimdallAPIClient(
            baseURL: URL(string: "http://127.0.0.1:8787")!,
            session: session,
            retryPolicy: .init(attempts: 3, backoffNanoseconds: [1_000, 1_000, 1_000])
        )
    }

    private static func jsonResponse(url: URL, body: String) -> (HTTPURLResponse, Data) {
        (
            HTTPURLResponse(url: url, statusCode: 200, httpVersion: nil, headerFields: nil)!,
            Data(body.utf8)
        )
    }

    private static func snapshotEnvelopeJSON(provider: String) -> String {
        """
        {
          "contract_version": 2,
          "providers": [
            {
              "provider": "\(provider)",
              "available": true,
              "source_used": "oauth",
              "last_attempted_source": "oauth",
              "resolved_via_fallback": false,
              "refresh_duration_ms": 42,
              "source_attempts": [],
              "identity": null,
              "primary": {
                "used_percent": 25,
                "resets_at": null,
                "resets_in_minutes": 10,
                "window_minutes": 300,
                "reset_label": "resets in 10m"
              },
              "secondary": null,
              "tertiary": null,
              "credits": null,
              "status": null,
              "auth": {
                "login_method": "oauth",
                "credential_backend": "keychain",
                "auth_mode": "subscription",
                "is_authenticated": true,
                "is_refreshable": true,
                "is_source_compatible": true,
                "requires_relogin": false,
                "managed_restriction": null,
                "diagnostic_code": "authenticated-compatible",
                "failure_reason": null,
                "last_validated_at": "2026-04-20T15:30:00Z",
                "recovery_actions": []
              },
              "cost_summary": {
                "today_tokens": 1200,
                "today_cost_usd": 3.25,
                "last_30_days_tokens": 6400,
                "last_30_days_cost_usd": 21.75,
                "daily": []
              },
              "claude_usage": null,
              "quota_suggestions": {
                "sample_count": 4,
                "population_count": 9,
                "sample_strategy": "trailing_blocks",
                "sample_label": "4 of 9 completed blocks",
                "recommended_key": "p90",
                "levels": [
                  { "key": "p90", "label": "P90", "limit_tokens": 800000 },
                  { "key": "p95", "label": "P95", "limit_tokens": 900000 }
                ],
                "note": "Based on fewer than 10 completed blocks."
              },
              "depletion_forecast": {
                "primary_signal": {
                  "kind": "primary_window",
                  "title": "Primary window",
                  "used_percent": 25,
                  "remaining_percent": 75,
                  "resets_in_minutes": 10,
                  "pace_label": "Comfortable",
                  "end_time": "2026-04-20T15:40:00Z"
                },
                "secondary_signals": [],
                "summary_label": "Primary window currently at 25% used",
                "severity": "ok"
              },
              "predictive_insights": {
                "rolling_hour_burn": {
                  "tokens_per_min": 12450,
                  "cost_per_hour_nanos": 1450000000,
                  "coverage_minutes": 42,
                  "tier": "moderate"
                }
              },
              "last_refresh": "2026-04-20T15:30:00Z",
              "stale": false,
              "error": null
            }
          ],
          "fetched_at": "2026-04-20T15:30:00Z",
          "requested_provider": null,
          "response_scope": "all",
          "cache_hit": false,
          "refreshed_providers": ["\(provider)"],
          "local_notification_state": {
            "generated_at": "2026-04-20T15:30:00Z",
            "cost_threshold_usd": 25,
            "conditions": [
              {
                "id": "claude-session-depleted",
                "kind": "session_depleted",
                "provider": "claude",
                "service_label": "Claude",
                "is_active": false,
                "activation_title": "Claude session depleted",
                "activation_body": "Claude session is depleted.",
                "recovery_title": "Claude session restored",
                "recovery_body": "Claude session capacity is available again."
              }
            ]
          }
        }
        """
    }
}

final class StubURLProtocol: URLProtocol, @unchecked Sendable {
    typealias Handler = @Sendable (URLRequest, Int) throws -> (HTTPURLResponse, Data)

    /// Lock-guarded container for `handler` and `requests`. Encapsulating the
    /// mutable state behind an `let`-bound object lets the public API stay
    /// `static var handler` / `static var requestCount` while removing every
    /// `nonisolated(unsafe) static var` from the test surface — Swift no
    /// longer needs an escape hatch because there is no exposed mutable
    /// static.
    private final class State: @unchecked Sendable {
        private let lock = NSLock()
        private var handler: Handler?
        private var requests: [URLRequest] = []

        func getHandler() -> Handler? {
            self.lock.lock()
            defer { self.lock.unlock() }
            return self.handler
        }

        func setHandler(_ value: Handler?) {
            self.lock.lock()
            defer { self.lock.unlock() }
            self.handler = value
        }

        func requestCount() -> Int {
            self.lock.lock()
            defer { self.lock.unlock() }
            return self.requests.count
        }

        func reset() {
            self.lock.lock()
            defer { self.lock.unlock() }
            self.handler = nil
            self.requests = []
        }

        /// Atomically append `request`, then return the new attempt count and
        /// the current handler so `startLoading` reads both under one lock
        /// acquisition.
        func recordAndFetch(_ request: URLRequest) -> (Int, Handler?) {
            self.lock.lock()
            defer { self.lock.unlock() }
            self.requests.append(request)
            return (self.requests.count, self.handler)
        }
    }

    private static let state = State()

    static var handler: Handler? {
        get { self.state.getHandler() }
        set { self.state.setHandler(newValue) }
    }

    static var requestCount: Int { self.state.requestCount() }

    static func reset() { self.state.reset() }

    override class func canInit(with request: URLRequest) -> Bool {
        true
    }

    override class func canonicalRequest(for request: URLRequest) -> URLRequest {
        request
    }

    override func startLoading() {
        let (attempt, handler) = Self.state.recordAndFetch(self.request)

        guard let handler else {
            self.client?.urlProtocol(self, didFailWithError: URLError(.badURL))
            return
        }

        do {
            let (response, data) = try handler(self.request, attempt)
            self.client?.urlProtocol(self, didReceive: response, cacheStoragePolicy: .notAllowed)
            self.client?.urlProtocol(self, didLoad: data)
            self.client?.urlProtocolDidFinishLoading(self)
        } catch {
            self.client?.urlProtocol(self, didFailWithError: error)
        }
    }

    override func stopLoading() {}
}
