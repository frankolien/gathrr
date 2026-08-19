import SwiftUI

struct GlassSurface<Outline: InsettableShape>: ViewModifier {
    let outline: Outline

    func body(content: Content) -> some View {
        if #available(iOS 26.0, *) {
            content.glassEffect(.regular, in: outline)
        } else {
            content
                .background(.ultraThinMaterial, in: outline)
                .overlay { outline.strokeBorder(Palette.glassEdge.opacity(0.6), lineWidth: 0.5) }
        }
    }
}

extension View {
    public func glassPanel(radius: CGFloat = Radius.tile) -> some View {
        modifier(GlassSurface(outline: RoundedRectangle(cornerRadius: radius, style: .continuous)))
    }

    public func glassCapsule() -> some View {
        modifier(GlassSurface(outline: Capsule()))
    }
}

public struct ProviderButton: View {
    public enum Emblem {
        case apple
        case google
        case symbol(String)
        case plain
    }

    public enum Tone {
        case solid
        case glass
    }

    public enum Presentation {
        case labelled
        case iconOnly
    }

    private let title: String
    private let emblem: Emblem
    private let tone: Tone
    private let presentation: Presentation
    private let action: () -> Void

    public init(
        _ title: String,
        emblem: Emblem,
        tone: Tone = .glass,
        presentation: Presentation = .labelled,
        action: @escaping () -> Void
    ) {
        self.title = title
        self.emblem = emblem
        self.tone = tone
        self.presentation = presentation
        self.action = action
    }

    public var body: some View {
        Button(action: action) {
            HStack(spacing: Spacing.unit * 2) {
                mark
                if presentation == .labelled {
                    Text(title).font(Typography.headline)
                }
            }
            .foregroundStyle(tone == .solid ? Palette.onProvider : Palette.textPrimary)
            .padding(.horizontal, Spacing.cardPadding)
            .frame(maxWidth: .infinity, minHeight: OnboardingMetrics.providerButtonHeight)
            .modifier(ProviderSurface(tone: tone))
        }
        .buttonStyle(.plain)
        .accessibilityLabel(title)
    }

    @ViewBuilder
    private var mark: some View {
        switch emblem {
        case .apple:
            Image(systemName: "apple.logo")
                .font(.system(size: OnboardingMetrics.providerMarkSize, weight: .medium))
        case .google:
            Image("GoogleMark", bundle: .module)
                .renderingMode(.original)
                .resizable()
                .scaledToFit()
                .frame(
                    width: OnboardingMetrics.providerMarkSize,
                    height: OnboardingMetrics.providerMarkSize
                )
        case .symbol(let name):
            Image(systemName: name)
                .font(.system(size: OnboardingMetrics.providerMarkSize, weight: .medium))
        case .plain:
            EmptyView()
        }
    }
}

private struct ProviderSurface: ViewModifier {
    let tone: ProviderButton.Tone

    func body(content: Content) -> some View {
        switch tone {
        case .solid:
            content.background(
                Palette.providerSurface,
                in: RoundedRectangle(cornerRadius: Radius.tile, style: .continuous)
            )
        case .glass:
            content.glassPanel()
        }
    }
}

public struct OrDivider: View {
    private let label: String

    public init(_ label: String = "or") {
        self.label = label
    }

    public var body: some View {
        Text(label)
            .font(Typography.footnote)
            .foregroundStyle(Palette.textTertiary)
            .frame(maxWidth: .infinity)
            .accessibilityHidden(true)
    }
}
