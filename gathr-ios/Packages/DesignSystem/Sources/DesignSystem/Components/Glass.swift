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

