import Foundation
import Models
import Networking
import Testing

@testable import CreateEvent
@testable import JoinEvent

@MainActor
@Test func aDraftWithoutATitleCannotBePublished() async {
    let model = CreateEventModel(service: FakeEventService())
    #expect(!model.canPublish)

    model.draft.title = "   "
    #expect(!model.canPublish)

    await model.publish()
    #expect(model.published == nil)
    #expect(model.errorMessage != nil)
}

@MainActor
@Test func publishingSendsTheDraftAndReturnsTheEvent() async {
    let service = FakeEventService()
    let model = CreateEventModel(service: service)
    model.draft.title = "Amara's 26th Birthday"
    model.draft.category = .birthday
    model.draft.locationName = "Victoria Island, Lagos"

    await model.publish()

    #expect(model.published?.title == "Amara's 26th Birthday")
    #expect(model.published?.status == .published)
    #expect(model.errorMessage == nil)
}

@MainActor
@Test func retryingAFailedPublishReusesTheKeySoNoDuplicateEventIsCreated() async {
    let service = FakeEventService()
    await service.failCreation(with: .transport("dropped"))
    let model = CreateEventModel(service: service)
    model.draft.title = "Sunday Super Club"

    await model.publish()
    await model.publish()

    let keys = await service.keys()
    #expect(keys.count == 2)
    #expect(keys[0] == keys[1], "the same draft retried must not create two events")
}

@MainActor
@Test func editingTheDraftAfterAFailureStartsAFreshRequest() async {
    let service = FakeEventService()
    await service.failCreation(with: .transport("dropped"))
    let model = CreateEventModel(service: service)
    model.draft.title = "First title"
    await model.publish()

    model.draft.title = "Second title"
    await model.publish()

    let keys = await service.keys()
    #expect(keys[0] != keys[1])
}

@MainActor
@Test func theTimeToCreateTimerStartsAtTheFirstEditNotAtPublish() async {
    let model = CreateEventModel(service: FakeEventService())
    #expect(model.timeToCreateSeconds == nil)

    let start = Date().addingTimeInterval(-45)
    model.beginEditing(now: start)
    model.beginEditing(now: Date())
    model.draft.title = "Game Night"
    await model.publish()

    let elapsed = try? #require(model.timeToCreateSeconds)
    #expect((elapsed ?? 0) >= 45, "the timer must run from first focus, not from publish")
}

@Test(arguments: [
    ("abcdefghjk", "ABCDEFGHJK"),
    ("ABCD-EFGH JK", "ABCDEFGHJK"),
    ("iloo00o1ab", "1100000" + "1AB"),
    ("!!ABCDEFGHJK!!", "ABCDEFGHJK"),
    ("ABCDEFGHJKEXTRA", "ABCDEFGHJK"),
])
func theCodeFieldForgivesHowPeopleActuallyTypeCodes(raw: String, expected: String) {
    #expect(InviteCodeInput.normalize(raw) == expected)
}

@Test func uIsRejectedRatherThanNormalizedJustLikeTheServer() {
    #expect(!InviteCodeInput.normalize("UUUUUUUUUU").contains("U"))
    #expect(InviteCodeInput.normalize("UUUUUUUUUU").isEmpty)
}

@MainActor
@Test func theJoinButtonStaysDisabledUntilTheCodeIsComplete() {
    let model = JoinEventModel(service: FakeEventService())
    model.code = "ABC"
    #expect(!model.canSubmit)

    model.code = "ABCDEFGHJK"
    #expect(model.canSubmit)
}

@MainActor
@Test func typingIsNormalizedLiveSoTheFieldNeverHoldsAnInvalidCode() {
    let model = JoinEventModel(service: FakeEventService())
    model.code = "abcd-efgh jk"
    #expect(model.code == "ABCDEFGHJK")
}

@MainActor
@Test func eachInviteFailureGetsItsOwnExplanation() async {
    let cases: [(GathrError, String)] = [
        (.api(code: .inviteExpired, message: "x", requestId: "r"), "expired"),
        (.api(code: .inviteExhausted, message: "x", requestId: "r"), "used up"),
        (.api(code: .inviteInvalid, message: "x", requestId: "r"), "doesn't match"),
        (.api(code: .eventCancelled, message: "x", requestId: "r"), "cancelled"),
    ]

    for (error, fragment) in cases {
        let service = FakeEventService()
        await service.failInviteResolution(with: error)
        let model = JoinEventModel(service: service)
        model.code = "ABCDEFGHJK"
        await model.resolve()
        #expect(
            model.errorMessage?.localizedCaseInsensitiveContains(fragment) == true,
            "\(error) should explain itself with \(fragment)"
        )
    }
}
