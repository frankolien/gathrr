import Foundation

public enum EventCategory: String, Codable, Sendable, CaseIterable {
    case birthday
    case party
    case meetup
    case dinner
    case gameNight = "game_night"
    case wedding
    case other

    public init(from decoder: any Decoder) throws {
        let raw = try decoder.singleValueContainer().decode(String.self)
        self = EventCategory(rawValue: raw) ?? .other
    }
}

public enum EventStatus: String, Codable, Sendable {
    case draft
    case published
    case ongoing
    case ended
    case cancelled

    public var acceptsRSVPs: Bool {
        self == .published || self == .ongoing
    }
}

public enum RSVPStatus: String, Codable, Sendable, CaseIterable {
    case invited
    case going
    case maybe
    case declined
    case waitlisted

    public var isGuestSelectable: Bool {
        self == .going || self == .maybe || self == .declined
    }

    public var holdsSeats: Bool {
        self == .going
    }
}

public enum FeedFilter: String, Sendable, CaseIterable {
    case thisWeek = "this_week"
    case hosting
    case attending
}
