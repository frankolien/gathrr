import DesignSystem
import Models
import SwiftUI

struct ActivityLook {
    let symbol: String
    let tint: Color
}

extension ActivityKind {
    var look: ActivityLook {
        switch self {
        case .rsvpAccepted:
            ActivityLook(symbol: "checkmark", tint: Palette.statusGoing)
        case .rsvpDeclined:
            ActivityLook(symbol: "xmark", tint: Palette.statusDeclined)
        case .rsvpWaitlisted:
            ActivityLook(symbol: "hourglass", tint: Palette.statusWaitlisted)
        case .messagePosted:
            ActivityLook(symbol: "bubble.left.fill", tint: Palette.accent)
        case .eventPublished:
            ActivityLook(symbol: "sparkles", tint: Palette.accent)
        case .eventCancelled:
            ActivityLook(symbol: "calendar.badge.minus", tint: Palette.statusDeclined)
        case .eventReminder:
            ActivityLook(symbol: "bell.fill", tint: Palette.statusMaybe)
        case .unknown:
            ActivityLook(symbol: "sparkles", tint: Palette.textSecondary)
        }
    }

    func headline(actor: String?) -> LocalizedStringKey {
        let who = actor ?? String(localized: "Someone")

        return switch self {
        case .rsvpAccepted: "**\(who)** is going"
        case .rsvpDeclined: "**\(who)** can't make it"
        case .rsvpWaitlisted: "**\(who)** joined the waitlist"
        case .messagePosted: "**\(who)** sent a message"
        case .eventPublished: "Your event is live"
        case .eventCancelled: "**\(who)** called the event off"
        case .eventReminder: "Starting soon"
        case .unknown: "Something happened"
        }
    }
}
