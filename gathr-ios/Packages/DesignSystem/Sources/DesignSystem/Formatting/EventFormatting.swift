import Foundation

public struct CountdownParts: Equatable, Sendable {
    public let days: Int
    public let hours: Int
    public let minutes: Int
    public let hasStarted: Bool

    public init(until target: Date, from now: Date) {
        let remaining = target.timeIntervalSince(now)
        hasStarted = remaining <= 0
        let clamped = Int(max(0, remaining))
        days = clamped / 86_400
        hours = (clamped % 86_400) / 3_600
        minutes = (clamped % 3_600) / 60
    }
}

public enum EventFormatting {
    public static func longWhen(
        _ date: Date,
        timezone: String,
        locale: Locale = .current
    ) -> String {
        var day = Date.FormatStyle.dateTime.weekday(.abbreviated).month(.abbreviated).day()
        var clock = Date.FormatStyle.dateTime.hour(.defaultDigits(amPM: .abbreviated)).minute()
        let zone = TimeZone(identifier: timezone) ?? .gmt

        day.timeZone = zone
        day.locale = locale
        clock.timeZone = zone
        clock.locale = locale

        return "\(date.formatted(day)) · \(date.formatted(clock))"
    }

    public static func shortWhen(
        _ date: Date,
        timezone: String,
        locale: Locale = .current
    ) -> String {
        longWhen(date, timezone: timezone, locale: locale)
    }

    public static func countdownPhrase(until target: Date, from now: Date = .now) -> String {
        let parts = CountdownParts(until: target, from: now)
        if parts.hasStarted { return "Happening now" }
        if parts.days > 0 { return "In \(parts.days) \(parts.days == 1 ? "day" : "days")" }
        if parts.hours > 0 { return "In \(parts.hours) \(parts.hours == 1 ? "hour" : "hours")" }
        if parts.minutes > 0 { return "In \(parts.minutes) min" }
        return "Starting now"
    }

    public static func goingSummary(goingGuests: Int, hostDisplayName: String) -> String {
        "\(goingGuests) going · hosted by \(hostDisplayName)"
    }

    public static func clusterOverflow(goingGuests: Int, shown: Int) -> String? {
        let remaining = goingGuests - shown
        return remaining > 0 ? "+\(remaining) going" : nil
    }

    public static func greeting(at date: Date, timezone: String) -> String {
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = TimeZone(identifier: timezone) ?? .gmt
        return switch calendar.component(.hour, from: date) {
        case ..<12: "Good morning"
        case ..<17: "Good afternoon"
        default: "Good evening"
        }
    }
}
