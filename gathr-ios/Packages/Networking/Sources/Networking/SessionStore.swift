import Foundation
import Models

public actor SessionStore: TokenStorage {
    private let service: String
    private var cached: TokenPair?

    public init(service: String = "app.gathr.session") {
        self.service = service
        cached = Self.read(service: service)
    }

    public func accessToken() async -> String? {
        cached?.accessToken
    }

    public func current() -> TokenPair? {
        cached
    }

    public func save(_ pair: TokenPair) {
        cached = pair
        guard let data = try? GathrJSON.encoder().encode(pair) else { return }
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: pair.userId.uuidString,
        ]
        SecItemDelete(query as CFDictionary)
        var insert = query
        insert[kSecValueData as String] = data
        insert[kSecAttrAccessible as String] = kSecAttrAccessibleAfterFirstUnlock
        SecItemAdd(insert as CFDictionary, nil)
    }

    public func clear() {
        cached = nil
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
        ]
        SecItemDelete(query as CFDictionary)
    }

    private static func read(service: String) -> TokenPair? {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecReturnData as String: true,
            kSecMatchLimit as String: kSecMatchLimitOne,
        ]
        var item: CFTypeRef?
        guard SecItemCopyMatching(query as CFDictionary, &item) == errSecSuccess,
            let data = item as? Data
        else {
            return nil
        }
        return try? GathrJSON.decoder().decode(TokenPair.self, from: data)
    }
}
