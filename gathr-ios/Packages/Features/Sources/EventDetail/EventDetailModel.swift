import Foundation
import Models
import Networking
import Observation

@MainActor
@Observable
public final class EventDetailModel {
    public enum Phase: Equatable {
        case idle
        case loading
        case loaded
        case failed(String)
    }

    private struct Intent: Equatable {
        let status: RSVPStatus
        let plusOnes: Int
        let acceptWaitlist: Bool
    }

    private let service: any EventService
    private let eventId: UUID
    private var activeIntent: Intent?
    private var activeKey: String?

    public private(set) var detail: EventDetail?
    public private(set) var guests: GuestList?
    public private(set) var myRSVP: RSVP?
    public private(set) var phase: Phase = .idle
    public private(set) var isFull = false
    public private(set) var submissionError: String?
    public private(set) var isSubmitting = false
    public var plusOnes = 0

    public init(service: any EventService, eventId: UUID) {
        self.service = service
        self.eventId = eventId
    }

    public var canRSVP: Bool {
        detail?.observedStatus.acceptsRSVPs ?? false
    }

    public var maxPlusOnes: Int {
        detail?.maxPlusOnes ?? 0
    }

    public var primaryActionTitle: String {
        guard let rsvp = myRSVP else { return "Your RSVP" }
        return rsvp.plusOnes > 0
            ? "\(rsvp.status.label) · +\(rsvp.plusOnes)"
            : rsvp.status.label
    }

    public func load() async {
        if detail == nil { phase = .loading }
        do {
            let loaded = try await service.detail(eventId)
            detail = loaded
            phase = .loaded
            guests = try? await service.guests(eventId)
        } catch let error as GathrError {
            phase = .failed(error.userFacingMessage)
        } catch {
            phase = .failed("Something went wrong.")
        }
    }

    public func submit(_ status: RSVPStatus) async {
        await send(Intent(status: status, plusOnes: plusOnes, acceptWaitlist: false))
    }

    public func joinWaitlist() async {
        await send(Intent(status: .going, plusOnes: plusOnes, acceptWaitlist: true))
    }

    private func send(_ intent: Intent) async {
        isSubmitting = true
        submissionError = nil
        defer { isSubmitting = false }

        do {
            let result = try await service.rsvp(
                eventId,
                status: intent.status,
                plusOnes: intent.plusOnes,
                acceptWaitlist: intent.acceptWaitlist,
                idempotencyKey: idempotencyKey(for: intent)
            )
            myRSVP = result
            isFull = false
            clearIntent()
            guests = try? await service.guests(eventId)
        } catch let error as GathrError {
            if error.offersWaitlist {
                isFull = true
            }
            submissionError = error.userFacingMessage
        } catch {
            submissionError = "Something went wrong."
        }
    }

    private func idempotencyKey(for intent: Intent) -> String {
        if activeIntent == intent, let activeKey {
            return activeKey
        }
        let fresh = IdempotencyKey.generateOncePerMutationNeverPerRetry()
        activeIntent = intent
        activeKey = fresh
        return fresh
    }

    private func clearIntent() {
        activeIntent = nil
        activeKey = nil
    }
}
