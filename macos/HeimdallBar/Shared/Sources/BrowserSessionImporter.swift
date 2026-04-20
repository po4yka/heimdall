import CommonCrypto
import Foundation
import Security
import SQLite3

private let sqliteTransient = unsafeBitCast(-1, to: sqlite3_destructor_type.self)

public enum BrowserSessionImporterError: LocalizedError {
    case unreadableStore(String)
    case unsupportedStore(String)
    case openDatabase(String)
    case invalidBinaryCookies

    public var errorDescription: String? {
        switch self {
        case .unreadableStore(let path):
            return "Could not read browser cookie store at \(path)."
        case .unsupportedStore(let detail):
            return detail
        case .openDatabase(let path):
            return "Could not open browser cookie database at \(path)."
        case .invalidBinaryCookies:
            return "Safari cookie store is not in the expected binarycookies format."
        }
    }
}

public struct BrowserSessionImporter {
    private let fileManager: FileManager

    public init(fileManager: FileManager = .default) {
        self.fileManager = fileManager
    }

    public func discoverCandidates() -> [BrowserSessionImportCandidate] {
        safariCandidates() + chromiumCandidates(browser: .chrome) + chromiumCandidates(browser: .arc) + chromiumCandidates(browser: .brave)
    }

    public func importSession(
        provider: ProviderID,
        candidate: BrowserSessionImportCandidate
    ) throws -> ImportedBrowserSession {
        let cookies: [ImportedSessionCookie]
        switch candidate.browserSource {
        case .safari:
            cookies = try self.parseSafariBinaryCookies(at: URL(fileURLWithPath: candidate.storePath), provider: provider)
        case .chrome, .arc, .brave:
            cookies = try self.parseChromiumCookies(at: URL(fileURLWithPath: candidate.storePath), provider: provider)
        }

        let deduped = Array(Dictionary(uniqueKeysWithValues: cookies.map { ($0.id, $0) }).values)
            .sorted {
                if $0.domain == $1.domain {
                    return $0.name < $1.name
                }
                return $0.domain < $1.domain
            }
        let minimizedCookies = self.minimizeImportedCookies(provider: provider, cookies: deduped)
        let hasActiveAuthCookie = minimizedCookies.contains { cookie in
            !isExpired(cookie) && isLikelyAuthCookie(provider: provider, cookieName: cookie.name)
        }
        let importedAt = ISO8601DateFormatter().string(from: Date())

        return ImportedBrowserSession(
            provider: provider,
            browserSource: candidate.browserSource,
            profileName: candidate.profileName,
            importedAt: importedAt,
            storageKind: candidate.storageKind,
            cookies: minimizedCookies,
            loginRequired: minimizedCookies.isEmpty || !hasActiveAuthCookie,
            expired: !minimizedCookies.isEmpty && !hasActiveAuthCookie,
            lastValidatedAt: importedAt
        )
    }

    private func safariCandidates() -> [BrowserSessionImportCandidate] {
        let cookieURL = self.fileManager.homeDirectoryForCurrentUser
            .appendingPathComponent("Library", isDirectory: true)
            .appendingPathComponent("Cookies", isDirectory: true)
            .appendingPathComponent("Cookies.binarycookies", isDirectory: false)
        guard self.fileManager.fileExists(atPath: cookieURL.path) else { return [] }
        return [
            BrowserSessionImportCandidate(
                browserSource: .safari,
                profileName: "Default",
                storePath: cookieURL.path,
                storageKind: "binarycookies"
            )
        ]
    }

    private func chromiumCandidates(browser: BrowserSource) -> [BrowserSessionImportCandidate] {
        let relativeBase: String
        switch browser {
        case .chrome:
            relativeBase = "Library/Application Support/Google/Chrome"
        case .arc:
            relativeBase = "Library/Application Support/Arc/User Data"
        case .brave:
            relativeBase = "Library/Application Support/BraveSoftware/Brave-Browser"
        case .safari:
            return []
        }

        let baseURL = self.fileManager.homeDirectoryForCurrentUser.appendingPathComponent(relativeBase, isDirectory: true)
        guard self.fileManager.fileExists(atPath: baseURL.path) else { return [] }

        let profileNames = ["Default"] + (1...9).map { "Profile \($0)" }
        return profileNames.compactMap { profileName in
            let cookiesURL = baseURL.appendingPathComponent(profileName, isDirectory: true).appendingPathComponent("Cookies", isDirectory: false)
            guard self.fileManager.fileExists(atPath: cookiesURL.path) else { return nil }
            return BrowserSessionImportCandidate(
                browserSource: browser,
                profileName: profileName,
                storePath: cookiesURL.path,
                storageKind: "sqlite"
            )
        }
    }

