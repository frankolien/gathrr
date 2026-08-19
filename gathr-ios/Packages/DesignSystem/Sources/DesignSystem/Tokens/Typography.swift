import SwiftUI

public enum Typography {
    private static func rounded(_ size: CGFloat) -> Font {
        .system(size: size, weight: .medium, design: .rounded)
    }

    public static let onboardingHeadline = rounded(27)
    public static let display = rounded(25)
    public static let titleL = rounded(21)
    public static let titleM = rounded(19)
    public static let titleS = rounded(17)
    public static let headline = rounded(15)
    public static let body = rounded(14)
    public static let subhead = rounded(13)
    public static let footnote = rounded(12)
    public static let eyebrow = rounded(11)
    public static let chip = rounded(10)
    public static let numeral = rounded(17).monospacedDigit()
}

public struct EyebrowText: View {
    private let text: String

    public init(_ text: String) {
        self.text = text
    }

    public var body: some View {
        Text(text.uppercased())
            .font(Typography.eyebrow)
            .tracking(0.6)
            .foregroundStyle(Palette.textSecondary)
    }
}

public struct ChipText: View {
    private let text: String

    public init(_ text: String) {
        self.text = text
    }

    public var body: some View {
        Text(text.uppercased())
            .font(Typography.chip)
            .tracking(0.5)
    }
}
