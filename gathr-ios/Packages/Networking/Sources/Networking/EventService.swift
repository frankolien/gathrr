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

