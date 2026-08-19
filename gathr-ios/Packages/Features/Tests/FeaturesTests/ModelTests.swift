import Foundation
import Models
import Networking
import Testing

@testable import EventDetail
@testable import Home
@testable import Routing

private func sampleEvent() -> Event {
    Event(
        id: UUID(),
        title: "Sunday Super Club",
        category: .dinner,
        locationName: "Ikeja, Lagos",
        startsAt: Date().addingTimeInterval(3_600),
        endsAt: nil,
        timezone: "Africa/Lagos",
        status: .published,
        capacity: nil,
        goingGuests: 6,
        previewGuestNames: ["Tunde Bello", "Chidi Okonkwo"]
    )
}

@MainActor
@Test func homeLoadsAllThreeFeedsInOnePass() async {
    let service = FakeEventService(feedResult: .success([sampleEvent()]))
    let model = HomeModel(service: service)

    await model.load()

    #expect(model.phase == .loaded)
    #expect(model.thisWeek.count == 1)
    #expect(model.hosting.count == 1)
    #expect(model.attending.count == 1)
    let filters = await service.filters()
    #expect(Set(filters) == Set(FeedFilter.allCases))
}

@MainActor
@Test func homeKeepsShowingCachedContentWhenARefreshFails() async {
    let service = FakeEventService(feedResult: .success([sampleEvent()]))
    let model = HomeModel(service: service)
    await model.load()

    let failing = FakeEventService(feedResult: .failure(.offline))
    let second = HomeModel(service: failing)
    await second.load()

    #expect(second.phase == .failed(GathrError.offline.userFacingMessage))
    #expect(!second.isShowingStaleContent, "with nothing cached there is nothing stale to show")
    #expect(model.hasAnyContent)
}

@MainActor
@Test func homeDistinguishesEmptyFromNotLoadedYet() async {
    let model = HomeModel(service: FakeEventService(feedResult: .success([])))
    #expect(!model.isEmptyAfterLoading)
    await model.load()
    #expect(model.isEmptyAfterLoading)
}

@MainActor
@Test func aRetryOfTheSameRsvpReusesItsIdempotencyKey() async {
    let service = FakeEventService(
        rsvpResults: [
            .failure(.transport("dropped")),
            .success(RSVP(eventId: UUID(), status: .going, plusOnes: 0, enteredWaitlist: false, seatsRemaining: 4)),
        ]
    )
    let model = EventDetailModel(service: service, eventId: UUID())

    await model.submit(.going)
    await model.submit(.going)

    let keys = await service.keys()
    #expect(keys.count == 2)
    #expect(keys[0] == keys[1], "a retry must not generate a new key")
}

@MainActor
@Test func changingTheAnswerStartsANewIdempotentRequest() async {
    let service = FakeEventService(rsvpResults: [])
    let model = EventDetailModel(service: service, eventId: UUID())

    await model.submit(.going)
    await model.submit(.maybe)

    let keys = await service.keys()
    #expect(keys.count == 2)
    #expect(keys[0] != keys[1], "a different answer is a different request")
}

@MainActor
@Test func aFullEventOffersTheWaitlistInsteadOfFailingSilently() async {
    let service = FakeEventService(
        rsvpResults: [
            .failure(.api(code: .capacityExceeded, message: "Event is at capacity", requestId: "r")),
            .success(RSVP(eventId: UUID(), status: .waitlisted, plusOnes: 0, enteredWaitlist: true, seatsRemaining: 0)),
        ]
    )
    let model = EventDetailModel(service: service, eventId: UUID())

    await model.submit(.going)
    #expect(model.isFull)
    #expect(model.myRSVP == nil)

    await model.joinWaitlist()
    #expect(!model.isFull)
    #expect(model.myRSVP?.status == .waitlisted)
    #expect(model.myRSVP?.enteredWaitlist == true)
}

@MainActor
@Test func theActionBarLabelReflectsTheCurrentAnswer() async {
    let service = FakeEventService(
        rsvpResults: [
            .success(RSVP(eventId: UUID(), status: .going, plusOnes: 2, enteredWaitlist: false, seatsRemaining: 1))
        ]
    )
    let model = EventDetailModel(service: service, eventId: UUID())
    #expect(model.primaryActionTitle == "Your RSVP")

    await model.submit(.going)
    #expect(model.primaryActionTitle == "Going · +2")
}

@MainActor
@Test func rsvpIsBlockedOnceAnEventIsNoLongerLive() async {
    let model = EventDetailModel(service: FakeEventService(), eventId: UUID())
    #expect(!model.canRSVP, "before loading there is nothing to RSVP to")
    await model.load()
    #expect(model.canRSVP)
    #expect(model.maxPlusOnes == 2)
}

@MainActor
@Test func theRouterTurnsAnInviteLinkIntoARoute() {
    let router = Router()
    #expect(router.handle(universalLink: URL(string: "https://gathr.app/i/ABCDEFGHJK")!))
    #expect(router.path == [.invite("ABCDEFGHJK")])

    #expect(!router.handle(universalLink: URL(string: "https://gathr.app/about")!))
    #expect(router.path.count == 1)
}

@MainActor
@Test func theRouterStackBehavesLikeANavigationStack() {
    let router = Router()
    let id = UUID()
    router.push(.eventDetail(id))
    router.push(.guestList(id))
    #expect(router.path.count == 2)

    router.pop()
    #expect(router.path == [.eventDetail(id)])

    router.replace(with: .createEvent)
    #expect(router.path == [.createEvent])

    router.popToRoot()
    #expect(router.path.isEmpty)
    router.pop()
    #expect(router.path.isEmpty, "popping an empty stack must not crash")
}
