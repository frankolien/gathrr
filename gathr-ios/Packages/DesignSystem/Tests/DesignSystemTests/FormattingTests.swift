import Foundation
import Models
import Testing

@testable import DesignSystem

private let lagos = "Africa/Lagos"
private let posix = Locale(identifier: "en_US_POSIX")

private func date(_ iso: String) -> Date {
    try! Date(iso, strategy: .iso8601)
}

private func replacingNarrowNoBreakSpaceThatIOSPutsBeforeAmPm(_ value: String) -> String {
    value.replacingOccurrences(of: "\u{202F}", with: " ")
}

private func normalized(_ value: String) -> String {
    replacingNarrowNoBreakSpaceThatIOSPutsBeforeAmPm(value)
}

@Test func theCanonicalDateStyleMatchesTheSpecification() {
    let formatted = EventFormatting.longWhen(
        date("2026-08-08T18:00:00Z"),
        timezone: lagos,
        locale: posix
    )
    #expect(normalized(formatted) == "Sat, Aug 8 · 7:00 PM")
}

@Test func timesRenderInTheEventTimezoneNotTheDeviceTimezone() {
    let instant = date("2026-09-08T23:30:00Z")
    let inLagos = normalized(EventFormatting.longWhen(instant, timezone: lagos, locale: posix))
    let inUTC = normalized(EventFormatting.longWhen(instant, timezone: "UTC", locale: posix))

    #expect(inLagos.contains("12:30 AM"), "WAT is UTC+1 so this rolls into the next day")
    #expect(inLagos.contains("Sep 9"))
    #expect(inUTC.contains("Sep 8"))
}

@Test func anUnknownTimezoneFallsBackRatherThanCrashing() {
    let formatted = EventFormatting.longWhen(
        date("2026-09-08T18:00:00Z"),
        timezone: "Mars/Olympus",
        locale: posix
    )
    #expect(normalized(formatted) == "Tue, Sep 8 · 6:00 PM")
}

@Test func countdownSplitsIntoTheSegmentsTheDetailScreenShows() {
    let now = date("2026-08-30T03:28:00Z")
    let target = date("2026-09-08T18:00:00Z")
    let parts = CountdownParts(until: target, from: now)

    #expect(parts.days == 9)
    #expect(parts.hours == 14)
    #expect(parts.minutes == 32)
    #expect(!parts.hasStarted)
}

@Test func countdownNeverGoesNegative() {
    let parts = CountdownParts(
        until: date("2026-09-08T18:00:00Z"),
        from: date("2026-09-09T18:00:00Z")
    )
    #expect(parts.days == 0)
    #expect(parts.hours == 0)
    #expect(parts.minutes == 0)
    #expect(parts.hasStarted)
}

@Test(arguments: [
    ("2026-09-08T17:00:00Z", "In 1 hour"),
    ("2026-09-08T15:00:00Z", "In 3 hours"),
    ("2026-09-07T18:00:00Z", "In 1 day"),
    ("2026-08-30T18:00:00Z", "In 9 days"),
    ("2026-09-08T17:45:00Z", "In 15 min"),
    ("2026-09-08T18:30:00Z", "Happening now"),
])
func theCountdownPillReadsNaturallyAtEveryScale(from: String, expected: String) {
    let phrase = EventFormatting.countdownPhrase(
        until: date("2026-09-08T18:00:00Z"),
        from: date(from)
    )
    #expect(phrase == expected)
}

@Test func theAvatarClusterOverflowMatchesTheMockupArithmetic() {
    #expect(EventFormatting.clusterOverflow(goingGuests: 18, shown: 4) == "+14 going")
    #expect(EventFormatting.clusterOverflow(goingGuests: 4, shown: 4) == nil)
    #expect(EventFormatting.clusterOverflow(goingGuests: 2, shown: 4) == nil)
}

@Test func theGoingSummaryMatchesTheDetailScreenCopy() {
    let summary = EventFormatting.goingSummary(goingGuests: 18, hostDisplayName: "Amara Chukwu")
    #expect(summary == "18 going · hosted by Amara Chukwu")
}

@Test(arguments: [
    ("2026-09-08T06:00:00Z", "Good morning"),
    ("2026-09-08T12:00:00Z", "Good afternoon"),
    ("2026-09-08T18:00:00Z", "Good evening"),
])
func theGreetingFollowsLagosLocalTime(instant: String, expected: String) {
    #expect(EventFormatting.greeting(at: date(instant), timezone: lagos) == expected)
}

@Test func everyCategoryResolvesToACompleteStyle() {
    for category in EventCategory.allCases {
        #expect(!category.style.label.isEmpty)
        #expect(!category.style.symbol.isEmpty)
    }
}

@Test func everyRsvpStatusHasUserFacingCopy() {
    for status in RSVPStatus.allCases {
        #expect(!status.label.isEmpty)
        #expect(!status.symbol.isEmpty)
    }
    #expect(RSVPStatus.declined.label == "Can't go")
}