    private func parseChromiumCookies(
        at sourceURL: URL,
        provider: ProviderID
    ) throws -> [ImportedSessionCookie] {
        var database: OpaquePointer?
        let databaseURI = "file:\(sourceURL.path.addingPercentEncoding(withAllowedCharacters: .urlPathAllowed) ?? sourceURL.path)?mode=ro&immutable=1"
        guard sqlite3_open_v2(databaseURI, &database, SQLITE_OPEN_READONLY | SQLITE_OPEN_URI, nil) == SQLITE_OK else {
            throw BrowserSessionImporterError.openDatabase(sourceURL.path)
        }
        defer { sqlite3_close(database) }

        let domains = self.providerDomains(provider)
        let sql = """
        SELECT host_key, name, value, path, is_secure, is_httponly, expires_utc, encrypted_value
        FROM cookies
        WHERE \(domains.enumerated().map { index, _ in "host_key LIKE ?\(index + 1)" }.joined(separator: " OR "))
        """

        var statement: OpaquePointer?
        guard sqlite3_prepare_v2(database, sql, -1, &statement, nil) == SQLITE_OK else {
            throw BrowserSessionImporterError.openDatabase(sourceURL.path)
        }
        defer { sqlite3_finalize(statement) }

        for (index, domain) in domains.enumerated() {
            sqlite3_bind_text(statement, Int32(index + 1), "%\(domain)", -1, sqliteTransient)
        }

        var cookies = [ImportedSessionCookie]()
        while sqlite3_step(statement) == SQLITE_ROW {
            guard
                let hostKey = sqlite3_column_text(statement, 0).map({ String(cString: $0) }),
                let name = sqlite3_column_text(statement, 1).map({ String(cString: $0) }),
                let path = sqlite3_column_text(statement, 3).map({ String(cString: $0) })
            else {
                continue
            }

            let plainValue = sqlite3_column_text(statement, 2).map { String(cString: $0) }
            let encryptedValue = sqliteBlob(statement: statement, column: 7)
            let secure = sqlite3_column_int(statement, 4) != 0
            let httpOnly = sqlite3_column_int(statement, 5) != 0
            let expiresUTC = sqlite3_column_int64(statement, 6)
            let expiry = chromeDateString(from: expiresUTC)
            let cookie = ImportedSessionCookie(
                domain: hostKey,
                name: name,
                value: self.chromiumCookieValue(
                    plainValue: plainValue,
                    encryptedValue: encryptedValue,
                    browser: candidateBrowser(for: sourceURL)
                ),
                path: path,
                expiresAt: expiry,
                secure: secure,
                httpOnly: httpOnly
            )
            if self.domainMatches(provider, domain: cookie.domain) {
                cookies.append(cookie)
            }
        }
        return cookies
    }

    private func parseSafariBinaryCookies(
        at sourceURL: URL,
        provider: ProviderID
    ) throws -> [ImportedSessionCookie] {
        guard let data = try? Data(contentsOf: sourceURL), data.count > 8 else {
            throw BrowserSessionImporterError.unreadableStore(sourceURL.path)
        }
        guard String(data: data.prefix(4), encoding: .ascii) == "cook" else {
            throw BrowserSessionImporterError.invalidBinaryCookies
        }

        let reader = BinaryDataReader(data: data)
        guard let pageCount = reader.uint32BigEndian(at: 4) else {
            throw BrowserSessionImporterError.invalidBinaryCookies
        }

        var pageSizes = [Int]()
        var cursor = 8
        for _ in 0..<pageCount {
            guard let size = reader.uint32BigEndian(at: cursor) else {
                throw BrowserSessionImporterError.invalidBinaryCookies
            }
            pageSizes.append(Int(size))
            cursor += 4
        }

        var cookies = [ImportedSessionCookie]()
        for pageSize in pageSizes {
            let pageRange = cursor..<(cursor + pageSize)
            guard pageRange.upperBound <= data.count else { break }
            let pageData = data.subdata(in: pageRange)
            cookies.append(contentsOf: parseSafariPage(pageData, provider: provider))
            cursor += pageSize
        }
        return cookies
    }

    private func parseSafariPage(_ pageData: Data, provider: ProviderID) -> [ImportedSessionCookie] {
        let reader = BinaryDataReader(data: pageData)
        guard let cookieCount = reader.uint32LittleEndian(at: 4) else { return [] }

        var cookies = [ImportedSessionCookie]()
        for index in 0..<cookieCount {
            let offsetPosition = 8 + (Int(index) * 4)
            guard let cookieOffset = reader.uint32LittleEndian(at: offsetPosition) else { continue }
            let offset = Int(cookieOffset)
            guard let cookieSize = reader.uint32LittleEndian(at: offset) else { continue }
            let range = offset..<(offset + Int(cookieSize))
            guard range.upperBound <= pageData.count else { continue }
            let cookieData = pageData.subdata(in: range)
            if let cookie = parseSafariCookie(cookieData), self.domainMatches(provider, domain: cookie.domain) {
                cookies.append(cookie)
            }
        }
        return cookies
    }

