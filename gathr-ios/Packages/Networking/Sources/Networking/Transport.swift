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

public struct URLSessionTransport: Transport {
    private let session: URLSession

    public init(session: URLSession = .shared) {
        self.session = session
    }

    public func perform(_ request: URLRequest) async throws -> HTTPResponse {
        let (data, response) = try await session.data(for: request)
        guard let http = response as? HTTPURLResponse else {
            throw GathrError.transport("the server did not return an HTTP response")
        }
        return HTTPResponse(
            status: http.statusCode,
            body: data,
            requestId: http.value(forHTTPHeaderField: "X-Request-Id")
        )
    }
}
