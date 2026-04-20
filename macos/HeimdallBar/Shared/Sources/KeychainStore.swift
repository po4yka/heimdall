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
}
