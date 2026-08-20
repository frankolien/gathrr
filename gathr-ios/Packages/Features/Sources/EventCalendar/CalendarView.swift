import DesignSystem
import Home
import Models
import Routing
import SwiftUI

public struct CalendarView: View {
    @State private var model: HomeModel
    private let router: Router
    private let accountName: String
    private let onCreate: () -> Void
    private let onJoin: () -> Void

    public init(
        model: HomeModel,
        router: Router,
        accountName: String,
        onCreate: @escaping () -> Void,
        onJoin: @escaping () -> Void
    ) {
        _model = State(initialValue: model)
        self.router = router
        self.accountName = accountName
        self.onCreate = onCreate
        self.onJoin = onJoin
    }

    public var body: some View {
        GradientHeaderScreen {
            header
        } content: {
            agenda
        }
        .refreshable { await model.load() }
        .task { await model.load() }
    }

    private var header: some View {
        VStack(alignment: .leading, spacing: Spacing.gutter) {
            HStack(alignment: .center) {
                VStack(alignment: .leading, spacing: 1) {
                    Text(model.greeting().uppercased())
                        .font(Typography.eyebrow)
                        .tracking(0.6)
                        .foregroundStyle(Palette.onHeaderMuted)
                    Text(accountName)
                        .font(Typography.titleM)
                        .foregroundStyle(Palette.onHeader)
                }
                Spacer()
                Button {
                    router.push(.notifications)
                } label: {
                    Image(systemName: "bell")
                        .font(.system(size: 15, weight: .medium))
                        .foregroundStyle(Palette.onHeader)
                        .frame(width: 36, height: 36)
                        .background(
                            Palette.headerGlass,
                            in: RoundedRectangle(cornerRadius: Radius.thumb, style: .continuous)
                        )
                }
                .accessibilityLabel("Notifications")
            }

            WeekStrip(days: model.week())

            VStack(alignment: .leading, spacing: Spacing.unit) {
                Text("This week")
                    .font(Typography.subhead)
                    .foregroundStyle(Palette.onHeaderMuted)

                HStack(alignment: .center) {
                    HStack(alignment: .firstTextBaseline, spacing: 5) {
                        Text("\(model.plannedCount())")
                            .font(.system(size: 40, weight: .medium, design: .rounded))
                            .monospacedDigit()
                            .foregroundStyle(Palette.onHeader)
                        Text(model.plannedCount() == 1 ? "event" : "events")
                            .font(Typography.subhead)
                            .foregroundStyle(Palette.onHeaderMuted)
                    }
                    .accessibilityElement(children: .combine)
                    Spacer()
                    if model.plannedCount() > 0 {
                        ActivePill("Active")
                    }
                }
            }

            HStack(spacing: 0) {
                HeaderStat(value: model.todayCount(), label: "Today")
                HeaderStat(value: model.upcomingCount(), label: "Upcoming")
                HeaderStat(value: model.hostingCount, label: "Hosting")
            }
        }
        .padding(.horizontal, Spacing.gutter)
        .padding(.top, Spacing.stackGap)
        .padding(.bottom, Spacing.sectionGap)
    }

    private var agenda: some View {
        VStack(alignment: .leading, spacing: Spacing.sectionGap) {
            if model.isEmptyAfterLoading {
                ContentUnavailableView {
                    Label("Nothing scheduled", systemImage: "calendar")
                } description: {
                    Text("Events you host or join will line up here.")
                }
                .padding(.top, Spacing.sectionGap)
            } else {
                thisWeekSection
                quickActions
                listSection("Hosting", events: model.hosting, filter: .hosting)
                listSection("Attending", events: model.attending, filter: .attending)
            }
        }
        .padding(.horizontal, Spacing.gutter)
        .padding(.top, Spacing.gutter)
        .padding(.bottom, Spacing.tabBarClearance)
    }

    @ViewBuilder
    private var thisWeekSection: some View {
        if !model.thisWeek.isEmpty {
            VStack(alignment: .leading, spacing: Spacing.stackGap) {
                SectionHeader("Upcoming", actionTitle: "See all") {
                    router.push(.feed(.thisWeek))
                }

                ScrollView(.horizontal) {
                    LazyHStack(spacing: Spacing.stackGap) {
                        ForEach(model.thisWeek) { event in
                            Button {
                                router.push(.eventDetail(event.id))
                            } label: {
                                EventHeroCard(
                                    event: event,
                                    goingGuests: event.goingGuests,
                                    guestNames: event.previewGuestNames
                                )
                            }
                            .buttonStyle(.plain)
                            .containerRelativeFrame(.horizontal, count: 5, span: 4, spacing: Spacing.stackGap)
                            .scrollTransition { content, phase in
                                content
                                    .scaleEffect(phase.isIdentity ? 1 : 0.94)
                                    .opacity(phase.isIdentity ? 1 : 0.7)
                            }
                        }
                    }
                    .scrollTargetLayout()
                }
                .scrollTargetBehavior(.viewAligned)
                .scrollIndicators(.hidden)

                if model.thisWeek.count > 1 {
                    PageDots(count: model.thisWeek.count, selection: 0)
                        .frame(maxWidth: .infinity)
                }
            }
        }
    }

    private var quickActions: some View {
        VStack(alignment: .leading, spacing: Spacing.stackGap) {
            SectionHeader("Quick actions")
            HStack(spacing: Spacing.stackGap) {
                QuickActionTile(
                    symbol: "plus",
                    title: "New Event",
                    subtitle: "Start from scratch"
                ) {
                    onCreate()
                }
                QuickActionTile(
                    symbol: "qrcode.viewfinder",
                    title: "Join Event",
                    subtitle: "Scan or enter code",
                    tint: Palette.statusGoing
                ) {
                    onJoin()
                }
            }
            .fixedSize(horizontal: false, vertical: true)
        }
    }

    @ViewBuilder
    private func listSection(_ title: String, events: [Event], filter: FeedFilter) -> some View {
        if !events.isEmpty {
            VStack(alignment: .leading, spacing: Spacing.stackGap) {
                SectionHeader(title, actionTitle: events.count > 3 ? "See all" : nil) {
                    router.push(.feed(filter))
                }
                ForEach(events.prefix(3)) { event in
                    Button {
                        router.push(.eventDetail(event.id))
                    } label: {
                        EventListRow(event: event)
                    }
                    .buttonStyle(.plain)
                }
            }
        }
    }
}
