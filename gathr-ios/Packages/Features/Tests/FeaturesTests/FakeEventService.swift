import Foundation
import Models
import Networking
import Testing

actor FakeEventService: EventService {
    private var feedResult: Result<[Event], GathrError>
    private var rsvpResults: [Result<RSVP, GathrError>]
    private(set) var sentKeys: [String] = []
    private(set) var feedCalls: [FeedFilter] = []

    var createResult: Result<Event, GathrError>?
    var inviteResult: Result<PublicInvite, GathrError>?

    init(
        feedResult: Result<[Event], GathrError> = .success([]),
        rsvpResults: [Result<RSVP, GathrError>] = []
    ) {
        self.feedResult = feedResult
        self.rsvpResults = rsvpResults
    }

    func create(_ draft: NewEvent, idempotencyKey: String) async throws -> Event {
        sentKeys.append(idempotencyKey)
        if let createResult { return try createResult.get() }
        return Event(
            id: UUID(),
            title: draft.title,
            category: draft.category,
            locationName: draft.locationName,
            startsAt: draft.startsAt,
            endsAt: nil,
            timezone: "Africa/Lagos",
            status: draft.publishNow ? .published : .draft,
            capacity: draft.capacity
        )
    }

    func failCreation(with error: GathrError) {
        createResult = .failure(error)
    }

    func feed(_ filter: FeedFilter) async throws -> [Event] {
        feedCalls.append(filter)
        return try feedResult.get()
    }

    func detail(_ id: UUID) async throws -> EventDetail {
        EventDetail(
            id: id,
            title: "Amara's 26th Birthday",
            category: .birthday,
            locationName: "Victoria Island, Lagos",
            startsAt: Date().addingTimeInterval(86_400),
            endsAt: nil,
            timezone: "Africa/Lagos",
            status: .published,
            capacity: 20,
            description: "Good food, good music.",
            hostDisplayName: "Amara Chukwu",
            observedStatus: .published,
            goingGuests: 18,
            previewGuestNames: ["Tunde Bello"],
            maxPlusOnes: 2,
            serverTime: Date()
        )
    }

    func guests(_ id: UUID) async throws -> GuestList {
        GuestList(going: 18, seatsTaken: 20, guests: [])
    }

    func rsvp(
        _ id: UUID,
        status: RSVPStatus,
        plusOnes: Int,
        acceptWaitlist: Bool,
        idempotencyKey: String
    ) async throws -> RSVP {
        sentKeys.append(idempotencyKey)
        if rsvpResults.isEmpty {
            return RSVP(
                eventId: id,
                status: status,
                plusOnes: plusOnes,
                enteredWaitlist: false,
                seatsRemaining: nil
            )
        }
        return try rsvpResults.removeFirst().get()
    }

    func createInvite(_ id: UUID) async throws -> Invite {
        throw GathrError.transport("not used")
    }

    func failInviteResolution(with error: GathrError) {
        inviteResult = .failure(error)
    }

    func resolveInvite(code: String) async throws -> PublicInvite {
        if let inviteResult { return try inviteResult.get() }
        return PublicInvite(
            eventId: UUID(),
            title: "Amara's 26th Birthday",
            category: .birthday,
            locationName: "Victoria Island, Lagos",
            startsAt: Date().addingTimeInterval(86_400),
            timezone: "Africa/Lagos",
            hostFirstName: "Amara",
            goingGuests: 18
        )
    }

    func keys() -> [String] { sentKeys }
    func filters() -> [FeedFilter] { feedCalls }
}
