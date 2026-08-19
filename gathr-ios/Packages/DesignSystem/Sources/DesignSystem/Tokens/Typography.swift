import SwiftUI

public enum Typography {
    private static func rounded(_ size: CGFloat) -> Font {
        .system(size: size, weight: .medium, design: .rounded)
    }

    public static let onboardingHeadline = rounded(30)
    public static let display = rounded(28)
    public static let titleL = rounded(24)
    public static let titleM = rounded(22)
    public static let titleS = rounded(20)
    public static let headline = rounded(17)
    public static let body = rounded(16)
    public static let subhead = rounded(15)
    public static let footnote = rounded(13)
    public static let eyebrow = rounded(12)
    public static let chip = rounded(11)
    public static let numeral = rounded(20).monospacedDigit()
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

