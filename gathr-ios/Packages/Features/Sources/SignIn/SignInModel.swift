import AuthenticationServices
import Foundation
import Models
import Networking
import Observation

public enum SignInOutcome: Sendable {
    case provider(IdentityCredential)
    case verified(TokenPair)
}

@MainActor
@Observable
public final class SignInModel {
    public enum Step: Hashable {
        case chooseMethod
        case destination(VerificationChannel)
        case code(VerificationChannel)
    }

    public enum Phase: Equatable, Sendable {
        case idle
        case working
        case failed(String)
    }

    public private(set) var phase: Phase = .idle
    public var path: [Step] = []
    public var destination = ""
    public var code = ""
    public private(set) var revealedCode: String?

    let googleClientID: String?
    private(set) var appleNonce = SignInNonce.random()

    private let auth: any AuthService
    private let submit: (SignInOutcome) async -> Void

    public init(
        auth: any AuthService,
        googleClientID: String?,
        submit: @escaping (SignInOutcome) async -> Void
    ) {
        self.auth = auth
        self.googleClientID = googleClientID
        self.submit = submit
    }

    var hashedAppleNonce: String {
        SignInNonce.hexHash(of: appleNonce)
    }

    var canSubmitDestination: Bool {
        guard case .destination(let channel) = path.last else { return false }
        return VerificationRules.looksReachable(destination, on: channel)
    }

    var canSubmitCode: Bool {
        code.count == VerificationRules.codeLength
    }

    func choose(_ channel: VerificationChannel) {
        destination = ""
        code = ""
        revealedCode = nil
        phase = .idle
        path = [.destination(channel)]
    }

    func sendCode() async {
        guard case .destination(let channel) = path.last, canSubmitDestination else { return }
        phase = .working
        do {
            let challenge = try await auth.requestCode(channel: channel, destination: destination)
            destination = challenge.destination
            revealedCode = challenge.developmentCode
            code = ""
            phase = .idle
            path.append(.code(channel))
        } catch {
            fail(with: error)
        }
    }

    func verifyCode() async {
        guard case .code(let channel) = path.last, canSubmitCode else { return }
        phase = .working
        do {
            let pair = try await auth.verifyCode(
                channel: channel,
                destination: destination,
                code: code
            )
            await deliver(.verified(pair))
        } catch {
            code = ""
            fail(with: error)
        }
    }

    func completeApple(_ result: Result<ASAuthorization, any Error>) async {
        switch result {
        case .success(let authorization):
            do {
                let credential = try AppleIdentity.credential(
                    from: authorization,
                    nonce: appleNonce
                )
                await deliver(.provider(credential))
            } catch {
                fail(with: error)
            }
        case .failure(let error):
            fail(with: error)
        }
    }

    func continueWithGoogle() async {
        guard let googleClientID else { return }
        phase = .working
        do {
            let credential = try await GoogleIdentity(clientID: googleClientID)
                .credential(nonce: appleNonce)
            await deliver(.provider(credential))
        } catch {
            fail(with: error)
        }
    }

    private func deliver(_ outcome: SignInOutcome) async {
        phase = .working
        await submit(outcome)
        if case .working = phase { phase = .idle }
    }

    private func fail(with error: any Error) {
        appleNonce = SignInNonce.random()
        if (error as? ASAuthorizationError)?.code == .canceled {
            phase = .idle
            return
        }
        phase = .failed((error as? GathrError)?.userFacingMessage ?? error.localizedDescription)
    }

    public func report(_ message: String) {
        appleNonce = SignInNonce.random()
        phase = .failed(message)
    }
}

enum VerificationRules {
    static let codeLength = 6

    static func looksReachable(_ destination: String, on channel: VerificationChannel) -> Bool {
        let trimmed = destination.trimmingCharacters(in: .whitespaces)
        switch channel {
        case .email:
            let parts = trimmed.split(separator: "@")
            return parts.count == 2 && parts[1].contains(".") && !parts[0].isEmpty
        }
    }
}
