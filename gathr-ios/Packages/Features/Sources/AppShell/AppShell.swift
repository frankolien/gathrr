import Calendar
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

    public init(exploreTab: Bool = true, calendarTab: Bool = true) {
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

    public var tabs: [AppTab] {
        AppTab.allCases.filter(isEnabled)
    }
}

public struct AppShell: View {
    private let events: any EventService
    private let auth: any AuthService
    private let activity: any ActivityService
    private let account: Account?
    private let flags: FeatureFlags
    private let onSignOut: () -> Void

    @State private var router = Router()
    @State private var home: HomeModel
    @State private var selectedTab: AppTab = .home
    @State private var isCreating = false
    @State private var isJoining = false

    public init(
        events: any EventService,
        auth: any AuthService,
        activity: any ActivityService,
        account: Account?,
        flags: FeatureFlags = FeatureFlags(),
        onSignOut: @escaping () -> Void
    ) {
        self.events = events
        self.auth = auth
        self.activity = activity
        self.account = account
        self.flags = flags
        self.onSignOut = onSignOut
        _home = State(initialValue: HomeModel(service: events))
    }

    public var body: some View {
        NavigationStack(path: Binding(get: { router.path }, set: { router.path = $0 })) {
            tabContent
                .safeAreaInset(edge: .bottom) { floatingBar }
                .navigationDestination(for: Route.self) { route in
                    destination(route)
                }
        }
        .sheet(isPresented: $isCreating) {
            CreateEventView(model: CreateEventModel(service: events)) { event in
                Task {
                    await home.load()
                    router.push(.eventDetail(event.id))
                }
            }
        }
        .sheet(isPresented: $isJoining) {
            JoinEventView(model: JoinEventModel(service: events)) { eventId in
                Task {
                    await home.load()
                    router.push(.eventDetail(eventId))
                }
            }
        }
    }

    @ViewBuilder
    private var tabContent: some View {
        switch selectedTab {
        case .home:
            HomeView(
                model: home,
                router: router,
                accountName: account?.displayName ?? "there"
            )
        case .explore:
            ContentUnavailableView("Explore is coming soon", systemImage: "safari")
        case .calendar:
            CalendarView(
                model: home,
                router: router,
                accountName: account?.displayName ?? "there"
            )
        case .profile:
            ProfileView(model: ProfileModel(auth: auth, account: account), onSignOut: onSignOut)
        }
    }

    @ViewBuilder
    private func destination(_ route: Route) -> some View {
        switch route {
        case .eventDetail(let id):
            EventDetailView(model: EventDetailModel(service: events, eventId: id), router: router)
        case .notifications:
            NotificationsView(model: NotificationsModel(service: activity), router: router)
        case .profile:
            ProfileView(model: ProfileModel(auth: auth, account: account), onSignOut: onSignOut)
        default:
            ContentUnavailableView("Not built yet", systemImage: "hammer")
        }
    }

    private var floatingBar: some View {
        LiquidTabBar(
            items: flags.tabs.map(\.item),
            selection: Binding(
                get: { selectedTab.rawValue },
                set: { raw in
                    selectedTab = AppTab(rawValue: raw) ?? .home
                    router.popToRoot()
                }
            )
        ) {
            isCreating = true
        }
        .padding(.bottom, Spacing.unit)
    }

}
