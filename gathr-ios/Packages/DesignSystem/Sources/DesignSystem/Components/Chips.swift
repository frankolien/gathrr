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

