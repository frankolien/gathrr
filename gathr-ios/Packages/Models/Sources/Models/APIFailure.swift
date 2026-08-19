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

