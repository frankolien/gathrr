import Foundation

public struct APIFailure: Codable, Sendable, Hashable {
    public struct Body: Codable, Sendable, Hashable {
        public let code: String
        public let message: String
        public let requestId: String
    }

    public let error: Body

    public var code: ErrorCode { ErrorCode(rawValue: error.code) ?? .internalFailure }
    public var message: String { error.message }
    public var requestId: String { error.requestId }
}

public enum ErrorCode: String, Sendable, CaseIterable {
    case unauthenticated
    case tokenReuseDetected = "token_reuse_detected"
    case forbidden
    case notFound = "not_found"
    case inviteInvalid = "invite_invalid"
    case inviteExpired = "invite_expired"
    case inviteExhausted = "invite_exhausted"
    case capacityExceeded = "capacity_exceeded"
    case eventCancelled = "event_cancelled"
    case eventEnded = "event_ended"
    case plusOnesExceeded = "plus_ones_exceeded"
    case validationFailed = "validation_failed"
    case idempotencyConflict = "idempotency_conflict"
    case rateLimited = "rate_limited"
    case internalFailure = "internal"

    public var requiresReauthentication: Bool {
        self == .unauthenticated || self == .tokenReuseDetected
    }

    public var offersWaitlist: Bool {
        self == .capacityExceeded
    }

    public var isRetryable: Bool {
        self == .rateLimited || self == .internalFailure
    }
}
