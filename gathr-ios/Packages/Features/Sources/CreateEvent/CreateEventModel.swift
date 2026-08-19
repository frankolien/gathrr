import Foundation
import Models
import Networking
import Observation

@MainActor
@Observable
public final class CreateEventModel {
    private let service: any EventService
    private var draftKey: String?
    private var keyedDraft: NewEvent?

    public var draft = NewEvent()
    public private(set) var isPublishing = false
    public private(set) var errorMessage: String?
    public private(set) var published: Event?
    public private(set) var startedAt: Date?

    public init(service: any EventService) {
        self.service = service
    }

    public var canPublish: Bool {
        draft.isValid && !isPublishing
    }

    public var timeToCreateSeconds: TimeInterval? {
        guard let startedAt, published != nil else { return nil }
        return Date().timeIntervalSince(startedAt)
    }

    public func beginEditing(now: Date = .now) {
        if startedAt == nil { startedAt = now }
    }

    public func publish() async {
        guard draft.isValid else {
            errorMessage = "Give your event a title first."
            return
        }

        isPublishing = true
        errorMessage = nil
        defer { isPublishing = false }

        do {
            published = try await service.create(draft, idempotencyKey: idempotencyKey())
        } catch let error as GathrError {
            errorMessage = error.userFacingMessage
        } catch {
            errorMessage = "Something went wrong."
        }
    }

    private func idempotencyKey() -> String {
        if keyedDraft == draft, let draftKey {
            return draftKey
        }
        let fresh = IdempotencyKey.generateOncePerMutationNeverPerRetry()
        keyedDraft = draft
        draftKey = fresh
        return fresh
    }
}
