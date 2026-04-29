import Foundation
import Testing
import HeimdallDomain
@testable import HeimdallPlatformMac

struct HeimdallAPIClientContractVersionTests {
    // MARK: - ProviderSnapshotEnvelope contract version validation

    @Test
    func contractVersionAboveCompiledConstantThrows() async {
        StubURLProtocol.reset()
        StubURLProtocol.handler = { request, _ in
            Self.jsonResponse(
                url: try #require(request.url),
                body: Self.snapshotEnvelopeJSON(contractVersion: 999)
            )
        }

        let client = Self.makeClient()
        await #expect(throws: ContractVersionError.self) {
            _ = try await client.fetchSnapshots()
        }
    }

    @Test
    func contractVersionAboveCompiledConstantErrorMessageIncludesBothVersions() async {
        StubURLProtocol.reset()
        StubURLProtocol.handler = { request, _ in
            Self.jsonResponse(
                url: try #require(request.url),
                body: Self.snapshotEnvelopeJSON(contractVersion: 999)
            )
        }

        let client = Self.makeClient()
        do {
            _ = try await client.fetchSnapshots()
            Issue.record("Expected ContractVersionError to be thrown")
        } catch let error as ContractVersionError {
            let message = error.description
            #expect(message.contains("999"))
            #expect(message.contains("\(HeimdallAPIClient.liveProvidersContractVersion)"))
        } catch {
            Issue.record("Unexpected error type: \(error)")
        }
    }

    @Test
    func contractVersionMatchingCompiledConstantSucceeds() async throws {
        StubURLProtocol.reset()
        StubURLProtocol.handler = { request, _ in
            Self.jsonResponse(
                url: try #require(request.url),
                body: Self.snapshotEnvelopeJSON(contractVersion: HeimdallAPIClient.liveProvidersContractVersion)
            )
        }

        let client = Self.makeClient()
        let envelope = try await client.fetchSnapshots()
        #expect(envelope.contractVersion == HeimdallAPIClient.liveProvidersContractVersion)
    }

    @Test
    func contractVersionBelowCompiledConstantSucceeds() async throws {
        // Older helpers speak a lower version — accept silently.
        // Guard: only run if the compiled constant is > 1 so we can form a valid lower value.
        guard HeimdallAPIClient.liveProvidersContractVersion > 1 else { return }

        StubURLProtocol.reset()
        let olderVersion = HeimdallAPIClient.liveProvidersContractVersion - 1
        StubURLProtocol.handler = { request, _ in
            Self.jsonResponse(
                url: try #require(request.url),
                body: Self.snapshotEnvelopeJSON(contractVersion: olderVersion)
            )
        }

        let client = Self.makeClient()
        let envelope = try await client.fetchSnapshots()
        #expect(envelope.contractVersion == olderVersion)
    }

    // MARK: - Helpers

    private static func makeClient() -> HeimdallAPIClient {
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [StubURLProtocol.self]
        let session = URLSession(configuration: configuration)
        return HeimdallAPIClient(
            baseURL: URL(string: "http://127.0.0.1:8787")!,
            session: session,
            retryPolicy: .init(attempts: 1, backoffNanoseconds: [])
        )
    }

    private static func jsonResponse(url: URL, body: String) -> (HTTPURLResponse, Data) {
        (
            HTTPURLResponse(url: url, statusCode: 200, httpVersion: nil, headerFields: nil)!,
            Data(body.utf8)
        )
    }

    private static func snapshotEnvelopeJSON(contractVersion: Int) -> String {
        """
        {
          "contract_version": \(contractVersion),
          "providers": [],
          "fetched_at": "2026-04-29T00:00:00Z",
          "requested_provider": null,
          "response_scope": "all",
          "cache_hit": false,
          "refreshed_providers": []
        }
        """
    }
}
