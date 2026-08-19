import DesignSystem
import Foundation
import Models
import Networking
import Observation

@MainActor
@Observable
public final class HomeModel {
    public enum Phase: Equatable {
        case idle
        case loading
        case loaded
        case failed(String)
    }

    private let service: any EventService

    public private(set) var thisWeek: [Event] = []
    public private(set) var hosting: [Event] = []
    public private(set) var attending: [Event] = []
    public private(set) var phase: Phase = .idle
    public private(set) var isShowingStaleContent = false

    public init(service: any EventService) {
        self.service = service
    }

    public var hasAnyContent: Bool {
        !thisWeek.isEmpty || !hosting.isEmpty || !attending.isEmpty
    }

    public var isEmptyAfterLoading: Bool {
        phase == .loaded && !hasAnyContent
    }

    public func load() async {
        if !hasAnyContent {
            phase = .loading
        }

        do {
            async let week = service.feed(.thisWeek)
            async let hosted = service.feed(.hosting)
            async let attended = service.feed(.attending)

            let (weekResult, hostedResult, attendedResult) = try await (week, hosted, attended)
            thisWeek = weekResult
            hosting = hostedResult
            attending = attendedResult
            phase = .loaded
            isShowingStaleContent = false
        } catch let error as GathrError {
            phase = .failed(error.userFacingMessage)
            isShowingStaleContent = hasAnyContent
        } catch {
            phase = .failed("Something went wrong.")
            isShowingStaleContent = hasAnyContent
        }
    }

    private var everyEvent: [Event] {
        var seen = Set<UUID>()
        return (thisWeek + hosting + attending).filter { seen.insert($0.id).inserted }
    }

    private func calendar(_ timezone: String) -> Calendar {
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = TimeZone(identifier: timezone) ?? .current
        return calendar
    }

    public func todayCount(now: Date = .now, timezone: String = "Africa/Lagos") -> Int {
        let calendar = calendar(timezone)
        return everyEvent.filter { calendar.isDate($0.startsAt, inSameDayAs: now) }.count
    }

    public func upcomingCount(now: Date = .now, timezone: String = "Africa/Lagos") -> Int {
        let calendar = calendar(timezone)
        guard let endOfToday = calendar.dateInterval(of: .day, for: now)?.end else { return 0 }
        return everyEvent.filter { $0.startsAt >= endOfToday }.count
    }

    public var hostingCount: Int { hosting.count }

    public func plannedCount(now: Date = .now, timezone: String = "Africa/Lagos") -> Int {
        todayCount(now: now, timezone: timezone) + upcomingCount(now: now, timezone: timezone)
    }

    public func week(now: Date = .now, timezone: String = "Africa/Lagos") -> [WeekStripDay] {
        let calendar = calendar(timezone)
        guard let start = calendar.dateInterval(of: .weekOfYear, for: now)?.start else { return [] }

        var symbols = calendar.shortWeekdaySymbols.map { $0.uppercased() }
        let offset = calendar.firstWeekday - 1
        symbols = Array(symbols[offset...] + symbols[..<offset])

        return (0..<7).compactMap { index in
            guard let day = calendar.date(byAdding: .day, value: index, to: start) else { return nil }
            return WeekStripDay(
                id: day,
                symbol: symbols[index],
                number: calendar.component(.day, from: day),
                isToday: calendar.isDate(day, inSameDayAs: now),
                hasEvent: everyEvent.contains { calendar.isDate($0.startsAt, inSameDayAs: day) }
            )
        }
    }

    public func greeting(now: Date = .now, timezone: String = "Africa/Lagos") -> String {
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = TimeZone(identifier: timezone) ?? .gmt
        return switch calendar.component(.hour, from: now) {
        case ..<12: "Good morning"
        case ..<17: "Good afternoon"
        default: "Good evening"
        }
    }
}
