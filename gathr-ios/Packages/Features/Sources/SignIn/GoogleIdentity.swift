import AuthenticationServices
import Foundation
import Models
import UIKit

enum GoogleIdentityError: LocalizedError {
    case cancelled
    case noAuthorizationCode
    case exchangeFailed

    var errorDescription: String? {
        switch self {
        case .cancelled: "Sign-in was cancelled."
        case .noAuthorizationCode: "Google didn't return an authorization code."
        case .exchangeFailed: "Google wouldn't exchange that sign-in for a token."
        }
    }
}

@MainActor
final class GoogleIdentity: NSObject {
    private static let authorization = URL(string: "https://accounts.google.com/o/oauth2/v2/auth")!
    private static let exchange = URL(string: "https://oauth2.googleapis.com/token")!

    private let clientID: String
    private var session: ASWebAuthenticationSession?

    init(clientID: String) {
        self.clientID = clientID
    }

    func credential(nonce: String) async throws -> IdentityCredential {
        let verifier = SignInNonce.random()
        let code = try await authorizationCode(
            challenge: SignInNonce.base64URLHash(of: verifier),
            nonce: nonce
        )
        let token = try await identityToken(code: code, verifier: verifier)
        return IdentityCredential(provider: .google, idToken: token, nonce: nonce)
    }

    private var redirectScheme: String {
        let identifier = clientID.replacingOccurrences(
            of: ".apps.googleusercontent.com",
            with: ""
        )
        return "com.googleusercontent.apps.\(identifier)"
    }

    private var redirectURI: String {
        "\(redirectScheme):/oauth2redirect"
    }

    private func authorizationCode(challenge: String, nonce: String) async throws -> String {
        var components = URLComponents(url: Self.authorization, resolvingAgainstBaseURL: false)!
        components.queryItems = [
            URLQueryItem(name: "client_id", value: clientID),
            URLQueryItem(name: "redirect_uri", value: redirectURI),
            URLQueryItem(name: "response_type", value: "code"),
            URLQueryItem(name: "scope", value: "openid email profile"),
            URLQueryItem(name: "code_challenge", value: challenge),
            URLQueryItem(name: "code_challenge_method", value: "S256"),
            URLQueryItem(name: "nonce", value: nonce),
        ]

        guard let url = components.url else { throw GoogleIdentityError.cancelled }
        let scheme = redirectScheme

        let callback: URL = try await withCheckedThrowingContinuation { continuation in
            let session = ASWebAuthenticationSession(url: url, callbackURLScheme: scheme) { url, error in
                if let url {
                    continuation.resume(returning: url)
                } else {
                    continuation.resume(throwing: error ?? GoogleIdentityError.cancelled)
                }
            }
            session.presentationContextProvider = self
            self.session = session
            if !session.start() {
                self.session = nil
                continuation.resume(throwing: GoogleIdentityError.cancelled)
            }
        }

        session = nil
        guard
            let code = URLComponents(url: callback, resolvingAgainstBaseURL: false)?
                .queryItems?
                .first(where: { $0.name == "code" })?
                .value
        else {
            throw GoogleIdentityError.noAuthorizationCode
        }
        return code
    }

    private func identityToken(code: String, verifier: String) async throws -> String {
        var form = URLComponents()
        form.queryItems = [
            URLQueryItem(name: "client_id", value: clientID),
            URLQueryItem(name: "code", value: code),
            URLQueryItem(name: "code_verifier", value: verifier),
            URLQueryItem(name: "grant_type", value: "authorization_code"),
            URLQueryItem(name: "redirect_uri", value: redirectURI),
        ]

        var request = URLRequest(url: Self.exchange)
        request.httpMethod = "POST"
        request.setValue("application/x-www-form-urlencoded", forHTTPHeaderField: "Content-Type")
        request.httpBody = form.percentEncodedQuery.map { Data($0.utf8) }

        let (data, response) = try await URLSession.shared.data(for: request)
        guard (response as? HTTPURLResponse)?.statusCode == 200 else {
            throw GoogleIdentityError.exchangeFailed
        }

        struct Payload: Decodable {
            let idToken: String
        }

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        return try decoder.decode(Payload.self, from: data).idToken
    }
}

extension GoogleIdentity: ASWebAuthenticationPresentationContextProviding {
    nonisolated func presentationAnchor(for session: ASWebAuthenticationSession) -> ASPresentationAnchor {
        MainActor.assumeIsolated {
            UIApplication.shared.connectedScenes
                .compactMap { $0 as? UIWindowScene }
                .flatMap(\.windows)
                .first { $0.isKeyWindow } ?? ASPresentationAnchor()
        }
    }
}
