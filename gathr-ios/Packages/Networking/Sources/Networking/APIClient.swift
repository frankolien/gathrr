import Foundation
import Models

public protocol TokenStorage: Sendable {
    func accessToken() async -> String?
}

public struct StaticTokenStorage: TokenStorage {
    private let token: String?

    public init(token: String?) {
        self.token = token
    }

    public func accessToken() async -> String? {
        token
    }
}

