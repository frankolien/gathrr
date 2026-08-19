import Foundation
import Testing

@testable import Models

private func decode<T: Decodable>(_ type: T.Type, _ json: String) throws -> T {
    try GathrJSON.decoder().decode(type, from: Data(json.utf8))
}

@Test func eventDecodesTheShapeTheBackendActuallySends() throws {
    let event = try decode(
        Event.self,
        """
        {"id":"9f2c3d4e-5a6b-7c8d-9e0f-1a2b3c4d5e6f","title":"Amara's 26th Birthday",
         "category":"birthday","location_name":"Victoria Island, Lagos",
         "starts_at":"2026-09-08T18:00:00Z","ends_at":null,
         "timezone":"Africa/Lagos","status":"published","capacity":null,
         "going_guests":18,"preview_guest_names":["Tunde Bello","Chidi Okonkwo"]}
        """
    )

    #expect(event.title == "Amara's 26th Birthday")
    #expect(event.category == .birthday)
    #expect(event.timezone == "Africa/Lagos")
    #expect(event.goingGuests == 18)
    #expect(event.previewGuestNames == ["Tunde Bello", "Chidi Okonkwo"])
    #expect(event.capacity == nil)

    var components = DateComponents()
    components.year = 2026
    components.month = 9
    components.day = 8
    components.hour = 18
    components.timeZone = TimeZone(identifier: "UTC")
    let expected = Calendar(identifier: .gregorian).date(from: components)
    #expect(event.startsAt == expected)
}

@Test func unknownCategoriesDegradeInsteadOfFailingTheWholeFeed() throws {
    let event = try decode(
        Event.self,
        """
        {"id":"9f2c3d4e-5a6b-7c8d-9e0f-1a2b3c4d5e6f","title":"Something new",
         "category":"hackathon","location_name":null,"starts_at":"2026-09-08T18:00:00Z",
         "ends_at":null,"timezone":"Africa/Lagos","status":"published","capacity":null,
         "going_guests":18,"preview_guest_names":["Tunde Bello","Chidi Okonkwo"]}
        """
    )
    #expect(event.category == .other)
}

@Test func eventDetailFlattensTheSummaryTheWayTheApiSendsIt() throws {
    let detail = try decode(
        EventDetail.self,
        """
        {"id":"9f2c3d4e-5a6b-7c8d-9e0f-1a2b3c4d5e6f","title":"Amara's 26th Birthday",
         "category":"birthday","location_name":"Victoria Island, Lagos",
         "starts_at":"2026-09-08T18:00:00Z","ends_at":null,"timezone":"Africa/Lagos",
         "status":"published","capacity":20,"going_guests":18,
         "preview_guest_names":["Tunde Bello"],"description":"Good food, good music.",
         "host_display_name":"Amara Chukwu","observed_status":"published",
         "going_guests":18,"max_plus_ones":2,"server_time":"2026-08-30T03:28:00Z"}
        """
    )

    #expect(detail.goingGuests == 18)
    #expect(detail.hostDisplayName == "Amara Chukwu")
    #expect(detail.event.status == .published)
    #expect(detail.event.capacity == 20)
}

@Test func guestListSeparatesPeopleFromSeats() throws {
    let list = try decode(
        GuestList.self,
        """
        {"going":1,"seats_taken":2,"guests":[
          {"user_id":"1980d0fc-efb4-4c7f-a12d-4cca71f3262a","display_name":"Tunde Bello",
           "status":"going","plus_ones":1}]}
        """
    )

    #expect(list.going == 1)
    #expect(list.seatsTaken == 2)
    #expect(list.grouped(by: .going).count == 1)
    #expect(list.grouped(by: .waitlisted).isEmpty)
}

@Test func errorEnvelopeMapsToATypedCode() throws {
    let failure = try decode(
        APIFailure.self,
        """
        {"error":{"code":"capacity_exceeded","message":"Event is at capacity",
                  "request_id":"11111111-2222-3333-4444-555555555555"}}
        """
    )

    #expect(failure.code == .capacityExceeded)
    #expect(failure.code.offersWaitlist)
    #expect(!failure.code.requiresReauthentication)
    #expect(failure.requestId == "11111111-2222-3333-4444-555555555555")
}

@Test func unrecognisedErrorCodesFallBackRatherThanCrashing() throws {
    let failure = try decode(
        APIFailure.self,
        """
        {"error":{"code":"a_code_from_a_newer_server","message":"?","request_id":"x"}}
        """
    )
    #expect(failure.code == .internalFailure)
}

@Test func fractionalSecondTimestampsAreAccepted() throws {
    let event = try decode(
        Event.self,
        """
        {"id":"9f2c3d4e-5a6b-7c8d-9e0f-1a2b3c4d5e6f","title":"T","category":"other",
         "location_name":null,"starts_at":"2026-09-08T18:00:00.250Z","ends_at":null,
         "timezone":"UTC","status":"published","capacity":null,
         "going_guests":18,"preview_guest_names":["Tunde Bello","Chidi Okonkwo"]}
        """
    )
    #expect(event.startsAt.timeIntervalSince1970 > 1_788_638_400)
}

@Test func statusHelpersMatchTheServerInvariants() {
    #expect(RSVPStatus.going.holdsSeats)
    #expect(!RSVPStatus.waitlisted.holdsSeats)
    #expect(RSVPStatus.allCases.filter(\.isGuestSelectable).count == 3)
    #expect(EventStatus.published.acceptsRSVPs)
    #expect(!EventStatus.cancelled.acceptsRSVPs)
}
