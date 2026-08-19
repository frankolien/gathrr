import Models
import SwiftUI

public struct CategoryStyle: Sendable {
    public let label: String
    public let symbol: String
    public let tint: Color
}

extension EventCategory {
    public var style: CategoryStyle {
        switch self {
        case .birthday:
            CategoryStyle(label: "Birthday", symbol: "birthday.cake.fill", tint: Palette.adaptive(light: 0xFF_37_5F, dark: 0xFF_37_5F))
        case .party:
            CategoryStyle(label: "Party", symbol: "party.popper.fill", tint: Palette.adaptive(light: 0xAF_52_DE, dark: 0xBF_5A_F2))
        case .meetup:
            CategoryStyle(label: "Meetup", symbol: "person.3.fill", tint: Palette.adaptive(light: 0x0A_84_FF, dark: 0x0A_84_FF))
        case .dinner:
            CategoryStyle(label: "Dinner", symbol: "fork.knife", tint: Palette.adaptive(light: 0xFF_95_00, dark: 0xFF_9F_0A))
        case .gameNight:
            CategoryStyle(label: "Game Night", symbol: "gamecontroller.fill", tint: Palette.adaptive(light: 0x30_D1_58, dark: 0x30_D1_58))
        case .wedding:
            CategoryStyle(label: "Wedding", symbol: "heart.fill", tint: Palette.adaptive(light: 0xFF_2D_55, dark: 0xFF_37_5F))
        case .other:
            CategoryStyle(label: "Event", symbol: "calendar", tint: Palette.textSecondary)
        }
    }
}

