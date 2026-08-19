import SwiftUI

public struct TabItem: Identifiable, Hashable, Sendable {
    public let id: String
    public let symbol: String
    public let selectedSymbol: String
    public let title: String

    public init(id: String, symbol: String, selectedSymbol: String, title: String) {
        self.id = id
        self.symbol = symbol
        self.selectedSymbol = selectedSymbol
        self.title = title
    }
}

public struct LiquidTabBar: View {
    private let items: [TabItem]
    @Binding private var selection: String
    private let action: () -> Void

    @Namespace private var slot
    @Namespace private var glass

    public init(items: [TabItem], selection: Binding<String>, action: @escaping () -> Void) {
        self.items = items
        _selection = selection
        self.action = action
    }

    public var body: some View {
        if #available(iOS 26.0, *) {
            GlassEffectContainer(spacing: Spacing.gutter) { bar }
        } else {
            bar
        }
    }

    private var bar: some View {
        HStack(spacing: Spacing.stackGap) {
            pill
            composeButton
        }
        .padding(.horizontal, Spacing.gutter)
    }

    private var pill: some View {
        HStack(spacing: 0) {
            ForEach(items) { item in
                Button {
                    withAnimation(.bouncy(duration: 0.5, extraBounce: 0.25)) {
                        selection = item.id
                    }
                } label: {
                    tabLabel(item)
                }
                .buttonStyle(.plain)
                .accessibilityLabel(item.title)
                .accessibilityAddTraits(selection == item.id ? [.isSelected] : [])
            }
        }
        .frame(maxWidth: .infinity)
        .background { lens }
        .modifier(BarSurface())
    }

    private func tabLabel(_ item: TabItem) -> some View {
        let chosen = selection == item.id
        return Image(systemName: chosen ? item.selectedSymbol : item.symbol)
            .font(.system(size: 18, weight: chosen ? .semibold : .regular))
            .foregroundStyle(chosen ? Palette.accent : Palette.textTertiary)
            .frame(maxWidth: .infinity)
            .frame(height: Spacing.tabBarHeight)
            .contentShape(Rectangle())
            .matchedGeometryEffect(id: item.id, in: slot, isSource: true)
    }

    @ViewBuilder
    private var lens: some View {
        if #available(iOS 26.0, *) {
            Capsule()
                .fill(.clear)
                .glassEffect(.regular.interactive(), in: .capsule)
                .glassEffectID("lens", in: glass)
                .matchedGeometryEffect(id: selection, in: slot, isSource: false)
        } else {
            Capsule()
                .fill(Palette.accent.opacity(0.14))
                .matchedGeometryEffect(id: selection, in: slot, isSource: false)
        }
    }

    private var composeButton: some View {
        Button(action: action) {
            Image(systemName: "plus")
                .font(.system(size: 20, weight: .semibold))
                .foregroundStyle(Palette.onAccent)
                .frame(
                    width: Spacing.floatingActionDiameter,
                    height: Spacing.floatingActionDiameter
                )
                .background(Palette.accent, in: Circle())
        }
        .buttonStyle(.plain)
        .accessibilityLabel("New event")
    }
}

private struct BarSurface: ViewModifier {
    func body(content: Content) -> some View {
        if #available(iOS 26.0, *) {
            content.glassEffect(.regular, in: .capsule)
        } else {
            content
                .background(.ultraThinMaterial, in: Capsule())
                .overlay { Capsule().strokeBorder(Palette.glassEdge.opacity(0.6), lineWidth: 0.5) }
        }
    }
}
