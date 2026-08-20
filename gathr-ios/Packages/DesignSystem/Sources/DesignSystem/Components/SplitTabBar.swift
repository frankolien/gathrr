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

public struct SplitTabBar: View {
    private let items: [TabItem]
    private let trailing: TabItem
    @Binding private var selection: String
    private let trailingAction: () -> Void

    @Namespace private var slot

    public init(
        items: [TabItem],
        trailing: TabItem,
        selection: Binding<String>,
        trailingAction: @escaping () -> Void
    ) {
        self.items = items
        self.trailing = trailing
        _selection = selection
        self.trailingAction = trailingAction
    }

    public var body: some View {
        if #available(iOS 26.0, *) {
            GlassEffectContainer(spacing: Spacing.stackGap) { bar }
        } else {
            bar
        }
    }

    private var bar: some View {
        HStack(spacing: Spacing.stackGap) {
            pill
            trailingCapsule
        }
        .padding(.horizontal, Spacing.gutter)
    }

    private var pill: some View {
        HStack(spacing: 0) {
            ForEach(items) { item in
                Button {
                    withAnimation(.smooth(duration: 0.38, extraBounce: 0.12)) {
                        selection = item.id
                    }
                } label: {
                    cell(item, chosen: selection == item.id)
                        .matchedGeometryEffect(id: item.id, in: slot, isSource: true)
                }
                .buttonStyle(.plain)
                .accessibilityLabel(item.title)
                .accessibilityAddTraits(selection == item.id ? [.isSelected] : [])
            }
        }
        .padding(Spacing.unit)
        .background { lens }
        .modifier(GlassBar())
    }

    private var trailingCapsule: some View {
        Button(action: trailingAction) {
            cell(trailing, chosen: false)
                .frame(width: Spacing.trailingTabWidth)
                .padding(Spacing.unit)
        }
        .buttonStyle(.plain)
        .accessibilityLabel(trailing.title)
        .modifier(GlassBar())
    }

    private func cell(_ item: TabItem, chosen: Bool) -> some View {
        VStack(spacing: 3) {
            Image(systemName: chosen ? item.selectedSymbol : item.symbol)
                .font(.system(size: 17, weight: chosen ? .semibold : .regular))
            Text(item.title)
                .font(Typography.chip)
        }
        .foregroundStyle(chosen ? Palette.accent : Palette.tabIdle)
        .frame(maxWidth: .infinity)
        .frame(height: Spacing.tabBarHeight)
        .contentShape(Rectangle())
    }

    @ViewBuilder
    private var lens: some View {
        if #available(iOS 26.0, *) {
            Capsule()
                .fill(.clear)
                .glassEffect(.regular.interactive(), in: .capsule)
                .matchedGeometryEffect(id: selection, in: slot, isSource: false)
        } else {
            Capsule()
                .fill(Palette.tabLens)
                .matchedGeometryEffect(id: selection, in: slot, isSource: false)
        }
    }
}

private struct GlassBar: ViewModifier {
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
