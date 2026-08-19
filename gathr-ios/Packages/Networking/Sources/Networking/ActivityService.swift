import Foundation
import Models

public protocol ActivityService: Sendable {
    func feed(before: Date?, limit: Int?) async throws -> ActivityFeed
    func markRead(_ ids: [UUID]) async throws -> Int
}

public struct LiveActivityService: ActivityService {
    private let client: APIClient
    private let encoder = GathrJSON.encoder()

    public init(client: APIClient) {
        self.client = client
    }

    public func feed(before: Date? = nil, limit: Int? = nil) async throws -> ActivityFeed {
        var query: [URLQueryItem] = []
        if let before {
            query.append(URLQueryItem(name: "before", value: before.formatted(.iso8601)))
        }
        if let limit {
            query.append(URLQueryItem(name: "limit", value: String(limit)))
        }

        return try await client.send(
            Endpoint(path: "v1/notifications", query: query),
            as: ActivityFeed.self
        )
    }

    public func markRead(_ ids: [UUID]) async throws -> Int {
        struct Body: Encodable {
            let ids: [String]
        }
        struct Unread: Decodable {
            let unread: Int
        }

        let body = try encoder.encode(Body(ids: ids.map { $0.uuidString.lowercased() }))

        return try await client.send(
            Endpoint(method: .post, path: "v1/notifications/read", body: body),
            as: Unread.self
        ).unread
    }
}
