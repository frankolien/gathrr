import Foundation

public struct HTTPResponse: Sendable {
    public let status: Int
    public let body: Data
    public let requestId: String?

    public init(status: Int, body: Data, requestId: String?) {
        self.status = status
        self.body = body
        self.requestId = requestId
    }
}

public protocol Transport: Sendable {
    func perform(_ request: URLRequest) async throws -> HTTPResponse
}

