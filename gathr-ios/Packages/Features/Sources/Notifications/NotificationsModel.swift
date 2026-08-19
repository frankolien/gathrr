import Foundation
import Models
import Networking
import Observation

public enum ActivityBucket: Sendable, Hashable, CaseIterable {
    case today
    case yesterday
    case thisWeek
    case earlier
}

public struct ActivitySection: Identifiable, Sendable, Hashable {
    public let bucket: ActivityBucket
    public let entries: [ActivityEntry]

    public var id: ActivityBucket { bucket }
}

@MainActor
@Observable
public final class NotificationsModel {
    public enum Phase: Equatable {
        case idle
        case loading
        case loaded
        case failed(String)
    }

    private let service: any ActivityService

    public private(set) var entries: [ActivityEntry] = []
    public private(set) var unread = 0
    public private(set) var phase: Phase = .idle

    public init(service: any ActivityService) {
        self.service = service
    }

    public var isEmptyAfterLoading: Bool {
        phase == .loaded && entries.isEmpty
    }

    public func load() async {
        if entries.isEmpty {
            phase = .loading
        }

        do {
            let feed = try await service.feed(before: nil, limit: nil)
            entries = feed.notifications
            unread = feed.unread
            phase = .loaded
        } catch let error as GathrError {
            phase = .failed(error.userFacingMessage)
        } catch {
            phase = .failed("Something went wrong.")
        }
    }

    public func markEverythingRead() async {
        guard unread > 0 else { return }

        do {
            unread = try await service.markRead([])
            entries = entries.map(read)
        } catch {
            await load()
        }
    }

    public func sections(now: Date = .now, timezone: String = "Africa/Lagos") -> [ActivitySection] {
        let calendar = calendar(timezone)
        let grouped = Dictionary(grouping: entries) { bucket(for: $0.createdAt, now: now, in: calendar) }

        return ActivityBucket.allCases.compactMap { bucket in
            guard let entries = grouped[bucket], !entries.isEmpty else { return nil }
            return ActivitySection(bucket: bucket, entries: entries)
        }
    }

    func bucket(for moment: Date, now: Date, in calendar: Calendar) -> ActivityBucket {
        if calendar.isDate(moment, inSameDayAs: now) {
            return .today
        }
        if let yesterday = calendar.date(byAdding: .day, value: -1, to: now),
            calendar.isDate(moment, inSameDayAs: yesterday) {
            return .yesterday
        }
        if let weekAgo = calendar.date(byAdding: .day, value: -7, to: now), moment >= weekAgo {
            return .thisWeek
        }
        return .earlier
    }

    private func read(_ entry: ActivityEntry) -> ActivityEntry {
        ActivityEntry(
            id: entry.id,
            kind: entry.kind,
            eventId: entry.eventId,
            eventTitle: entry.eventTitle,
            actorDisplayName: entry.actorDisplayName,
            read: true,
            createdAt: entry.createdAt
        )
    }

    private func calendar(_ timezone: String) -> Calendar {
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = TimeZone(identifier: timezone) ?? .current
        return calendar
    }
}
