import Foundation

public struct Event: Codable, Sendable, Identifiable, Hashable {
    public let id: UUID
    public let title: String
    public let category: EventCategory
    public let locationName: String?
    public let startsAt: Date
    public let endsAt: Date?
    public let timezone: String
    public let status: EventStatus
    public let capacity: Int?
    public let goingGuests: Int
    public let previewGuestNames: [String]

    public init(
        id: UUID,
        title: String,
        category: EventCategory,
        locationName: String?,
        startsAt: Date,
        endsAt: Date?,
        timezone: String,
        status: EventStatus,
        capacity: Int?,
        goingGuests: Int = 0,
        previewGuestNames: [String] = []
    ) {
        self.id = id
        self.title = title
        self.category = category
        self.locationName = locationName
        self.startsAt = startsAt
        self.endsAt = endsAt
        self.timezone = timezone
        self.status = status
        self.capacity = capacity
        self.goingGuests = goingGuests
        self.previewGuestNames = previewGuestNames
    }
}

public struct EventDetail: Codable, Sendable, Identifiable, Hashable {
    public let id: UUID
    public let title: String
    public let category: EventCategory
    public let locationName: String?
    public let startsAt: Date
    public let endsAt: Date?
    public let timezone: String
    public let status: EventStatus
    public let capacity: Int?
    public let description: String?
    public let hostDisplayName: String
    public let observedStatus: EventStatus
    public let goingGuests: Int
    public let previewGuestNames: [String]
    public let maxPlusOnes: Int
    public let serverTime: Date

    public init(
        id: UUID,
        title: String,
        category: EventCategory,
        locationName: String?,
        startsAt: Date,
        endsAt: Date?,
        timezone: String,
        status: EventStatus,
        capacity: Int?,
        description: String?,
        hostDisplayName: String,
        observedStatus: EventStatus,
        goingGuests: Int,
        previewGuestNames: [String],
        maxPlusOnes: Int,
        serverTime: Date
    ) {
        self.id = id
        self.title = title
        self.category = category
        self.locationName = locationName
        self.startsAt = startsAt
        self.endsAt = endsAt
        self.timezone = timezone
        self.status = status
        self.capacity = capacity
        self.description = description
        self.hostDisplayName = hostDisplayName
        self.observedStatus = observedStatus
        self.goingGuests = goingGuests
        self.previewGuestNames = previewGuestNames
        self.maxPlusOnes = maxPlusOnes
        self.serverTime = serverTime
    }

    public var event: Event {
        Event(
            id: id,
            title: title,
            category: category,
            locationName: locationName,
            startsAt: startsAt,
            endsAt: endsAt,
            timezone: timezone,
            status: observedStatus,
            capacity: capacity,
            goingGuests: goingGuests,
            previewGuestNames: previewGuestNames
        )
    }

    public var clockSkew: TimeInterval {
        serverTime.timeIntervalSinceNow
    }
}

public struct Guest: Codable, Sendable, Identifiable, Hashable {
    public let userId: UUID
    public let displayName: String
    public let status: RSVPStatus
    public let plusOnes: Int

    public var id: UUID { userId }

    public init(userId: UUID, displayName: String, status: RSVPStatus, plusOnes: Int) {
        self.userId = userId
        self.displayName = displayName
        self.status = status
        self.plusOnes = plusOnes
    }
}

public struct GuestList: Codable, Sendable, Hashable {
    public let going: Int
    public let seatsTaken: Int
    public let guests: [Guest]

    public init(going: Int, seatsTaken: Int, guests: [Guest]) {
        self.going = going
        self.seatsTaken = seatsTaken
        self.guests = guests
    }

    public func grouped(by status: RSVPStatus) -> [Guest] {
        guests.filter { $0.status == status }
    }
}

public struct RSVP: Codable, Sendable, Hashable {
    public let eventId: UUID
    public let status: RSVPStatus
    public let plusOnes: Int
    public let enteredWaitlist: Bool
    public let seatsRemaining: Int?

    public init(
        eventId: UUID,
        status: RSVPStatus,
        plusOnes: Int,
        enteredWaitlist: Bool,
        seatsRemaining: Int?
    ) {
        self.eventId = eventId
        self.status = status
        self.plusOnes = plusOnes
        self.enteredWaitlist = enteredWaitlist
        self.seatsRemaining = seatsRemaining
    }
}

public struct Invite: Codable, Sendable, Identifiable, Hashable {
    public let id: UUID
    public let eventId: UUID
    public let code: String
    public let url: URL
    public let maxUses: Int?
    public let uses: Int
    public let expiresAt: Date?
}

public struct PublicInvite: Codable, Sendable, Hashable {
    public let eventId: UUID
    public let title: String
    public let category: EventCategory
    public let locationName: String?
    public let startsAt: Date
    public let timezone: String
    public let hostFirstName: String
    public let goingGuests: Int

    public init(
        eventId: UUID,
        title: String,
        category: EventCategory,
        locationName: String?,
        startsAt: Date,
        timezone: String,
        hostFirstName: String,
        goingGuests: Int
    ) {
        self.eventId = eventId
        self.title = title
        self.category = category
        self.locationName = locationName
        self.startsAt = startsAt
        self.timezone = timezone
        self.hostFirstName = hostFirstName
        self.goingGuests = goingGuests
    }
}

public struct Account: Codable, Sendable, Identifiable, Hashable {
    public let id: UUID
    public let displayName: String
    public let isGuest: Bool
    public let bio: String?
    public let avatarURL: URL?

    public init(
        id: UUID,
        displayName: String,
        isGuest: Bool,
        bio: String? = nil,
        avatarURL: URL? = nil
    ) {
        self.id = id
        self.displayName = displayName
        self.isGuest = isGuest
        self.bio = bio
        self.avatarURL = avatarURL
    }
}

