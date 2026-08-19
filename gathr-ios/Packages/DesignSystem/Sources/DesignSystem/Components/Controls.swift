import SwiftUI

public struct SectionHeader: View {
    private let title: String
    private let actionTitle: String?
    private let action: (() -> Void)?

    public init(_ title: String, actionTitle: String? = nil, action: (() -> Void)? = nil) {
        self.title = title
        self.actionTitle = actionTitle
        self.action = action
    }

    public var body: some View {
        HStack(alignment: .firstTextBaseline) {
            Text(title)
                .font(Typography.titleS)
                .foregroundStyle(Palette.textPrimary)
            Spacer()
            if let actionTitle, let action {
                Button(actionTitle, action: action)
                    .font(Typography.subhead)
                    .foregroundStyle(Palette.accent)
            }
        }
    }
}

public struct QuickActionTile: View {
    private let symbol: String
    private let title: String
    private let subtitle: String
    private let tint: Color
    private let action: () -> Void

    public init(
        symbol: String,
        title: String,
        subtitle: String,
        tint: Color = Palette.accent,
        action: @escaping () -> Void
    ) {
        self.symbol = symbol
        self.title = title
        self.subtitle = subtitle
        self.tint = tint
        self.action = action
    }

    public var body: some View {
        Button(action: action) {
            VStack(alignment: .leading, spacing: 8) {
                Image(systemName: symbol)
                    .font(.system(size: 20, weight: .semibold))
                    .foregroundStyle(tint)
                VStack(alignment: .leading, spacing: 2) {
                    Text(title)
                        .font(Typography.headline)
                        .foregroundStyle(Palette.textPrimary)
                    Text(subtitle)
                        .font(Typography.footnote)
                        .foregroundStyle(Palette.textSecondary)
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(Spacing.cardPadding)
            .glassPanel(radius: Radius.tile)
        }
        .buttonStyle(.plain)
        .accessibilityElement(children: .combine)
        .accessibilityHint(subtitle)
    }
}

public struct PrimaryButton: View {
    public enum Shape { case pill, rounded }

    private let title: String
    private let shape: Shape
    private let isEnabled: Bool
    private let action: () -> Void

    public init(
        _ title: String,
        shape: Shape = .pill,
        isEnabled: Bool = true,
        action: @escaping () -> Void
    ) {
        self.title = title
        self.shape = shape
        self.isEnabled = isEnabled
        self.action = action
    }

    public var body: some View {
        Button(action: action) {
            Text(title)
                .font(Typography.headline)
                .foregroundStyle(Palette.onAccent)
                .frame(maxWidth: .infinity, minHeight: 56)
                .background(isEnabled ? Palette.accent : Palette.textTertiary)
                .clipShape(RoundedRectangle(cornerRadius: shape == .pill ? 28 : Radius.tile, style: .continuous))
        }
        .buttonStyle(.plain)
        .disabled(!isEnabled)
    }
}

public struct SecondaryButton: View {
    private let title: String
    private let action: () -> Void

    public init(_ title: String, action: @escaping () -> Void) {
        self.title = title
        self.action = action
    }

    public var body: some View {
        Button(action: action) {
            Text(title)
                .font(Typography.headline)
                .foregroundStyle(Palette.textPrimary)
                .frame(maxWidth: .infinity, minHeight: 56)
                .background(Palette.surfaceInset)
                .clipShape(RoundedRectangle(cornerRadius: Radius.tile, style: .continuous))
        }
        .buttonStyle(.plain)
    }
}

public struct PageDots: View {
    private let count: Int
    private let selection: Int

    public init(count: Int, selection: Int) {
        self.count = count
        self.selection = selection
    }

    public var body: some View {
        HStack(spacing: 6) {
            ForEach(0..<max(count, 1), id: \.self) { index in
                Capsule()
                    .fill(index == selection ? Palette.accent : Palette.textTertiary)
                    .frame(width: index == selection ? 18 : 6, height: 6)
            }
        }
        .accessibilityElement(children: .ignore)
        .accessibilityLabel("Page \(selection + 1) of \(count)")
    }
}
