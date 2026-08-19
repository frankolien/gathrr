import SwiftUI

struct GlassCapsule: ViewModifier {
    func body(content: Content) -> some View {
        if #available(iOS 26.0, *) {
            content.glassEffect(.regular, in: .capsule)
        } else {
            content
                .background(.ultraThinMaterial, in: Capsule())
                .overlay {
                    Capsule().strokeBorder(Palette.glassEdge.opacity(0.6), lineWidth: 0.5)
                }
        }
    }
}

extension View {
    public func glassCapsule() -> some View {
        modifier(GlassCapsule())
    }
}

public struct ProviderButton: View {
    public enum Emblem {
        case apple
        case google
        case symbol(String)
    }

    public enum Tone {
        case solid
        case glass
    }

    private let title: String
    private let emblem: Emblem
    private let tone: Tone
    private let action: () -> Void

    public init(
        _ title: String,
        emblem: Emblem,
        tone: Tone = .glass,
        action: @escaping () -> Void
    ) {
        self.title = title
        self.emblem = emblem
        self.tone = tone
        self.action = action
    }

    public var body: some View {
        Button(action: action) {
            HStack(spacing: Spacing.stackGap) {
                mark
                Text(title)
                    .font(Typography.titleS)
            }
            .foregroundStyle(tone == .solid ? Palette.onProvider : Palette.textPrimary)
            .padding(.horizontal, Spacing.gutter)
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
                .font(.system(size: 19, weight: .medium))
                .frame(width: 24)
        case .google:
            Image("GoogleMark", bundle: .module)
                .renderingMode(.original)
                .resizable()
                .scaledToFit()
                .frame(width: 20, height: 20)
                .frame(width: 24)
        case .symbol(let name):
            Image(systemName: name)
                .font(.system(size: 17, weight: .medium))
                .frame(width: 24)
        }
    }
}

private struct ProviderSurface: ViewModifier {
    let tone: ProviderButton.Tone

    func body(content: Content) -> some View {
        switch tone {
        case .solid:
            content.background(Palette.providerSurface, in: Capsule())
        case .glass:
            content.glassCapsule()
        }
    }
}