    private func parseSafariCookie(_ cookieData: Data) -> ImportedSessionCookie? {
        let reader = BinaryDataReader(data: cookieData)
        guard
            cookieData.count >= 56,
            let flags = reader.uint32LittleEndian(at: 8),
            let domainOffset = reader.uint32LittleEndian(at: 16),
            let nameOffset = reader.uint32LittleEndian(at: 20),
            let pathOffset = reader.uint32LittleEndian(at: 24),
            let valueOffset = reader.uint32LittleEndian(at: 28),
            let expiresDouble = reader.doubleLittleEndian(at: 40),
            let domain = reader.nullTerminatedString(at: Int(domainOffset)),
            let name = reader.nullTerminatedString(at: Int(nameOffset)),
            let path = reader.nullTerminatedString(at: Int(pathOffset))
        else {
            return nil
        }

        let expiryDate = expiresDouble > 0 ? ISO8601DateFormatter().string(from: Date(timeIntervalSinceReferenceDate: expiresDouble)) : nil
        return ImportedSessionCookie(
            domain: domain,
            name: name,
            value: reader.nullTerminatedString(at: Int(valueOffset)),
            path: path,
            expiresAt: expiryDate,
            secure: (flags & 1) != 0,
            httpOnly: (flags & 4) != 0
        )
    }

    private func providerDomains(_ provider: ProviderID) -> [String] {
        switch provider {
        case .claude:
            return ["claude.ai", "anthropic.com", "console.anthropic.com"]
        case .codex:
            return ["platform.openai.com", "chat.openai.com", "chatgpt.com", "openai.com"]
        }
    }

    private func domainMatches(_ provider: ProviderID, domain: String) -> Bool {
        let normalizedDomain = domain.trimmingCharacters(in: CharacterSet(charactersIn: ".")).lowercased()
        return self.providerDomains(provider).contains { normalizedDomain == $0 || normalizedDomain.hasSuffix(".\($0)") }
    }

    private func isLikelyAuthCookie(provider: ProviderID, cookieName: String) -> Bool {
        let normalized = cookieName.lowercased()
        let generic = ["session", "token", "auth"]
        let providerSpecific: [String]
        switch provider {
        case .claude:
            providerSpecific = ["claude", "__session", "sessionkey"]
        case .codex:
            providerSpecific = ["openai", "next-auth", "_puid", "oai"]
        }
        return (generic + providerSpecific).contains { normalized.contains($0) }
    }

    private func minimizeImportedCookies(
        provider: ProviderID,
        cookies: [ImportedSessionCookie]
    ) -> [ImportedSessionCookie] {
        cookies.filter { cookie in
            self.isLikelyAuthCookie(provider: provider, cookieName: cookie.name)
        }
    }

    private func isExpired(_ cookie: ImportedSessionCookie) -> Bool {
        guard let expiresAt = cookie.expiresAt else { return false }
        guard let expiryDate = ISO8601DateFormatter().date(from: expiresAt) else { return false }
        return expiryDate <= Date()
    }

    private func candidateBrowser(for sourceURL: URL) -> BrowserSource {
        let path = sourceURL.path.lowercased()
        if path.contains("/arc/") {
            return .arc
        }
        if path.contains("/bravesoftware/") {
            return .brave
        }
        return .chrome
    }

    private func chromiumCookieValue(
        plainValue: String?,
        encryptedValue: Data?,
        browser: BrowserSource
    ) -> String? {
        if let plainValue, !plainValue.isEmpty {
            return plainValue
        }
        guard let encryptedValue, !encryptedValue.isEmpty else {
            return nil
        }
        guard encryptedValue.count > 3 else {
            return String(data: encryptedValue, encoding: .utf8)
        }
        if encryptedValue.starts(with: Data("v10".utf8)) || encryptedValue.starts(with: Data("v11".utf8)) {
            return self.decryptLegacyChromiumCookie(encryptedValue, browser: browser)
        }
        return String(data: encryptedValue, encoding: .utf8)
    }

    private func decryptLegacyChromiumCookie(_ encryptedValue: Data, browser: BrowserSource) -> String? {
        guard let password = self.safeStoragePassword(for: browser) else { return nil }
        let salt = Data("saltysalt".utf8)
        let iv = Data(repeating: 0x20, count: kCCBlockSizeAES128)
        let payload = encryptedValue.dropFirst(3)
        let derivedKey = self.pbkdf2Key(password: password, salt: salt)
        return self.aesCBCDecrypt(payload: Data(payload), key: derivedKey, iv: iv)
    }

