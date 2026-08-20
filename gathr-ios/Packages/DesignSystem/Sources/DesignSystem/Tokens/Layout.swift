import SwiftUI

public enum Spacing {
    public static let unit: CGFloat = 4
    public static let gutter: CGFloat = 16
    public static let cardPadding: CGFloat = 13
    public static let stackGap: CGFloat = 10
    public static let sectionGap: CGFloat = 22
    public static let rowHeight: CGFloat = 56
    public static let minimumTarget: CGFloat = 44
    public static let tabBarClearance: CGFloat = 96
    public static let tabBarHeight: CGFloat = 52
    public static let floatingActionDiameter: CGFloat = 52
    public static let trailingTabWidth: CGFloat = 62
}

public enum Radius {
    public static let hero: CGFloat = 18
    public static let sheet: CGFloat = 24
    public static let card: CGFloat = 16
    public static let tile: CGFloat = 14
    public static let image: CGFloat = 14
    public static let thumb: CGFloat = 9
}

public enum Elevation {
    public static func card() -> some ViewModifier { Shadow(y: 4, blur: 16, opacity: 0.06) }
    public static func raised() -> some ViewModifier { Shadow(y: 8, blur: 24, opacity: 0.10) }

    struct Shadow: ViewModifier {
        let y: CGFloat
        let blur: CGFloat
        let opacity: Double

        func body(content: Content) -> some View {
            content.shadow(color: .black.opacity(opacity), radius: blur / 2, x: 0, y: y)
        }
    }
}

extension View {
    public func cardElevation() -> some View {
        modifier(Elevation.card())
    }

    public func raisedElevation() -> some View {
        modifier(Elevation.raised())
    }

    public func minimumHitTarget() -> some View {
        frame(minWidth: Spacing.minimumTarget, minHeight: Spacing.minimumTarget)
    }
}
