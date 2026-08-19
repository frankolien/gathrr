import Foundation
import Models

public enum GathrError: Error, Sendable, Equatable {
    case api(code: ErrorCode, message: String, requestId: String)
    case transport(String)
    case decoding(String)
    case offline

    public var code: ErrorCode? {
        if case .api(let code, _, _) = self { return code }
        return nil
    }

    public var requiresReauthentication: Bool {
        code?.requiresReauthentication ?? false
    }

    public var offersWaitlist: Bool {
        code?.offersWaitlist ?? false
    }

    public var isRetryable: Bool {
        switch self {
        case .offline, .transport: true
        case .api(let code, _, _): code.isRetryable
        case .decoding: false
        }
    }

    public var userFacingMessage: String {
        switch self {
        case .api(_, let message, _): message
        case .offline: "You're offline. We'll send this when you're back."
        case .transport: "We couldn't reach Gathr. Try again in a moment."
        case .decoding: "Something went wrong on our end."
        }
    }

    public var supportReference: String? {
        if case .api(_, _, let requestId) = self { return requestId }
        return nil
    }
}
