import Foundation
import Models
import Testing

@testable import Networking

private actor RecordingTransport: Transport {
    private var responses: [HTTPResponse]
    private(set) var requests: [URLRequest] = []

    init(responses: [HTTPResponse]) {
        self.responses = responses
    }

    func perform(_ request: URLRequest) async throws -> HTTPResponse {
        requests.append(request)
        guard !responses.isEmpty else {
            return HTTPResponse(status: 500, body: Data(), requestId: nil)
        }
        return responses.removeFirst()
    }

    func lastRequest() -> URLRequest? { requests.last }
    func requestCount() -> Int { requests.count }
}

private func ok(_ json: String) -> HTTPResponse {
    HTTPResponse(status: 200, body: Data(json.utf8), requestId: "req-1")
}

private func failure(_ status: Int, _ code: String, _ message: String) -> HTTPResponse {
    HTTPResponse(
        status: status,
        body: Data(
            #"{"error":{"code":"\#(code)","message":"\#(message)","request_id":"req-err"}}"#.utf8
        ),
        requestId: "req-err"
    )
}

private func service(_ transport: RecordingTransport, token: String? = "abc123") -> LiveEventService {
    LiveEventService(
        client: APIClient(
            baseURL: URL(string: "https://api.gathr.test")!,
            transport: transport,
            tokens: StaticTokenStorage(token: token)
        )
    )
}

private let eventJSON = """
{"id":"9f2c3d4e-5a6b-7c8d-9e0f-1a2b3c4d5e6f","title":"Amara's 26th Birthday",
 "category":"birthday","location_name":"Victoria Island, Lagos",
 "starts_at":"2026-09-08T18:00:00Z","ends_at":null,"timezone":"Africa/Lagos",
 "status":"published","capacity":null,"going_guests":18,
 "preview_guest_names":["Tunde Bello","Chidi Okonkwo"]}
"""

@Test func theFeedRequestCarriesTheFilterAndTheBearerToken() async throws {
    let transport = RecordingTransport(responses: [ok("[\(eventJSON)]")])
    let events = try await service(transport).feed(.thisWeek)

    #expect(events.count == 1)
    #expect(events[0].title == "Amara's 26th Birthday")

    let request = await transport.lastRequest()
    #expect(request?.url?.path == "/v1/events")
    #expect(request?.url?.query == "filter=this_week")
    #expect(request?.value(forHTTPHeaderField: "Authorization") == "Bearer abc123")
}

@Test func publicEndpointsAreSentWithoutCredentials() async throws {
    let transport = RecordingTransport(
        responses: [
            ok(
                """
                {"event_id":"9f2c3d4e-5a6b-7c8d-9e0f-1a2b3c4d5e6f","title":"Amara's 26th Birthday",
                 "category":"birthday","location_name":"Victoria Island, Lagos",
                 "starts_at":"2026-09-08T18:00:00Z","timezone":"Africa/Lagos",
                 "host_first_name":"Amara","going_guests":18}
                """
            )
        ]
    )
    let invite = try await service(transport).resolveInvite(code: "ABCDEFGHJK")

    #expect(invite.hostFirstName == "Amara")
    #expect(invite.goingGuests == 18)
    let request = await transport.lastRequest()
    #expect(request?.value(forHTTPHeaderField: "Authorization") == nil)
}

@Test func anRsvpSendsTheIdempotencyKeyItWasGiven() async throws {
    let transport = RecordingTransport(
        responses: [
            ok(
                """
                {"event_id":"9f2c3d4e-5a6b-7c8d-9e0f-1a2b3c4d5e6f","status":"going",
                 "plus_ones":1,"entered_waitlist":false,"seats_remaining":3}
                """
            )
        ]
    )

    let key = "the-key-from-the-outbox"
    let rsvp = try await service(transport).rsvp(
        UUID(uuidString: "9F2C3D4E-5A6B-7C8D-9E0F-1A2B3C4D5E6F")!,
        status: .going,
        plusOnes: 1,
        acceptWaitlist: false,
        idempotencyKey: key
    )

    #expect(rsvp.status == .going)
    #expect(rsvp.seatsRemaining == 3)

    let request = await transport.lastRequest()
    #expect(request?.httpMethod == "POST")
    #expect(request?.value(forHTTPHeaderField: "Idempotency-Key") == key)
    #expect(request?.value(forHTTPHeaderField: "Content-Type") == "application/json")

    let body = try #require(request?.httpBody)
    let sent = try JSONSerialization.jsonObject(with: body) as? [String: Any]
    #expect(sent?["plus_ones"] as? Int == 1)
    #expect(sent?["accept_waitlist"] as? Bool == false)
    #expect(sent?["status"] as? String == "going")
}

@Test func aFullEventSurfacesAsAWaitlistOffer() async {
    let transport = RecordingTransport(
        responses: [failure(409, "capacity_exceeded", "Event is at capacity")]
    )

    await #expect(throws: GathrError.self) {
        _ = try await service(transport).rsvp(
            UUID(),
            status: .going,
            plusOnes: 0,
            acceptWaitlist: false,
            idempotencyKey: "k"
        )
    }

    do {
        _ = try await service(transport).feed(.thisWeek)
    } catch let error as GathrError {
        #expect(!error.offersWaitlist)
    } catch {
        Issue.record("unexpected error type")
    }
}

@Test func errorCodesMapToTheBehaviourTheUiNeeds() async {
    let cases: [(Int, String, (GathrError) -> Bool)] = [
        (409, "capacity_exceeded", { $0.offersWaitlist }),
        (401, "unauthenticated", { $0.requiresReauthentication }),
        (401, "token_reuse_detected", { $0.requiresReauthentication }),
        (429, "rate_limited", { $0.isRetryable }),
        (403, "forbidden", { !$0.requiresReauthentication && !$0.isRetryable }),
    ]

    for (status, code, assertion) in cases {
        let transport = RecordingTransport(responses: [failure(status, code, "m")])
        do {
            _ = try await service(transport).feed(.thisWeek)
            Issue.record("\(code) should not have succeeded")
        } catch let error as GathrError {
            #expect(assertion(error), "\(code) mapped to the wrong behaviour")
            #expect(error.supportReference == "req-err")
        } catch {
            Issue.record("\(code) produced the wrong error type")
        }
    }
}

@Test func anErrorBodyThatIsNotJsonStillProducesAUsableError() async {
    let transport = RecordingTransport(
        responses: [HTTPResponse(status: 502, body: Data("<html>oops</html>".utf8), requestId: "gw-1")]
    )

    do {
        _ = try await service(transport).feed(.thisWeek)
        Issue.record("a 502 should not have succeeded")
    } catch let error as GathrError {
        #expect(error.code == .internalFailure)
        #expect(error.supportReference == "gw-1")
        #expect(!error.userFacingMessage.isEmpty)
    } catch {
        Issue.record("wrong error type")
    }
}

@Test func aMalformedSuccessBodyIsReportedAsDecodingNotAsSuccess() async {
    let transport = RecordingTransport(responses: [ok(#"{"unexpected":true}"#)])

    do {
        _ = try await service(transport).detail(UUID())
        Issue.record("a malformed body should not decode")
    } catch let error as GathrError {
        #expect(error.code == nil)
        #expect(!error.isRetryable)
    } catch {
        Issue.record("wrong error type")
    }
}

@Test func offlineIsDistinctFromAServerFailure() {
    #expect(GathrError.offline.isRetryable)
    #expect(GathrError.offline.code == nil)
    #expect(GathrError.offline.userFacingMessage.contains("offline"))
}
