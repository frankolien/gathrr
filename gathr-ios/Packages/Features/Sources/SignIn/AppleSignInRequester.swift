import AuthenticationServices
import UIKit

@MainActor
final class AppleSignInRequester: NSObject {
    private var controller: ASAuthorizationController?
    private var pending: CheckedContinuation<Result<ASAuthorization, any Error>, Never>?

    func request(hashedNonce: String) async -> Result<ASAuthorization, any Error> {
        let request = ASAuthorizationAppleIDProvider().createRequest()
        request.requestedScopes = [.fullName, .email]
        request.nonce = hashedNonce

        let controller = ASAuthorizationController(authorizationRequests: [request])
        controller.delegate = self
        controller.presentationContextProvider = self
        self.controller = controller

        return await withCheckedContinuation { continuation in
            pending = continuation
            controller.performRequests()
        }
    }

    private func settle(_ outcome: Result<ASAuthorization, any Error>) {
        pending?.resume(returning: outcome)
        pending = nil
        controller = nil
    }
}

extension AppleSignInRequester: ASAuthorizationControllerDelegate {
    nonisolated func authorizationController(
        controller: ASAuthorizationController,
        didCompleteWithAuthorization authorization: ASAuthorization
    ) {
        MainActor.assumeIsolated { settle(.success(authorization)) }
    }

    nonisolated func authorizationController(
        controller: ASAuthorizationController,
        didCompleteWithError error: any Error
    ) {
        MainActor.assumeIsolated { settle(.failure(error)) }
    }
}

extension AppleSignInRequester: ASAuthorizationControllerPresentationContextProviding {
    nonisolated func presentationAnchor(
        for controller: ASAuthorizationController
    ) -> ASPresentationAnchor {
        MainActor.assumeIsolated {
            UIApplication.shared.connectedScenes
                .compactMap { $0 as? UIWindowScene }
                .flatMap(\.windows)
                .first { $0.isKeyWindow } ?? ASPresentationAnchor()
        }
    }
}
