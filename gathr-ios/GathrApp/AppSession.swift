import Foundation
import Models
import Networking
import Observation

@MainActor
@Observable
final class AppSession {
    enum State: Equatable {
        case signedOut
        case signingIn
        case signedIn(Account)
        case failed(String)
    }

    private let store = SessionStore()
    private let client: APIClient
    let auth: any AuthService
    let events: any EventService
    let media: any MediaService
    let activity: any ActivityService
    private(set) var state: State = .signedOut

    init(baseURL: URL) {
        let client = APIClient(
            baseURL: baseURL,
            transport: URLSessionTransport(),
            tokens: store
        )
        self.client = client
        auth = LiveAuthService(client: client)
        events = LiveEventService(client: client)
        media = LiveMediaService(client: client, transport: URLSessionTransport())
        activity = LiveActivityService(client: client)
    }

    var account: Account? {
        if case .signedIn(let account) = state { return account }
        return nil
    }

    func restore() async {
        guard await store.current() != nil else { return }
        state = .signingIn
        do {
            state = .signedIn(try await auth.me())
        } catch {
            await store.clear()
            state = .signedOut
        }
    }

    func adopt(_ pair: TokenPair) async {
        await establish { pair }
    }

    func signIn(with credential: IdentityCredential) async {
        await establish { try await auth.signIn(with: credential) }
    }

    func signIn(displayName: String) async {
        await establish { try await auth.signInForDevelopment(displayName: displayName) }
    }

    private func establish(_ authenticate: () async throws -> TokenPair) async {
        state = .signingIn
        do {
            await store.save(try await authenticate())
            state = .signedIn(try await auth.me())
        } catch let error as GathrError {
            state = .failed(error.userFacingMessage)
        } catch {
            state = .failed("Could not sign in.")
        }
    }

    func adopt(_ account: Account) {
        state = .signedIn(account)
    }

    func signOut() async {
        await store.clear()
        state = .signedOut
    }
}
