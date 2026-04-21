import Foundation
import HeimdallDomain
import HeimdallServices

public actor BrowserSessionController: BrowserSessionManaging {
    private let keychainStore: KeychainStore
    private let importer: BrowserSessionImporter

    public init(
        keychainStore: KeychainStore = KeychainStore(),
        importer: BrowserSessionImporter = BrowserSessionImporter()
    ) {
        self.keychainStore = keychainStore
        self.importer = importer
    }

    public func importedSession(provider: ProviderID) async -> ImportedBrowserSession? {
        self.keychainStore.loadJSON(ImportedBrowserSession.self, account: self.account(for: provider))
    }

    public func discoverImportCandidates(provider _: ProviderID) async -> [BrowserSessionImportCandidate] {
        self.importer.discoverCandidates()
    }

    public func importBrowserSession(
        provider: ProviderID,
        candidate: BrowserSessionImportCandidate
    ) async throws -> ImportedBrowserSession {
        let session = try self.importer.importSession(provider: provider, candidate: candidate)
        try self.keychainStore.saveJSON(session, account: self.account(for: provider))
        return session
    }

    public func resetImportedSession(provider: ProviderID) async throws {
        try self.keychainStore.delete(account: self.account(for: provider))
    }

    private func account(for provider: ProviderID) -> String {
        "\(provider.rawValue).web-session"
    }
}
