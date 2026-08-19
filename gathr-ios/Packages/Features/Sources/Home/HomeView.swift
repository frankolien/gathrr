import DesignSystem
import Models
import Routing
import SwiftUI

public struct HomeView: View {
    @State private var model: HomeModel
    private let router: Router
    private let accountName: String

    public init(model: HomeModel, router: Router, accountName: String) {
        _model = State(initialValue: model)
        self.router = router
        self.accountName = accountName
    }

    public var body: some View {
        GradientHeaderScreen {
            header
        } content: {
            sheet
        }
        .refreshable { await model.load() }
        .task { await model.load() }
        .overlay(alignment: .top) { staleBanner }
    }

    private var sheet: some View {
        VStack(alignment: .leading, spacing: Spacing.sectionGap) {
            if model.isEmptyAfterLoading {
                emptyState
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

    private var header: some View {
        VStack(alignment: .leading, spacing: Spacing.gutter) {
            HStack(alignment: .center) {
                VStack(alignment: .leading, spacing: 2) {
                    Text(model.greeting().uppercased())
                        .font(Typography.eyebrow)
                        .tracking(0.6)
                        .foregroundStyle(Palette.onHeaderMuted)
                    Text(accountName)
                        .font(Typography.titleL)
                        .foregroundStyle(Palette.onHeader)
                }
                Spacer()
                Button {
                    router.push(.notifications)
                } label: {
                    Image(systemName: "bell")
                        .font(.system(size: 17, weight: .medium))
                        .foregroundStyle(Palette.onHeader)
                        .frame(width: 40, height: 40)
                        .background(Palette.headerGlass, in: RoundedRectangle(cornerRadius: Radius.thumb, style: .continuous))
                }
                .accessibilityLabel("Notifications")
            }

            WeekStrip(days: model.week())

            HStack(alignment: .center) {
                HStack(alignment: .firstTextBaseline, spacing: 6) {
                    Text("\(model.plannedCount())")
                        .font(.system(size: 46, weight: .medium, design: .rounded))
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
                            .containerRelativeFrame(.horizontal, count: 1, spacing: Spacing.stackGap)
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
                    router.push(.createEvent)
                }
                QuickActionTile(
                    symbol: "qrcode.viewfinder",
                    title: "Join Event",
                    subtitle: "Scan or enter code",
                    tint: Palette.statusGoing
                ) {
                    router.push(.joinEvent)
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

    private var emptyState: some View {
        ContentUnavailableView {
            Label("No events yet", systemImage: "calendar.badge.plus")
        } description: {
            Text("Create your first invite, or join one with a code.")
        } actions: {
            PrimaryButton("New Event") { router.push(.createEvent) }
            SecondaryButton("Join Event") { router.push(.joinEvent) }
        }
        .padding(.top, Spacing.sectionGap)
    }

    @ViewBuilder
    private var staleBanner: some View {
        if model.isShowingStaleContent, case .failed(let message) = model.phase {
            Text(message)
                .font(Typography.footnote)
                .foregroundStyle(Palette.textSecondary)
                .padding(.horizontal, Spacing.stackGap)
                .padding(.vertical, 8)
                .background(Palette.surfaceInset)
                .clipShape(Capsule())
                .padding(.top, Spacing.unit)
        }
    }
}
