import DesignSystem
import Home
import Models
import Routing
import SwiftUI

public struct CalendarView: View {
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
                section("Upcoming", events: model.thisWeek, filter: .thisWeek)
                section("Hosting", events: model.hosting, filter: .hosting)
                section("Attending", events: model.attending, filter: .attending)
            }
        }
        .padding(.horizontal, Spacing.gutter)
        .padding(.top, Spacing.gutter)
        .padding(.bottom, Spacing.tabBarClearance)
    }

    @ViewBuilder
    private func section(_ title: String, events: [Event], filter: FeedFilter) -> some View {
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
