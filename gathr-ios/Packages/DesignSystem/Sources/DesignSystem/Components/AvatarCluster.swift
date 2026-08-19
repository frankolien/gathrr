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

