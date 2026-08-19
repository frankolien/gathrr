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

