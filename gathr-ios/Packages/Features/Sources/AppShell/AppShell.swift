import CreateEvent
import DesignSystem
import EventDetail
import Home
import JoinEvent
import Models
import Networking
import Notifications
import Profile
import Routing
import SwiftUI

public struct FeatureFlags: Sendable {
    public var exploreTab: Bool
    public var calendarTab: Bool

    public init(exploreTab: Bool = false, calendarTab: Bool = false) {
        self.exploreTab = exploreTab
        self.calendarTab = calendarTab
    }

    public func isEnabled(_ tab: AppTab) -> Bool {
        switch tab {
        case .explore: exploreTab
        case .calendar: calendarTab
        case .home, .profile: true
        }
    }
}

