import Foundation
import Security

public struct KeychainStore: Sendable {
    public let service: String

    public init(service: String = "dev.heimdall.HeimdallBar") {
        self.service = service
    }

    public func load(account: String) -> Data? {
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrService: self.service,
            kSecAttrAccount: account,
            kSecReturnData: true,
            kSecMatchLimit: kSecMatchLimitOne,
        ]

        var item: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &item)
        guard status == errSecSuccess else { return nil }
        return item as? Data
    }

    public func save(_ data: Data, account: String) throws {
        let baseQuery: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrService: self.service,
            kSecAttrAccount: account,
        ]

        let attributes: [CFString: Any] = [
            kSecValueData: data,
        ]

        let updateStatus = SecItemUpdate(baseQuery as CFDictionary, attributes as CFDictionary)
        if updateStatus == errSecSuccess {
            return
        }

        var createQuery = baseQuery
        createQuery[kSecValueData] = data
        let createStatus = SecItemAdd(createQuery as CFDictionary, nil)
        guard createStatus == errSecSuccess else {
            throw NSError(domain: NSOSStatusErrorDomain, code: Int(createStatus))
        }
    }

    public func delete(account: String) throws {
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrService: self.service,
            kSecAttrAccount: account,
        ]
        let status = SecItemDelete(query as CFDictionary)
        guard status == errSecSuccess || status == errSecItemNotFound else {
            throw NSError(domain: NSOSStatusErrorDomain, code: Int(status))
        }
    }

    public func loadJSON<T: Decodable>(_ type: T.Type, account: String) -> T? {
        guard let data = self.load(account: account) else { return nil }
        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        return try? decoder.decode(T.self, from: data)
    }

    public func saveJSON<T: Encodable>(_ value: T, account: String) throws {
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.sortedKeys]
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let data = try encoder.encode(value)
        try self.save(data, account: account)
    }
}
