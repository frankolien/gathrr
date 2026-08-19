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

