import Models
import SwiftUI

public struct CategoryChip: View {
    public enum Treatment { case glass, tinted }

    private let category: EventCategory
    private let treatment: Treatment

    public init(_ category: EventCategory, treatment: Treatment = .glass) {
        self.category = category
        self.treatment = treatment
    }

    public var body: some View {
        let style = category.style
        HStack(spacing: 5) {
            Image(systemName: style.symbol).font(.system(size: 10, weight: .semibold))
            ChipText(style.label)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .foregroundStyle(treatment == .glass ? Palette.onPhoto : style.tint)
        .background(treatment == .glass ? AnyShapeStyle(.ultraThinMaterial) : AnyShapeStyle(style.tint.opacity(0.12)))
        .clipShape(Capsule())
        .accessibilityElement(children: .ignore)
        .accessibilityLabel(style.label)
    }
}

public struct RoleBadge: View {
    private let title: String

    public init(_ title: String) {
        self.title = title
    }

    public var body: some View {
        HStack(spacing: 5) {
            Image(systemName: "crown.fill").font(.system(size: 10, weight: .semibold))
            Text(title).font(Typography.chip)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .foregroundStyle(Palette.onPhoto)
        .background(.ultraThinMaterial)
        .clipShape(Capsule())
    }
}

public struct CountdownPill: View {
    private let startsAt: Date
    private let onPhoto: Bool

    public init(startsAt: Date, onPhoto: Bool = true) {
        self.startsAt = startsAt
        self.onPhoto = onPhoto
    }

    public var body: some View {
        TimelineView(.periodic(from: .now, by: 60)) { context in
            Text(EventFormatting.countdownPhrase(until: startsAt, from: context.date))
                .font(Typography.footnote)
                .monospacedDigit()
                .padding(.horizontal, 12)
                .padding(.vertical, 7)
                .foregroundStyle(onPhoto ? Palette.textPrimary : Palette.textSecondary)
                .background(onPhoto ? AnyShapeStyle(Palette.pillOnPhoto) : AnyShapeStyle(Palette.surfaceInset))
                .clipShape(Capsule())
        }
    }
}

public struct StatusChip: View {
    private let status: RSVPStatus
    private let plusOnes: Int

    public init(status: RSVPStatus, plusOnes: Int = 0) {
        self.status = status
        self.plusOnes = plusOnes
    }

    public var body: some View {
        HStack(spacing: 5) {
            Image(systemName: status.symbol).font(.system(size: 11, weight: .semibold))
            Text(plusOnes > 0 ? "\(status.label) · +\(plusOnes)" : status.label)
                .font(Typography.footnote)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .foregroundStyle(status.tint)
        .background(status.tint.opacity(0.12))
        .clipShape(Capsule())
    }
}
