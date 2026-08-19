import Foundation
import Models

public struct NewEvent: Sendable, Equatable {
    public var title: String
    public var category: EventCategory
    public var description: String?
    public var locationName: String?
    public var startsAt: Date
    public var capacity: Int?
    public var publishNow: Bool

    public init(
        title: String = "",
        category: EventCategory = .other,
        description: String? = nil,
        locationName: String? = nil,
        startsAt: Date = Date().addingTimeInterval(86_400),
        capacity: Int? = nil,
        publishNow: Bool = true
    ) {
        self.title = title
        self.category = category
        self.description = description
        self.locationName = locationName
        self.startsAt = startsAt
        self.capacity = capacity
        self.publishNow = publishNow
    }

    public var isValid: Bool {
        !title.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
    }
}

public protocol EventService: Sendable {
    func create(_ draft: NewEvent, idempotencyKey: String) async throws -> Event
    func feed(_ filter: FeedFilter) async throws -> [Event]
    func detail(_ id: UUID) async throws -> EventDetail
    func guests(_ id: UUID) async throws -> GuestList
    func rsvp(
        _ id: UUID,
        status: RSVPStatus,
        plusOnes: Int,
        acceptWaitlist: Bool,
        idempotencyKey: String
    ) async throws -> RSVP
    func createInvite(_ id: UUID) async throws -> Invite
    func resolveInvite(code: String) async throws -> PublicInvite
}

public struct LiveEventService: EventService {
    private let client: APIClient
    private let encoder = GathrJSON.encoder()

    public init(client: APIClient) {
        self.client = client
    }

    public func create(_ draft: NewEvent, idempotencyKey: String) async throws -> Event {
        struct Body: Encodable {
            let title: String
            let category: String
            let description: String?
            let locationName: String?
            let startsAt: Date
            let timezone: String
            let capacity: Int?
            let publishNow: Bool
        }

        let body = try encoder.encode(
            Body(
                title: draft.title.trimmingCharacters(in: .whitespacesAndNewlines),
                category: draft.category.rawValue,
                description: draft.description,
                locationName: draft.locationName,
                startsAt: draft.startsAt,
                timezone: "Africa/Lagos",
                capacity: draft.capacity,
                publishNow: draft.publishNow
            )
        )

        return try await client.send(
            Endpoint(
                method: .post,
                path: "v1/events",
                body: body,
                idempotencyKey: idempotencyKey
            ),
            as: Event.self
        )
    }

    public func feed(_ filter: FeedFilter) async throws -> [Event] {
        try await client.send(
            Endpoint(path: "v1/events", query: [URLQueryItem(name: "filter", value: filter.rawValue)]),
            as: [Event].self
        )
    }

    public func detail(_ id: UUID) async throws -> EventDetail {
        try await client.send(
            Endpoint(path: "v1/events/\(id.uuidString.lowercased())"),
            as: EventDetail.self
        )
    }

    public func guests(_ id: UUID) async throws -> GuestList {
        try await client.send(
            Endpoint(path: "v1/events/\(id.uuidString.lowercased())/guests"),
            as: GuestList.self
        )
    }

    public func rsvp(
        _ id: UUID,
        status: RSVPStatus,
        plusOnes: Int,
        acceptWaitlist: Bool,
        idempotencyKey: String
    ) async throws -> RSVP {
        struct Body: Encodable {
            let status: RSVPStatus
            let plusOnes: Int
            let acceptWaitlist: Bool
        }

        let body = try encoder.encode(
            Body(status: status, plusOnes: plusOnes, acceptWaitlist: acceptWaitlist)
        )

        return try await client.send(
            Endpoint(
                method: .post,
                path: "v1/events/\(id.uuidString.lowercased())/rsvp",
                body: body,
                idempotencyKey: idempotencyKey
            ),
            as: RSVP.self
        )
    }

    public func createInvite(_ id: UUID) async throws -> Invite {
        try await client.send(
            Endpoint(
                method: .post,
                path: "v1/events/\(id.uuidString.lowercased())/invites",
                body: Data("{}".utf8)
            ),
            as: Invite.self
        )
    }

    public func resolveInvite(code: String) async throws -> PublicInvite {
        try await client.send(
            Endpoint(path: "v1/invites/\(code)", requiresAuth: false),
            as: PublicInvite.self
        )
    }
}
