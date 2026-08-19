import AppShell
import DesignSystem
import Models
import Onboarding
import ProfileSetup
import SignIn
import SwiftUI

struct RootView: View {
    @Bindable var session: AppSession

    var body: some View {
        switch session.state {
        case .signedIn(let account):
            FirstRun(session: session, account: account)

        case .signingIn:
            ProgressView("Signing you in…")
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .background(Palette.onboardingCanvas)

        case .signedOut, .failed:
            WelcomeGate(session: session)
        }
    }
}

private struct WelcomeGate: View {
    private let session: AppSession
    @State private var model: SignInModel
    @State private var isAuthenticating = false

    init(session: AppSession) {
        self.session = session
        _model = State(
            initialValue: SignInModel(
                auth: session.auth,
                googleClientID: AppConfiguration.googleClientID,
                submit: { [weak session] outcome in
                    guard let session else { return }
                    switch outcome {
                    case .provider(let credential):
                        await session.signIn(with: credential)
                    case .verified(let pair):
                        await session.adopt(pair)
                    }
                }
            )
        )
    }

    var body: some View {
        OnboardingView { isAuthenticating = true }
            .fullScreenCover(isPresented: $isAuthenticating) {
                AuthFlowView(model: model)
            }
            .onChange(of: session.state) { _, state in
                if case .failed(let message) = state { model.report(message) }
            }
    }
}


