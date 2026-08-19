import Foundation
import Models
import Networking
import Testing

@testable import Notifications

actor FakeActivityService: ActivityService {
    private var feedResult: Result<ActivityFeed, GathrError>
    private(set) var markReadCalls = 0

    init(feedResult: Result<ActivityFeed, GathrError> = .success(ActivityFeed(unread: 0, notifications: []))) {
        self.feedResult = feedResult
    }

    func feed(before: Date?, limit: Int?) async throws -> ActivityFeed {
        try feedResult.get()
    }

    func markRead(_ ids: [UUID]) async throws -> Int {
        markReadCalls += 1
        return 0
    }
}

private func entry(
    _ kind: ActivityKind,
    at moment: Date,
    actor: String? = "Tunde Bello",
    read: Bool = false
) -> ActivityEntry {
    ActivityEntry(
        id: UUID(),
        kind: kind,
        eventId: UUID(),
        eventTitle: "Rooftop Supper",
        actorDisplayName: actor,
        read: read,
        createdAt: moment
    )
}

