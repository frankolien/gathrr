import SwiftUI

public enum Palette {
    public static let canvas = adaptive(light: 0xF2_F3_F5, dark: 0x0B_0B_0C)
    public static let surface = adaptive(light: 0xFF_FF_FF, dark: 0x1C_1C_1E)
    public static let surfaceInset = adaptive(light: 0xF0_F1_F3, dark: 0x2C_2C_2E)
    public static let surfaceInsetActive = adaptive(light: 0xE7_E8_EB, dark: 0x3A_3A_3C)

    public static let textPrimary = adaptive(light: 0x11_12_14, dark: 0xFF_FF_FF)
    public static let textSecondary = adaptive(light: 0x8E_8E_93, dark: 0x98_98_9F)
    public static let textTertiary = adaptive(light: 0xB0_B0_B5, dark: 0x6C_6C_70)

    public static let accent = adaptive(light: 0x00_7A_FF, dark: 0x0A_84_FF)
    public static let accentPressed = adaptive(light: 0x00_62_CC, dark: 0x33_95_FF)
    public static let onAccent = Color.white
    public static let separator = adaptive(light: 0xE5_E5_EA, dark: 0x38_38_3A)
    public static let onPhoto = Color.white

    public static let statusGoing = adaptive(light: 0x34_C7_59, dark: 0x30_D1_58)
    public static let statusMaybe = adaptive(light: 0xFF_9F_0A, dark: 0xFF_9F_0A)
    public static let statusDeclined = adaptive(light: 0xFF_3B_30, dark: 0xFF_45_3A)
    public static let statusWaitlisted = adaptive(light: 0xAF_52_DE, dark: 0xBF_5A_F2)
    public static let statusInvited = adaptive(light: 0x8E_8E_93, dark: 0x98_98_9F)

    public static let glassChip = Color(white: 0.11, opacity: 0.45)
    public static let pillOnPhoto = Color.white.opacity(0.92)

    private static let veil = adaptive(light: 0xFF_FF_FF, dark: 0x0B_0B_0C)

    public static let onboardingCanvas = adaptive(light: 0xEF_F2_F7, dark: 0x0B_0B_0C)
    public static let glassEdge = adaptive(light: 0xFF_FF_FF, dark: 0x3A_3A_3C)
    public static let onHeader = Color.white
    public static let onHeaderMuted = Color.white.opacity(0.85)
    public static let headerGlass = Color.white.opacity(0.18)
    public static let tabIdle = adaptive(light: 0x8E_8E_93, dark: 0x8E_8E_93)
    public static let tabLens = adaptive(light: 0xFF_FF_FF, dark: 0x3A_3A_3C)

    public static let headerGradient = LinearGradient(
        stops: [
            .init(color: adaptive(light: 0x00_89_FF, dark: 0x0A_4E_9E), location: 0.00),
            .init(color: adaptive(light: 0x4E_BC_FF, dark: 0x10_36_6B), location: 0.44),
            .init(color: adaptive(light: 0xDD_F3_FF, dark: 0x0B_0B_0C), location: 1.00),
        ],
        startPoint: .top,
        endPoint: .bottom
    )
    public static let providerInk = adaptive(light: 0x11_12_14, dark: 0x11_12_14)
    public static let providerSurface = adaptive(light: 0x11_12_14, dark: 0xFF_FF_FF)
    public static let onProvider = adaptive(light: 0xFF_FF_FF, dark: 0x11_12_14)

    public static let photoVeil = LinearGradient(
        stops: [
            .init(color: veil.opacity(0.30), location: 0.00),
            .init(color: veil.opacity(0.00), location: 0.14),
            .init(color: veil.opacity(0.00), location: 0.44),
            .init(color: veil.opacity(0.45), location: 0.58),
            .init(color: veil.opacity(0.90), location: 0.70),
            .init(color: veil, location: 0.80),
            .init(color: veil, location: 1.00),
        ],
        startPoint: .top,
        endPoint: .bottom
    )

    public static let photoClarityMask = LinearGradient(
        stops: [
            .init(color: .clear, location: 0.00),
            .init(color: .white, location: 0.10),
            .init(color: .white, location: 0.46),
            .init(color: .clear, location: 0.66),
        ],
        startPoint: .top,
        endPoint: .bottom
    )

    public static let photoScrim = LinearGradient(
        stops: [
            .init(color: .black.opacity(0), location: 0.45),
            .init(color: .black.opacity(0.60), location: 1.0),
        ],
        startPoint: .top,
        endPoint: .bottom
    )

    static func adaptive(light: UInt32, dark: UInt32) -> Color {
        Color(uiColor: UIColor { traits in
            traits.userInterfaceStyle == .dark ? UIColor(hex: dark) : UIColor(hex: light)
        })
    }
}

extension UIColor {
    convenience init(hex: UInt32) {
        self.init(
            red: CGFloat((hex >> 16) & 0xFF) / 255,
            green: CGFloat((hex >> 8) & 0xFF) / 255,
            blue: CGFloat(hex & 0xFF) / 255,
            alpha: 1
        )
    }
}
