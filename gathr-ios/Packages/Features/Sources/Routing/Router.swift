import Foundation
import Models
import Observation

public enum Route: Hashable, Sendable {
    case eventDetail(UUID)
    case guestList(UUID)
    case invite(String)
    case feed(FeedFilter)
    case createEvent
    case joinEvent
    case notifications
    case profile
}

public enum AppTab: String, Hashable, Sendable, CaseIterable {
    case home
    case explore
    case calendar
    case profile

    public var symbol: String {
        switch self {
        case .home: "house.fill"
        case .explore: "safari"
        case .calendar: "calendar"
        case .profile: "person"
        }
    }

    public var title: String {
        switch self {
        case .home: "Home"
        case .explore: "Explore"
        case .calendar: "Calendar"
        case .profile: "Profile"
        }
    }
}

