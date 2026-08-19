import SwiftUI

public struct CountdownSegments: View {
    private let startsAt: Date

    public init(startsAt: Date) {
        self.startsAt = startsAt
    }

    public var body: some View {
        TimelineView(.periodic(from: .now, by: 60)) { context in
            let parts = CountdownParts(until: startsAt, from: context.date)
            HStack(spacing: 0) {
                segment(value: parts.days, unit: "Days", focused: true)
                segment(value: parts.hours, unit: "Hours", focused: false)
                segment(value: parts.minutes, unit: "Mins", focused: false)
            }
            .background(Palette.surfaceInset)
            .clipShape(RoundedRectangle(cornerRadius: Radius.tile, style: .continuous))
            .accessibilityElement(children: .ignore)
            .accessibilityLabel(accessibilityPhrase(parts))
        }
    }

    private func segment(value: Int, unit: String, focused: Bool) -> some View {
        VStack(spacing: 2) {
            Text(String(format: "%02d", value))
                .font(Typography.numeral)
                .foregroundStyle(Palette.textPrimary)
            Text(unit)
                .font(Typography.footnote)
                .foregroundStyle(Palette.textSecondary)
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 12)
        .background(focused ? Palette.surfaceInsetActive : Color.clear)
        .clipShape(RoundedRectangle(cornerRadius: Radius.tile, style: .continuous))
    }

    private func accessibilityPhrase(_ parts: CountdownParts) -> String {
        if parts.hasStarted { return "This event is happening now" }
        return "Event starts in \(parts.days) days, \(parts.hours) hours, \(parts.minutes) minutes"
    }
}
