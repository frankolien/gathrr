import DesignSystem
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
        case .home: "house"
        case .explore: "safari"
        case .calendar: "calendar"
        case .profile: "person"
        }
    }

    public var selectedSymbol: String {
        switch self {
        case .home: "house.fill"
        case .explore: "safari.fill"
        case .calendar: "calendar"
        case .profile: "person.fill"
        }
    }

    public var item: TabItem {
        TabItem(id: rawValue, symbol: symbol, selectedSymbol: selectedSymbol, title: title)
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

@MainActor
@Observable
public final class Router {
    public var path: [Route] = []

    public init() {}

    public func push(_ route: Route) {
        path.append(route)
    }

    public func pop() {
        guard !path.isEmpty else { return }
        path.removeLast()
    }

    public func popToRoot() {
        path.removeAll()
    }

    public func replace(with route: Route) {
        path = [route]
    }

    public func handle(universalLink url: URL) -> Bool {
        let parts = url.pathComponents.filter { $0 != "/" }
        guard parts.count == 2, parts[0] == "i" else { return false }
        push(.invite(parts[1]))
        return true
    }
}
