import SwiftUI

public struct WeekStripDay: Identifiable, Hashable, Sendable {
    public let id: Date
    public let symbol: String
    public let number: Int
    public let isToday: Bool
    public let hasEvent: Bool

    public init(id: Date, symbol: String, number: Int, isToday: Bool, hasEvent: Bool) {
        self.id = id
        self.symbol = symbol
        self.number = number
        self.isToday = isToday
        self.hasEvent = hasEvent
    }
}

public struct WeekStrip: View {
    private let days: [WeekStripDay]

    public init(days: [WeekStripDay]) {
        self.days = days
    }

    public var body: some View {
        HStack(spacing: 0) {
            ForEach(days) { day in
                VStack(spacing: 6) {
                    Text(day.symbol)
                        .font(Typography.chip)
                        .foregroundStyle(Palette.onHeaderMuted)
                    Text("\(day.number)")
                        .font(Typography.subhead)
                        .monospacedDigit()
                        .foregroundStyle(day.isToday ? Palette.accent : Palette.onHeader)
                        .frame(width: 32, height: 32)
                        .background {
                            if day.isToday {
                                Circle().fill(Palette.onHeader)
                            }
                        }
                    Circle()
                        .fill(day.hasEvent ? Palette.onHeader : .clear)
                        .frame(width: 4, height: 4)
                }
                .frame(maxWidth: .infinity)
                .accessibilityElement(children: .ignore)
                .accessibilityLabel(dayLabel(day))
            }
        }
    }

    private func dayLabel(_ day: WeekStripDay) -> String {
        var parts = ["\(day.symbol) \(day.number)"]
        if day.isToday { parts.append("today") }
        if day.hasEvent { parts.append("has an event") }
        return parts.joined(separator: ", ")
    }
}

