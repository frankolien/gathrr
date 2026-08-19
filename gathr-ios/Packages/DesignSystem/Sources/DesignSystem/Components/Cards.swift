import Models
import SwiftUI

public struct CoverArt: View {
    private let category: EventCategory

    public init(category: EventCategory) {
        self.category = category
    }

    public var body: some View {
        LinearGradient(
            colors: [category.style.tint, category.style.tint.opacity(0.55)],
            startPoint: .topLeading,
            endPoint: .bottomTrailing
        )
    }
}

public struct EventHeroCard: View {
    private let event: Event
    private let goingGuests: Int
    private let guestNames: [String]
    private let onOverflow: (() -> Void)?

    public init(
        event: Event,
        goingGuests: Int,
        guestNames: [String],
        onOverflow: (() -> Void)? = nil
    ) {
        self.event = event
        self.goingGuests = goingGuests
        self.guestNames = guestNames
        self.onOverflow = onOverflow
    }

    public var body: some View {
        ZStack(alignment: .bottomLeading) {
            CoverArt(category: event.category)
            Palette.photoScrim

            VStack(alignment: .leading, spacing: 6) {
                Text(event.title)
                    .font(Typography.titleL)
                    .foregroundStyle(Palette.onPhoto)
                    .lineLimit(2)
                meta(symbol: "calendar", text: EventFormatting.longWhen(event.startsAt, timezone: event.timezone))
                if let location = event.locationName {
                    meta(symbol: "mappin.and.ellipse", text: location)
                }
                HStack(alignment: .center) {
                    AvatarCluster(names: guestNames, goingGuests: goingGuests, ringColor: .white.opacity(0.35))
                    if let overflow = EventFormatting.clusterOverflow(goingGuests: goingGuests, shown: min(guestNames.count, 4)) {
                        Text(overflow)
                            .font(Typography.footnote)
                            .foregroundStyle(Palette.onPhoto)
                    }
                    Spacer()
                    CountdownPill(startsAt: event.startsAt)
                }
                .padding(.top, 6)
            }
            .padding(Spacing.cardPadding)

            VStack {
                HStack {
                    CategoryChip(event.category)
                    Spacer()
                    if let onOverflow {
                        Button(action: onOverflow) {
                            Image(systemName: "ellipsis")
                                .font(.system(size: 15, weight: .semibold))
                                .foregroundStyle(Palette.onPhoto)
                                .minimumHitTarget()
                        }
                        .accessibilityLabel("More options")
                    }
                }
                Spacer()
            }
            .padding(Spacing.cardPadding)
        }
        .aspectRatio(4 / 3, contentMode: .fill)
        .clipShape(RoundedRectangle(cornerRadius: Radius.hero, style: .continuous))
        .cardElevation()
        .accessibilityElement(children: .combine)
    }

    private func meta(symbol: String, text: String) -> some View {
        HStack(spacing: 6) {
            Image(systemName: symbol).font(.system(size: 12))
            Text(text).font(Typography.footnote)
        }
        .foregroundStyle(Palette.onPhoto.opacity(0.9))
    }
}

public struct EventListRow: View {
    private let event: Event

    public init(event: Event) {
        self.event = event
    }

    public var body: some View {
        HStack(spacing: Spacing.stackGap) {
            CoverArt(category: event.category)
                .frame(width: 44, height: 44)
                .clipShape(RoundedRectangle(cornerRadius: Radius.thumb, style: .continuous))

            VStack(alignment: .leading, spacing: 2) {
                Text(event.title)
                    .font(Typography.headline)
                    .foregroundStyle(Palette.textPrimary)
                    .lineLimit(1)
                Text(EventFormatting.shortWhen(event.startsAt, timezone: event.timezone))
                    .font(Typography.footnote)
                    .foregroundStyle(Palette.textSecondary)
                    .lineLimit(1)
            }

            Spacer(minLength: Spacing.unit)

            Image(systemName: "chevron.right")
                .font(.system(size: 13, weight: .semibold))
                .foregroundStyle(Palette.textTertiary)
        }
        .padding(Spacing.cardPadding)
        .glassPanel(radius: Radius.tile)
        .accessibilityElement(children: .combine)
    }
}
