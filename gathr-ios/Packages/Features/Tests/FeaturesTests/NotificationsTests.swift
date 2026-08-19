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

private let noon = Date(timeIntervalSince1970: 1_787_400_000)

@MainActor
@Test func theFeedSplitsIntoTodayYesterdayThisWeekAndEarlier() async {
    let feed = ActivityFeed(
        unread: 2,
        notifications: [
            entry(.rsvpAccepted, at: noon.addingTimeInterval(-3600)),
            entry(.messagePosted, at: noon.addingTimeInterval(-26 * 3600)),
            entry(.eventReminder, at: noon.addingTimeInterval(-4 * 86_400)),
            entry(.eventCancelled, at: noon.addingTimeInterval(-30 * 86_400)),
        ]
    )
    let model = NotificationsModel(service: FakeActivityService(feedResult: .success(feed)))

    await model.load()
    let sections = model.sections(now: noon)

    #expect(sections.map(\.bucket) == [.today, .yesterday, .thisWeek, .earlier])
    #expect(sections.allSatisfy { $0.entries.count == 1 })
}

@MainActor
@Test func anEmptyFeedReportsItselfEmptyOnlyAfterLoading() async {
    let model = NotificationsModel(service: FakeActivityService())
    #expect(!model.isEmptyAfterLoading)

    await model.load()
    #expect(model.isEmptyAfterLoading)
}

@MainActor
@Test func markingEverythingReadClearsTheBadgeAndEveryDot() async {
    let feed = ActivityFeed(
        unread: 2,
        notifications: [entry(.rsvpAccepted, at: noon), entry(.messagePosted, at: noon)]
    )
    let service = FakeActivityService(feedResult: .success(feed))
    let model = NotificationsModel(service: service)
    await model.load()

    await model.markEverythingRead()

    #expect(model.unread == 0)
    #expect(model.entries.allSatisfy { $0.read })
}

@MainActor
@Test func anAlreadyClearBadgeDoesNotCallTheServer() async {
    let service = FakeActivityService()
    let model = NotificationsModel(service: service)
    await model.load()

    await model.markEverythingRead()

    let calls = await service.markReadCalls
    #expect(calls == 0)
}

@MainActor
@Test func aFailedLoadSurfacesTheMessageRatherThanAnEmptyList() async {
    let service = FakeActivityService(feedResult: .failure(.offline))
    let model = NotificationsModel(service: service)

    await model.load()

    #expect(model.phase == .failed(GathrError.offline.userFacingMessage))
    #expect(!model.isEmptyAfterLoading)
}
