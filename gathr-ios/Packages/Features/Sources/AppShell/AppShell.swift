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
        ZStack(alignment: .bottomTrailing) {
            NavigationStack(path: Binding(get: { router.path }, set: { router.path = $0 })) {
                tabContent
                    .safeAreaInset(edge: .bottom) { tabBar }
                    .navigationDestination(for: Route.self) { route in
                        destination(route)
                    }
            }

            if selectedTab == .home, router.path.isEmpty {
                floatingActionButton
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
            ContentUnavailableView("Calendar is coming soon", systemImage: "calendar")
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

    private var tabBar: some View {
        HStack {
            ForEach(AppTab.allCases.filter(flags.isEnabled), id: \.self) { tab in
                Button {
                    selectedTab = tab
                    router.popToRoot()
                } label: {
                    Image(systemName: tab.symbol)
                        .font(.system(size: 20))
                        .foregroundStyle(selectedTab == tab ? Palette.accent : Palette.textTertiary)
                        .frame(maxWidth: .infinity)
                        .minimumHitTarget()
                }
                .accessibilityLabel(tab.title)
            }
        }
        .padding(.top, Spacing.stackGap)
        .background(Palette.surface)
        .overlay(alignment: .top) { Rectangle().fill(Palette.separator).frame(height: 0.5) }
    }

    private var floatingActionButton: some View {
        Button {
            isCreating = true
        } label: {
            Image(systemName: "plus")
                .font(.system(size: 24, weight: .semibold))
                .foregroundStyle(Palette.onAccent)
                .frame(width: 56, height: 56)
                .background(Palette.accent)
                .clipShape(Circle())
        }
        .accessibilityLabel("New event")
        .padding(.trailing, Spacing.gutter)
        .padding(.bottom, Spacing.stackGap)
    }
}
