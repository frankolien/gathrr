import SwiftUI
import UIKit

public struct CoverPalette: Hashable, Sendable {
    public let base: UInt32
    public let accent: UInt32
    public let ink: UInt32

    public init(base: UInt32, accent: UInt32, ink: UInt32 = 0xFF_FF_FF) {
        self.base = base
        self.accent = accent
        self.ink = ink
    }

    public var baseColor: Color { CoverPalette.color(base) }
    public var accentColor: Color { CoverPalette.color(accent) }
    public var inkColor: Color { CoverPalette.color(ink) }

    public var deep: Color { blended(with: 0x00_00_00, amount: 0.52) }
    public var deepest: Color { blended(with: 0x00_00_00, amount: 0.80) }
    public var lit: Color { blended(with: 0xFF_FF_FF, amount: 0.24) }

    public func blended(with target: UInt32, amount: Double) -> Color {
        CoverPalette.color(CoverPalette.mix(base, target, amount))
    }

    public static func color(_ hex: UInt32) -> Color {
        Color(
            red: Double((hex >> 16) & 0xFF) / 255,
            green: Double((hex >> 8) & 0xFF) / 255,
            blue: Double(hex & 0xFF) / 255
        )
    }

    static func mix(_ lhs: UInt32, _ rhs: UInt32, _ amount: Double) -> UInt32 {
        let clamped = min(max(amount, 0), 1)
        var result: UInt32 = 0
        for shift in stride(from: 16, through: 0, by: -8) {
            let left = Double((lhs >> UInt32(shift)) & 0xFF)
            let right = Double((rhs >> UInt32(shift)) & 0xFF)
            let value = UInt32(left + (right - left) * clamped)
            result |= value << UInt32(shift)
        }
        return result
    }
}

extension CoverPalette {
    public init?(averaging image: UIImage) {
        guard let cgImage = image.cgImage else { return nil }
        var pixel = [UInt8](repeating: 0, count: 4)
        let space = CGColorSpaceCreateDeviceRGB()
        let info = CGImageAlphaInfo.premultipliedLast.rawValue
        guard
            let context = CGContext(
                data: &pixel,
                width: 1,
                height: 1,
                bitsPerComponent: 8,
                bytesPerRow: 4,
                space: space,
                bitmapInfo: info
            )
        else { return nil }

        context.draw(cgImage, in: CGRect(x: 0, y: 0, width: 1, height: 1))
        let source = UIColor(
            red: CGFloat(pixel[0]) / 255,
            green: CGFloat(pixel[1]) / 255,
            blue: CGFloat(pixel[2]) / 255,
            alpha: 1
        )
        self.init(base: CoverPalette.vivid(source), accent: 0xFF_FF_FF)
    }

    static func vivid(_ color: UIColor) -> UInt32 {
        var hue: CGFloat = 0
        var saturation: CGFloat = 0
        var brightness: CGFloat = 0
        var alpha: CGFloat = 0
        color.getHue(&hue, saturation: &saturation, brightness: &brightness, alpha: &alpha)
        let boosted = UIColor(
            hue: hue,
            saturation: min(max(saturation * 1.5, 0.34), 0.92),
            brightness: min(max(brightness, 0.42), 0.78),
            alpha: 1
        )
        var red: CGFloat = 0
        var green: CGFloat = 0
        var blue: CGFloat = 0
        boosted.getRed(&red, green: &green, blue: &blue, alpha: &alpha)
        return UInt32(red * 255) << 16 | UInt32(green * 255) << 8 | UInt32(blue * 255)
    }
}
