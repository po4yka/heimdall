import Foundation
import HeimdallDomain

struct PersistedMobileDashboardPreferences: Sendable, Equatable, Codable {
    var selectedProvider: ProviderID
    var selectedAccountScope: MobileAccountScope
    var compressionPreference: MobileCompressionPreference
    var aliases: [MobileAccountAlias]
    var collapsedSectionIDs: [String]

    init(
        selectedProvider: ProviderID = .claude,
        selectedAccountScope: MobileAccountScope = .all,
        compressionPreference: MobileCompressionPreference = .compact,
        aliases: [MobileAccountAlias] = [],
        collapsedSectionIDs: [String] = []
    ) {
        self.selectedProvider = selectedProvider
        self.selectedAccountScope = selectedAccountScope
        self.compressionPreference = compressionPreference
        self.aliases = aliases
        self.collapsedSectionIDs = collapsedSectionIDs
    }
}

protocol MobileDashboardPreferencesPersisting: Sendable {
    func loadPreferences() -> PersistedMobileDashboardPreferences?
    func savePreferences(_ preferences: PersistedMobileDashboardPreferences)
}

final class UserDefaultsMobileDashboardPreferencesStore: @unchecked Sendable, MobileDashboardPreferencesPersisting {
    private enum Keys {
        static let preferences = "heimdall.mobile.dashboard.preferences"
    }

    private let defaults: UserDefaults
    private let encoder = JSONEncoder()
    private let decoder = JSONDecoder()

    init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
    }

    func loadPreferences() -> PersistedMobileDashboardPreferences? {
        guard let data = self.defaults.data(forKey: Keys.preferences) else {
            return nil
        }
        return try? self.decoder.decode(PersistedMobileDashboardPreferences.self, from: data)
    }

    func savePreferences(_ preferences: PersistedMobileDashboardPreferences) {
        guard let data = try? self.encoder.encode(preferences) else {
            return
        }
        self.defaults.set(data, forKey: Keys.preferences)
    }
}
