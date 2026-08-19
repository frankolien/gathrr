import Foundation

public enum ActivityKind: String, Codable, Sendable, CaseIterable {
    case rsvpAccepted = "rsvp_accepted"
    case rsvpDeclined = "rsvp_declined"
    case rsvpWaitlisted = "rsvp_waitlisted"
    case messagePosted = "message_posted"
    case eventPublished = "event_published"
    case eventCancelled = "event_cancelled"
    case eventReminder = "event_reminder"
    case unknown

    public init(from decoder: any Decoder) throws {
        let raw = try decoder.singleValueContainer().decode(String.self)
        self = ActivityKind(rawValue: raw) ?? .unknown
    }
}

public struct ActivityEntry: Codable, Sendable, Identifiable, Hashable {
    public let id: UUID
    public let kind: ActivityKind
    public let eventId: UUID
    public let eventTitle: String
    public let actorDisplayName: String?
    public let read: Bool
    public let createdAt: Date

    public init(
        id: UUID,
        kind: ActivityKind,
        eventId: UUID,
        eventTitle: String,
        actorDisplayName: String?,
        read: Bool,
        createdAt: Date
    ) {
        self.id = id
        self.kind = kind
        self.eventId = eventId
        self.eventTitle = eventTitle
        self.actorDisplayName = actorDisplayName
        self.read = read
        self.createdAt = createdAt
    }
}

