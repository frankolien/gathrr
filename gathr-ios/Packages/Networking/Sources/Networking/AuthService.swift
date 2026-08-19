import Foundation
import Models

public protocol AuthService: Sendable {
    func signIn(with credential: IdentityCredential) async throws -> TokenPair
    func signInForDevelopment(displayName: String) async throws -> TokenPair
    func requestCode(
        channel: VerificationChannel,
        destination: String
    ) async throws -> VerificationChallenge
    func verifyCode(
        channel: VerificationChannel,
        destination: String,
        code: String
    ) async throws -> TokenPair
    func updateProfile(_ edit: ProfileEdit) async throws -> Account
    func refresh(using refreshToken: String) async throws -> TokenPair
    func me() async throws -> Account
}

public struct LiveAuthService: AuthService {
    private let client: APIClient
    private let encoder = GathrJSON.encoder()

    public init(client: APIClient) {
        self.client = client
    }

    public func signIn(with credential: IdentityCredential) async throws -> TokenPair {
        struct Body: Encodable {
            let provider: String
            let idToken: String
            let nonce: String?
            let displayName: String?
        }

        let body = Body(
            provider: credential.provider.rawValue,
            idToken: credential.idToken,
            nonce: credential.nonce,
            displayName: credential.displayName
        )

        return try await client.send(
            Endpoint(
                method: .post,
                path: "v1/auth/oauth",
                body: try encoder.encode(body),
                requiresAuth: false
            ),
            as: TokenPair.self
        )
    }

    public func requestCode(
        channel: VerificationChannel,
        destination: String
    ) async throws -> VerificationChallenge {
        struct Body: Encodable {
            let channel: String
            let destination: String
        }

        return try await client.send(
            Endpoint(
                method: .post,
                path: "v1/auth/otp/request",
                body: try encoder.encode(
                    Body(channel: channel.rawValue, destination: destination)
                ),
                requiresAuth: false
            ),
            as: VerificationChallenge.self
        )
    }

    public func verifyCode(
        channel: VerificationChannel,
        destination: String,
        code: String
    ) async throws -> TokenPair {
        struct Body: Encodable {
            let channel: String
            let destination: String
            let code: String
        }

        return try await client.send(
            Endpoint(
                method: .post,
                path: "v1/auth/otp/verify",
                body: try encoder.encode(
                    Body(channel: channel.rawValue, destination: destination, code: code)
                ),
                requiresAuth: false
            ),
            as: TokenPair.self
        )
    }

    public func updateProfile(_ edit: ProfileEdit) async throws -> Account {
        struct Body: Encodable {
            let displayName: String?
            let bio: String?
            let avatarMediaId: UUID?
        }

        let body = Body(
            displayName: edit.displayName,
            bio: edit.bio,
            avatarMediaId: edit.avatarMediaID
        )

        return try await client.send(
            Endpoint(method: .patch, path: "v1/me", body: try encoder.encode(body)),
            as: Account.self
        )
    }

    public func signInForDevelopment(displayName: String) async throws -> TokenPair {
        struct Body: Encodable {
            let displayName: String
        }

        return try await client.send(
            Endpoint(
                method: .post,
                path: "v1/auth/dev",
                body: try encoder.encode(Body(displayName: displayName)),
                requiresAuth: false
            ),
            as: TokenPair.self
        )
    }

    public func refresh(using refreshToken: String) async throws -> TokenPair {
        struct Body: Encodable {
            let refreshToken: String
        }

        return try await client.send(
            Endpoint(
                method: .post,
                path: "v1/auth/refresh",
                body: try encoder.encode(Body(refreshToken: refreshToken)),
                requiresAuth: false
            ),
            as: TokenPair.self
        )
    }

    public func me() async throws -> Account {
        try await client.send(Endpoint(path: "v1/me"), as: Account.self)
    }
}
