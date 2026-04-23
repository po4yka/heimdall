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
            #expect(request.timeoutInterval == 12)
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
            return Self.jsonResponse(
                url: try #require(request.url),
                body: """
                {
                  "contract_version": 1,
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
                        "recommended_key": "p90",
                        "levels": [
                          { "key": "p90", "label": "P90", "limit_tokens": 800000 },
                          { "key": "p95", "label": "P95", "limit_tokens": 900000 },
                          { "key": "max", "label": "Max", "limit_tokens": 950000 }
                        ],
                        "note": "Based on fewer than 10 completed blocks."
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
        #expect(envelope.providers.first?.depletionForecast?.primarySignal.kind == "billing_block")
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
          "contract_version": 1,
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

private final class StubURLProtocol: URLProtocol, @unchecked Sendable {
    nonisolated(unsafe) static var handler: (@Sendable (URLRequest, Int) throws -> (HTTPURLResponse, Data))?

    private static let lock = NSLock()
    nonisolated(unsafe) private static var requests: [URLRequest] = []

    static var requestCount: Int {
        self.lock.lock()
        defer { self.lock.unlock() }
        return self.requests.count
    }

    static func reset() {
        self.lock.lock()
        defer { self.lock.unlock() }
        self.handler = nil
        self.requests = []
    }

    override class func canInit(with request: URLRequest) -> Bool {
        true
    }

    override class func canonicalRequest(for request: URLRequest) -> URLRequest {
        request
    }

    override func startLoading() {
        Self.lock.lock()
        Self.requests.append(self.request)
        let attempt = Self.requests.count
        let handler = Self.handler
        Self.lock.unlock()

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
