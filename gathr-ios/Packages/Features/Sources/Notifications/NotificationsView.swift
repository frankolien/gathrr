import DesignSystem
import Models
import Routing
import SwiftUI

public struct NotificationsView: View {
    @State private var model: NotificationsModel
    private let router: Router

    public init(model: NotificationsModel, router: Router) {
        _model = State(initialValue: model)
        self.router = router
    }

    public var body: some View {
        GradientHeaderScreen {
            header
        } content: {
            sheet
        }
        .refreshable { await model.load() }
        .task { await model.load() }
        .navigationBarBackButtonHidden()
    }

    private var header: some View {
        HStack(alignment: .center) {
            Button {
                router.pop()
            } label: {
                Image(systemName: "chevron.left")
                    .font(.system(size: 17, weight: .semibold))
                    .foregroundStyle(Palette.onHeader)
                    .frame(width: 40, height: 40)
                    .background(
                        Palette.headerGlass,
                        in: RoundedRectangle(cornerRadius: Radius.thumb, style: .continuous)
                    )
            }
            .accessibilityLabel("Back")

            VStack(alignment: .leading, spacing: 2) {
                Text("Notifications")
                    .font(Typography.titleL)
                    .foregroundStyle(Palette.onHeader)
                Text(model.unread == 0 ? "You're all caught up" : "\(model.unread) unread")
                    .font(Typography.footnote)
                    .foregroundStyle(Palette.onHeaderMuted)
            }
            .padding(.leading, Spacing.unit)

            Spacer()

            if model.unread > 0 {
                Button {
                    Task { await model.markEverythingRead() }
                } label: {
                    Text("Mark read")
                        .font(Typography.footnote)
                        .foregroundStyle(Palette.onHeader)
                        .padding(.horizontal, Spacing.stackGap)
                        .padding(.vertical, 8)
                        .background(Palette.headerGlass, in: Capsule())
                }
            }
        }
        .padding(.horizontal, Spacing.gutter)
        .padding(.top, Spacing.stackGap)
        .padding(.bottom, Spacing.sectionGap)
    }

    private var sheet: some View {
        VStack(alignment: .leading, spacing: Spacing.sectionGap) {
            if model.isEmptyAfterLoading {
                emptyState
            } else if case .failed(let message) = model.phase, model.entries.isEmpty {
                failureState(message)
            } else {
                ForEach(model.sections()) { section in
                    VStack(alignment: .leading, spacing: Spacing.stackGap) {
                        SectionHeader(title(for: section.bucket))
                        ForEach(section.entries) { entry in
                            Button {
                                router.push(.eventDetail(entry.eventId))
                            } label: {
                                NotificationRow(entry: entry)
                            }
                            .buttonStyle(.plain)
                        }
                    }
                }
            }
        }
        .padding(.horizontal, Spacing.gutter)
        .padding(.top, Spacing.gutter)
        .padding(.bottom, Spacing.tabBarClearance)
    }

    private func title(for bucket: ActivityBucket) -> String {
        switch bucket {
        case .today: String(localized: "Today")
        case .yesterday: String(localized: "Yesterday")
        case .thisWeek: String(localized: "This week")
        case .earlier: String(localized: "Earlier")
        }
    }

    private var emptyState: some View {
        ContentUnavailableView {
            Label("Nothing yet", systemImage: "bell")
        } description: {
            Text("Replies, RSVPs and reminders will land here.")
        }
        .padding(.top, Spacing.sectionGap)
    }

    private func failureState(_ message: String) -> some View {
        ContentUnavailableView {
            Label("Can't load notifications", systemImage: "exclamationmark.triangle")
        } description: {
            Text(message)
        } actions: {
            PrimaryButton("Try again") { Task { await model.load() } }
        }
        .padding(.top, Spacing.sectionGap)
    }
}
