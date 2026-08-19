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

