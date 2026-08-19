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

