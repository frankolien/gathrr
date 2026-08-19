import Foundation
import Models
import Networking
import Testing

@testable import Profile
@testable import ProfileSetup
@testable import SignIn

actor FakeAuthService: AuthService {
    private var renamed: [String] = []
    private let failure: GathrError?

    init(failure: GathrError? = nil) {
        self.failure = failure
    }

    func namesSent() -> [String] { renamed }

    func signIn(with credential: IdentityCredential) async throws -> TokenPair {
        throw GathrError.transport("offline")
    }

    func signInForDevelopment(displayName: String) async throws -> TokenPair {
        throw GathrError.transport("offline")
    }

    func requestCode(
        channel: VerificationChannel,
        destination: String
    ) async throws -> VerificationChallenge {
        VerificationChallenge(
            destination: destination.lowercased(),
            expiresInSeconds: 600,
            developmentCode: "424242"
        )
    }

    func verifyCode(
        channel: VerificationChannel,
        destination: String,
        code: String
    ) async throws -> TokenPair {
        guard code == "424242" else { throw GathrError.transport("wrong code") }
        return TokenPair(
            userId: UUID(),
            accessToken: "access",
            refreshToken: "refresh",
            expiresInSeconds: 900
        )
    }

    func updateProfile(_ edit: ProfileEdit) async throws -> Account {
        if let failure { throw failure }
        let displayName = edit.displayName ?? "Amara Chukwu"
        renamed.append(displayName)
        return Account(id: UUID(), displayName: displayName, isGuest: false, bio: edit.bio)
    }

    func refresh(using refreshToken: String) async throws -> TokenPair {
        throw GathrError.transport("offline")
    }

    func me() async throws -> Account {
        throw GathrError.transport("offline")
    }
}

@MainActor
@Test func appleReceivesTheHashOfTheNonceAndNeverTheNonceItself() async {
    let model = SignInModel(auth: FakeAuthService(), googleClientID: nil) { _ in }

    #expect(model.hashedAppleNonce != model.appleNonce)
    #expect(model.hashedAppleNonce.count == 64)
    #expect(model.hashedAppleNonce == SignInNonce.hexHash(of: model.appleNonce))
}

@MainActor
@Test func aFailedSignInRetiresTheNonceSoTheNextAttemptCannotBeReplayed() async {
    let model = SignInModel(auth: FakeAuthService(), googleClientID: nil) { _ in }
    let first = model.appleNonce

    model.report("Apple said no.")

    #expect(model.appleNonce != first)
    #expect(model.phase == .failed("Apple said no."))
}

@MainActor
@Test func renamingIsOfferedOnlyWhenTheNameActuallyChanged() async {
    let account = Account(id: UUID(), displayName: "Amara Chukwu", isGuest: false)
    let model = ProfileModel(auth: FakeAuthService(), account: account)

    #expect(!model.canSave)

    model.draftName = "  Amara Chukwu  "
    #expect(!model.canSave)

    model.draftName = "Amara C."
    #expect(model.canSave)
}

@MainActor
@Test func savingANewNameSendsItTrimmedAndAdoptsTheServersAnswer() async {
    let service = FakeAuthService()
    let account = Account(id: UUID(), displayName: "Placeholder", isGuest: false)
    let model = ProfileModel(auth: service, account: account)

    model.draftName = "  Amara Chukwu  "
    await model.save()

    #expect(await service.namesSent() == ["Amara Chukwu"])
    #expect(model.account?.displayName == "Amara Chukwu")
    #expect(model.errorMessage == nil)
}

@MainActor
@Test func aRejectedRenameKeepsTheOldNameAndExplainsWhy() async {
    let account = Account(id: UUID(), displayName: "Amara Chukwu", isGuest: false)
    let model = ProfileModel(auth: FakeAuthService(failure: .transport("offline")), account: account)

    model.draftName = "Someone Else"
    await model.save()

    #expect(model.account?.displayName == "Amara Chukwu")
    #expect(model.errorMessage != nil)
}

@MainActor
@Test func theNextButtonWaitsForAPlausibleDestinationBeforeSendingACode() async {
    let model = SignInModel(auth: FakeAuthService(), googleClientID: nil) { _ in }

    model.choose(.email)
    model.destination = "amara"
    #expect(!model.canSubmitDestination)

    model.destination = "amara@example.com"
    #expect(model.canSubmitDestination)

    model.destination = "amara@example"
    #expect(!model.canSubmitDestination, "a domain with no dot is not reachable")
}

@MainActor
@Test func sendingACodeAdvancesToTheCodeStepAndAdoptsTheServersDestination() async {
    let model = SignInModel(auth: FakeAuthService(), googleClientID: nil) { _ in }

    model.choose(.email)
    model.destination = "Amara@Example.com"
    await model.sendCode()

    #expect(model.path.last == .code(.email))
    #expect(model.destination == "amara@example.com")
    #expect(model.revealedCode == "424242")
}

@MainActor
@Test func aWrongCodeIsClearedSoTheNextAttemptStartsFromAnEmptyField() async {
    var delivered = false
    let model = SignInModel(auth: FakeAuthService(), googleClientID: nil) { outcome in
        if case .verified = outcome { delivered = true }
    }

    model.choose(.email)
    model.destination = "amara@example.com"
    await model.sendCode()

    model.code = "111111"
    await model.verifyCode()
    #expect(!delivered)
    #expect(model.code.isEmpty)
    if case .failed = model.phase {} else { Issue.record("a rejected code should explain itself") }

    model.code = "424242"
    await model.verifyCode()
    #expect(delivered)
}


