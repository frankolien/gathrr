import DesignSystem
import Models
import Routing
import SwiftUI

public struct EventDetailView: View {
    @State private var model: EventDetailModel
    @State private var isShowingRSVPSheet = false
    private let router: Router

    public init(model: EventDetailModel, router: Router) {
        _model = State(initialValue: model)
        self.router = router
    }

    public var body: some View {
        content
            .background(Palette.canvas)
            .safeAreaInset(edge: .bottom) {
                if model.detail != nil {
                    actionBar
                }
            }
        .task { await model.load() }
        .sheet(isPresented: $isShowingRSVPSheet) {
            RSVPSheet(model: model)
                .presentationDetents([.medium])
                .presentationDragIndicator(.visible)
        }
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                if let detail = model.detail {
                    ShareLink(item: detail.title, subject: Text(detail.title)) {
                        Image(systemName: "square.and.arrow.up")
                            .font(.system(size: 18, weight: .medium))
                            .foregroundStyle(Palette.textPrimary)
                            .frame(width: 44, height: 44)
                            .background(Palette.surface, in: Circle())
                            .overlay { Circle().strokeBorder(Palette.glassEdge.opacity(0.8), lineWidth: 1) }
                    }
                    .accessibilityLabel("Share event")
                }
            }
        }
    }

    @ViewBuilder
    private var content: some View {
        switch model.phase {
        case .idle, .loading:
            ProgressView().frame(maxWidth: .infinity, maxHeight: .infinity)
        case .failed(let message):
            ContentUnavailableView("Couldn't load this event", systemImage: "wifi.exclamationmark", description: Text(message))
        case .loaded:
            if let detail = model.detail {
                loaded(detail)
            }
        }
    }

    private func loaded(_ detail: EventDetail) -> some View {
        ScrollView {
            VStack(alignment: .leading, spacing: Spacing.sectionGap) {
                coverCard(detail)
                guestsSection(detail)
                aboutSection(detail)
            }
            .padding(.horizontal, Spacing.gutter)
            .padding(.bottom, Spacing.stackGap)
        }
    }

    private func coverCard(_ detail: EventDetail) -> some View {
        VStack(alignment: .leading, spacing: Spacing.stackGap) {
            ZStack(alignment: .topLeading) {
                EventCoverImage(category: detail.category)
                    .frame(height: 200)
                    .clipShape(RoundedRectangle(cornerRadius: Radius.image, style: .continuous))
                if detail.category != .birthday {
                    CategoryChip(detail.category).padding(Spacing.stackGap)
                }
            }

            Text(detail.title)
                .font(Typography.titleM)
                .foregroundStyle(Palette.textPrimary)

            metaRow(symbol: "calendar", text: EventFormatting.longWhen(detail.startsAt, timezone: detail.timezone))
            if let location = detail.locationName {
                metaRow(symbol: "mappin.and.ellipse", text: location)
            }

            if detail.observedStatus.acceptsRSVPs {
                EyebrowText("Event starts in")
                CountdownSegments(startsAt: detail.startsAt)
            } else {
                StatusBanner(status: detail.observedStatus)
            }
        }
        .padding(Spacing.cardPadding)
        .background(Palette.surface)
        .clipShape(RoundedRectangle(cornerRadius: Radius.sheet, style: .continuous))
        .cardElevation()
        .padding(.top, Spacing.stackGap)
    }

    private func guestsSection(_ detail: EventDetail) -> some View {
        VStack(alignment: .leading, spacing: Spacing.stackGap) {
            SectionHeader("Guests", actionTitle: "Manage") {
                router.push(.guestList(detail.id))
            }
            VStack(spacing: Spacing.stackGap) {
                AvatarCluster(
                    names: detail.previewGuestNames,
                    goingGuests: detail.goingGuests,
                    limit: 6,
                    arrangement: .staggered
                )
                Text(EventFormatting.goingSummary(goingGuests: detail.goingGuests, hostDisplayName: detail.hostDisplayName))
                    .font(Typography.footnote)
                    .foregroundStyle(Palette.textSecondary)
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, Spacing.sectionGap)
            .padding(.horizontal, Spacing.cardPadding)
            .background(Palette.surface)
            .clipShape(RoundedRectangle(cornerRadius: Radius.sheet, style: .continuous))
        }
    }

    @ViewBuilder
    private func aboutSection(_ detail: EventDetail) -> some View {
        if let description = detail.description, !description.isEmpty {
            VStack(alignment: .leading, spacing: Spacing.stackGap) {
                SectionHeader("About")
                Text(description)
                    .font(Typography.body)
                    .foregroundStyle(Palette.textSecondary)
                    .multilineTextAlignment(.center)
                    .frame(maxWidth: .infinity)
                    .padding(.horizontal, Spacing.gutter)
                    .padding(.vertical, Spacing.sectionGap)
                    .background(Palette.surface)
                    .clipShape(RoundedRectangle(cornerRadius: Radius.sheet, style: .continuous))
            }
        }
    }

    private var actionBar: some View {
        HStack(spacing: Spacing.stackGap) {
            SecondaryButton(model.primaryActionTitle) {
                isShowingRSVPSheet = true
            }
            .disabled(!model.canRSVP)
            PrimaryButton("Chat", shape: .rounded) {}
        }
        .padding(.horizontal, Spacing.gutter)
        .padding(.vertical, Spacing.stackGap)
        .background(Palette.surface)
        .overlay(alignment: .top) { Rectangle().fill(Palette.separator).frame(height: 0.5) }
    }

    private func metaRow(symbol: String, text: String) -> some View {
        HStack(spacing: 6) {
            Image(systemName: symbol).font(.system(size: 13)).foregroundStyle(Palette.textSecondary)
            Text(text).font(Typography.subhead).foregroundStyle(Palette.textSecondary)
        }
    }
}

struct StatusBanner: View {
    let status: EventStatus

    private var copy: (String, Color) {
        switch status {
        case .ongoing: ("Happening now", Palette.statusGoing)
        case .ended: ("This event has ended", Palette.textSecondary)
        case .cancelled: ("This event was cancelled", Palette.statusDeclined)
        default: ("Not published yet", Palette.textSecondary)
        }
    }

    var body: some View {
        Text(copy.0)
            .font(Typography.footnote)
            .foregroundStyle(copy.1)
            .padding(.horizontal, Spacing.stackGap)
            .padding(.vertical, 8)
            .frame(maxWidth: .infinity)
            .background(copy.1.opacity(0.12))
            .clipShape(RoundedRectangle(cornerRadius: Radius.tile, style: .continuous))
    }
}
