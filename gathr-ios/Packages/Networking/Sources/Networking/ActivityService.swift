import Foundation
import Models

public protocol ActivityService: Sendable {
    func feed(before: Date?, limit: Int?) async throws -> ActivityFeed
    func markRead(_ ids: [UUID]) async throws -> Int
}

