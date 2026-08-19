import Foundation
import Models

public protocol MediaService: Sendable {
    func uploadAvatar(_ imageData: Data) async throws -> UUID
}

public struct LiveMediaService: MediaService {
    private let client: APIClient
    private let transport: any Transport
    private let encoder = GathrJSON.encoder()
    private let decoder = GathrJSON.decoder()

    public init(client: APIClient, transport: any Transport) {
        self.client = client
        self.transport = transport
    }

    public func uploadAvatar(_ imageData: Data) async throws -> UUID {
        struct TicketBody: Encodable {
            let purpose: String
        }
        struct RecordBody: Encodable {
            let publicId: String
            let contentType: String
            let width: Int?
            let height: Int?
        }

        let ticket = try await client.send(
            Endpoint(
                method: .post,
                path: "v1/media/sign",
                body: try encoder.encode(TicketBody(purpose: "avatar"))
            ),
            as: UploadTicket.self
        )

        let uploaded = try await upload(imageData, with: ticket)

        return try await client.send(
            Endpoint(
                method: .post,
                path: "v1/media",
                body: try encoder.encode(
                    RecordBody(
                        publicId: uploaded.publicID,
                        contentType: "image/jpeg",
                        width: uploaded.width,
                        height: uploaded.height
                    )
                )
            ),
            as: StoredMedia.self
        ).id
    }

    private struct CloudinaryResult: Decodable {
        let publicID: String
        let width: Int?
        let height: Int?

        enum CodingKeys: String, CodingKey {
            case publicID = "public_id"
            case width
            case height
        }
    }

    private func upload(_ imageData: Data, with ticket: UploadTicket) async throws -> CloudinaryResult {
        let boundary = "gathr.\(UUID().uuidString)"
        var request = URLRequest(url: ticket.uploadURL)
        request.httpMethod = "POST"
        request.setValue("multipart/form-data; boundary=\(boundary)", forHTTPHeaderField: "content-type")

        var body = Data()
        func field(_ name: String, _ value: String) {
            body.append(Data("--\(boundary)\r\n".utf8))
            body.append(Data("Content-Disposition: form-data; name=\"\(name)\"\r\n\r\n".utf8))
            body.append(Data("\(value)\r\n".utf8))
        }

        field("api_key", ticket.apiKey)
        field("timestamp", String(ticket.timestamp))
        field("folder", ticket.folder)
        field("signature", ticket.signature)

        body.append(Data("--\(boundary)\r\n".utf8))
        body.append(Data("Content-Disposition: form-data; name=\"file\"; filename=\"avatar.jpg\"\r\n".utf8))
        body.append(Data("Content-Type: image/jpeg\r\n\r\n".utf8))
        body.append(imageData)
        body.append(Data("\r\n--\(boundary)--\r\n".utf8))
        request.httpBody = body

        let response = try await transport.perform(request)
        guard (200..<300).contains(response.status) else {
            throw GathrError.transport("The photo could not be uploaded.")
        }

        return try JSONDecoder().decode(CloudinaryResult.self, from: response.body)
    }
}
