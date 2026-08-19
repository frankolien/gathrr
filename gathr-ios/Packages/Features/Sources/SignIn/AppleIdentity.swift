import AuthenticationServices
import Foundation
import Models

enum AppleIdentityError: LocalizedError {
    case noIdentityToken

    var errorDescription: String? {
        switch self {
        case .noIdentityToken: "Apple didn't return a usable sign-in token."
        }
    }
}

enum AppleIdentity {
    static func credential(
        from authorization: ASAuthorization,
        nonce: String
    ) throws -> IdentityCredential {
        guard
            let apple = authorization.credential as? ASAuthorizationAppleIDCredential,
            let data = apple.identityToken,
            let token = String(data: data, encoding: .utf8)
        else {
            throw AppleIdentityError.noIdentityToken
        }

        return IdentityCredential(
            provider: .apple,
            idToken: token,
            nonce: nonce,
            displayName: name(from: apple.fullName)
        )
    }

    private static func name(from components: PersonNameComponents?) -> String? {
        guard let components else { return nil }
        let formatted = PersonNameComponentsFormatter.localizedString(
            from: components,
            style: .default
        )
        return formatted.isEmpty ? nil : formatted
    }
}