    private func safeStoragePassword(for browser: BrowserSource) -> Data? {
        let service: String
        let account: String
        switch browser {
        case .chrome:
            service = "Chrome Safe Storage"
            account = "Chrome"
        case .arc:
            service = "Arc Safe Storage"
            account = "Arc"
        case .brave:
            service = "Brave Safe Storage"
            account = "Brave"
        case .safari:
            return nil
        }

        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrService: service,
            kSecAttrAccount: account,
            kSecReturnData: true,
            kSecMatchLimit: kSecMatchLimitOne,
        ]

        var item: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &item)
        guard status == errSecSuccess, let passwordData = item as? Data else {
            return nil
        }
        return passwordData
    }

    private func pbkdf2Key(password: Data, salt: Data) -> Data {
        let keyLength = kCCKeySizeAES128
        var key = Data(repeating: 0, count: keyLength)
        _ = key.withUnsafeMutableBytes { keyBytes in
            salt.withUnsafeBytes { saltBytes in
                password.withUnsafeBytes { passwordBytes in
                    CCKeyDerivationPBKDF(
                        CCPBKDFAlgorithm(kCCPBKDF2),
                        passwordBytes.bindMemory(to: Int8.self).baseAddress,
                        password.count,
                        saltBytes.bindMemory(to: UInt8.self).baseAddress,
                        salt.count,
                        CCPseudoRandomAlgorithm(kCCPRFHmacAlgSHA1),
                        1003,
                        keyBytes.bindMemory(to: UInt8.self).baseAddress,
                        keyLength
                    )
                }
            }
        }
        return key
    }

    private func aesCBCDecrypt(payload: Data, key: Data, iv: Data) -> String? {
        let outputCapacity = payload.count + kCCBlockSizeAES128
        var output = Data(repeating: 0, count: outputCapacity)
        var outputLength = 0

        let status = output.withUnsafeMutableBytes { outputBytes in
            payload.withUnsafeBytes { payloadBytes in
                key.withUnsafeBytes { keyBytes in
                    iv.withUnsafeBytes { ivBytes in
                        CCCrypt(
                            CCOperation(kCCDecrypt),
                            CCAlgorithm(kCCAlgorithmAES),
                            CCOptions(kCCOptionPKCS7Padding),
                            keyBytes.baseAddress,
                            key.count,
                            ivBytes.baseAddress,
                            payloadBytes.baseAddress,
                            payload.count,
                            outputBytes.baseAddress,
                            outputCapacity,
                            &outputLength
                        )
                    }
                }
            }
        }

        guard status == kCCSuccess else { return nil }
        output.removeSubrange(outputLength..<output.count)
        return String(data: output, encoding: .utf8)
    }

    private func chromeDateString(from microsecondsSince1601: Int64) -> String? {
        guard microsecondsSince1601 > 0 else { return nil }
        let seconds = TimeInterval(microsecondsSince1601) / 1_000_000 - 11_644_473_600
        guard seconds.isFinite else { return nil }
        return ISO8601DateFormatter().string(from: Date(timeIntervalSince1970: seconds))
    }
}

private func sqliteBlob(statement: OpaquePointer?, column: Int32) -> Data? {
    let length = Int(sqlite3_column_bytes(statement, column))
    guard length > 0, let bytes = sqlite3_column_blob(statement, column) else {
        return nil
    }
    return Data(bytes: bytes, count: length)
}

private struct BinaryDataReader {
    let data: Data

    func uint32BigEndian(at offset: Int) -> UInt32? {
        guard offset >= 0, offset + 4 <= self.data.count else { return nil }
        return self.data.subdata(in: offset..<(offset + 4)).withUnsafeBytes { bytes in
            bytes.load(as: UInt32.self).bigEndian
        }
    }

    func uint32LittleEndian(at offset: Int) -> UInt32? {
        guard offset >= 0, offset + 4 <= self.data.count else { return nil }
        return self.data.subdata(in: offset..<(offset + 4)).withUnsafeBytes { bytes in
            bytes.load(as: UInt32.self).littleEndian
        }
    }

    func doubleLittleEndian(at offset: Int) -> Double? {
        guard offset >= 0, offset + 8 <= self.data.count else { return nil }
        let bitPattern = self.data.subdata(in: offset..<(offset + 8)).withUnsafeBytes { bytes in
            bytes.load(as: UInt64.self).littleEndian
        }
        return Double(bitPattern: bitPattern)
    }

    func nullTerminatedString(at offset: Int) -> String? {
        guard offset >= 0, offset < self.data.count else { return nil }
        let suffix = self.data[offset...]
        guard let endIndex = suffix.firstIndex(of: 0) else { return nil }
        return String(data: self.data[offset..<endIndex], encoding: .utf8)
    }
}
