import Foundation
import Models

public protocol TokenStorage: Sendable {
    func accessToken() async -> String?
}

public struct StaticTokenStorage: TokenStorage {
    private let token: String?

    public init(token: String?) {
        self.token = token
    }

    public func accessToken() async -> String? {
        token
    }
}

public actor APIClient {
    private let baseURL: URL
    private let transport: any Transport
    private let tokens: any TokenStorage
    private let decoder = GathrJSON.decoder()

    public init(baseURL: URL, transport: any Transport, tokens: any TokenStorage) {
        self.baseURL = baseURL
        self.transport = transport
        self.tokens = tokens
    }

    public func send<Response: Decodable & Sendable>(
        _ endpoint: Endpoint,
        as type: Response.Type
    ) async throws -> Response {
        let response = try await perform(endpoint)
        do {
            return try decoder.decode(Response.self, from: response.body)
        } catch {
            throw GathrError.decoding(String(describing: error))
        }
    }

    public func send(_ endpoint: Endpoint) async throws {
        _ = try await perform(endpoint)
    }

    private func perform(_ endpoint: Endpoint) async throws -> HTTPResponse {
        let request = try await buildRequest(endpoint)

        let response: HTTPResponse
        do {
            response = try await transport.perform(request)
        } catch let error as GathrError {
            throw error
        } catch let error as URLError where error.code == .notConnectedToInternet {
            throw GathrError.offline
        } catch {
            throw GathrError.transport(error.localizedDescription)
        }

        guard (200..<300).contains(response.status) else {
            throw failure(from: response)
        }
        return response
    }

    private func buildRequest(_ endpoint: Endpoint) async throws -> URLRequest {
        guard
            var components = URLComponents(
                url: baseURL.appendingPathComponent(endpoint.path),
                resolvingAgainstBaseURL: false
            )
        else {
            throw GathrError.transport("\(endpoint.path) is not a valid path")
        }
        if !endpoint.query.isEmpty {
            components.queryItems = endpoint.query
        }
        guard let url = components.url else {
            throw GathrError.transport("\(endpoint.path) is not a valid URL")
        }

        var request = URLRequest(url: url)
        request.httpMethod = endpoint.method.rawValue
        request.httpBody = endpoint.body
        if endpoint.body != nil {
            request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        }
        if let key = endpoint.idempotencyKey {
            request.setValue(key, forHTTPHeaderField: "Idempotency-Key")
        }
        if endpoint.requiresAuth, let token = await tokens.accessToken() {
            request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        }
        return request
    }

    private func failure(from response: HTTPResponse) -> GathrError {
        if let envelope = try? decoder.decode(APIFailure.self, from: response.body) {
            return .api(
                code: envelope.code,
                message: envelope.message,
                requestId: envelope.requestId
            )
        }
        return .api(
            code: response.status == 401 ? .unauthenticated : .internalFailure,
            message: "The server returned \(response.status).",
            requestId: response.requestId ?? "unknown"
        )
    }
}
