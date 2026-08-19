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

public struct HeaderStat: View {
    private let value: Int
    private let label: String

    public init(value: Int, label: String) {
        self.value = value
        self.label = label
    }

    public var body: some View {
        VStack(spacing: 2) {
            Text("\(value)")
                .font(Typography.titleM)
                .monospacedDigit()
                .foregroundStyle(Palette.onHeader)
            Text(label)
                .font(Typography.footnote)
                .foregroundStyle(Palette.onHeaderMuted)
        }
        .frame(maxWidth: .infinity)
        .accessibilityElement(children: .combine)
        .accessibilityLabel("\(value) \(label)")
    }
}

public struct ActivePill: View {
    private let title: String

    public init(_ title: String) {
        self.title = title
    }

    public var body: some View {
        HStack(spacing: 6) {
            Circle()
                .fill(Palette.statusGoing)
                .frame(width: 7, height: 7)
            Text(title)
                .font(Typography.footnote)
                .foregroundStyle(Palette.onHeader)
        }
        .padding(.horizontal, Spacing.stackGap)
        .padding(.vertical, 7)
        .background(Palette.headerGlass, in: Capsule())
    }
}
