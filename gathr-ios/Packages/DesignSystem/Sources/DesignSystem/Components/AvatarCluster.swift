import SwiftUI

public struct AvatarCluster: View {
    public enum Arrangement { case linear, staggered }

    private let names: [String]
    private let goingGuests: Int
    private let limit: Int
    private let arrangement: Arrangement
    private let ringColor: Color

    public init(
        names: [String],
        goingGuests: Int,
        limit: Int = 4,
        arrangement: Arrangement = .linear,
        ringColor: Color = Palette.surface
    ) {
        self.names = names
        self.goingGuests = goingGuests
        self.limit = limit
        self.arrangement = arrangement
        self.ringColor = ringColor
    }

    private var shown: [String] { Array(names.prefix(limit)) }

    public var body: some View {
        Group {
            switch arrangement {
            case .linear:
                HStack(spacing: -8) {
                    ForEach(Array(shown.enumerated()), id: \.offset) { _, name in
                        Avatar(name: name, ringColor: ringColor)
                    }
                }
            case .staggered:
                VStack(spacing: -6) {
                    HStack(spacing: -8) {
                        ForEach(Array(shown.prefix(3).enumerated()), id: \.offset) { _, name in
                            Avatar(name: name, ringColor: ringColor)
                        }
                    }
                    HStack(spacing: -8) {
                        ForEach(Array(shown.dropFirst(3).enumerated()), id: \.offset) { _, name in
                            Avatar(name: name, ringColor: ringColor)
                        }
                    }
                }
            }
        }
        .accessibilityElement(children: .ignore)
        .accessibilityLabel("\(goingGuests) people going")
    }
}

public struct Avatar: View {
    let name: String
    let ringColor: Color

    public init(name: String, ringColor: Color = Palette.surface) {
        self.name = name
        self.ringColor = ringColor
    }

    private var initials: String {
        let parts = name.split(separator: " ").prefix(2)
        let letters = parts.compactMap { $0.first }.map(String.init)
        return letters.isEmpty ? "?" : letters.joined().uppercased()
    }

    private var tint: Color {
        let palette: [Color] = [
            Palette.adaptive(light: 0xFF_9F_0A, dark: 0xFF_9F_0A),
            Palette.adaptive(light: 0x0A_84_FF, dark: 0x0A_84_FF),
            Palette.adaptive(light: 0xAF_52_DE, dark: 0xBF_5A_F2),
            Palette.adaptive(light: 0x30_D1_58, dark: 0x30_D1_58),
            Palette.adaptive(light: 0xFF_37_5F, dark: 0xFF_37_5F),
        ]
        return palette[abs(name.hashValue) % palette.count]
    }

    public var body: some View {
        Text(initials)
            .font(.system(size: 12, weight: .semibold))
            .foregroundStyle(.white)
            .frame(width: 32, height: 32)
            .background(tint)
            .clipShape(Circle())
            .overlay(Circle().strokeBorder(ringColor, lineWidth: 2))
    }
}
