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
        ScrollView {
            VStack(alignment: .leading, spacing: Spacing.sectionGap) {
                greeting
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
            .padding(.top, Spacing.stackGap)
            .padding(.bottom, Spacing.tabBarClearance)
        }
        .scrollIndicators(.hidden)
        .background(Palette.canvas.ignoresSafeArea())
        .refreshable { await model.load() }
        .task { await model.load() }
        .overlay(alignment: .top) { staleBanner }
    }

    private var greeting: some View {
        HStack(alignment: .center) {
            VStack(alignment: .leading, spacing: 1) {
                EyebrowText(model.greeting())
                Text(accountName)
                    .font(Typography.titleM)
                    .foregroundStyle(Palette.textPrimary)
            }
            Spacer()
            Button {
                router.push(.notifications)
            } label: {
                Avatar(name: accountName)
                    .overlay(alignment: .topTrailing) {
                        if model.hasAnyContent {
                            Circle()
                                .fill(Palette.statusDeclined)
                                .frame(width: 9, height: 9)
                                .overlay { Circle().strokeBorder(Palette.canvas, lineWidth: 1.5) }
                                .offset(x: 1, y: -1)
                        }
                    }
            }
            .accessibilityLabel("Notifications")
        }
    }

    @ViewBuilder
    private var thisWeekSection: some View {
        if !model.thisWeek.isEmpty {
            VStack(alignment: .leading, spacing: Spacing.stackGap) {
                SectionHeader("This week", actionTitle: "See all") {
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
                .padding(.vertical, 6)
                .background(Palette.surfaceInset, in: Capsule())
                .padding(.top, Spacing.unit)
        }
    }
}
