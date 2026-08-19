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

