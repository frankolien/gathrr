import AuthenticationServices
import DesignSystem
import Onboarding
import SwiftUI

struct SignUpView: View {
    @Bindable var model: SignInModel

    var body: some View {
        VStack(spacing: 0) {
            DriftingArtwork()
                .padding(.top, OnboardingMetrics.signUpArtworkInset)

            VStack(spacing: OnboardingMetrics.copyGap) {
                Text("Meet Gathr.")
                    .font(Typography.onboardingHeadline)
                    .foregroundStyle(Palette.textPrimary)
                Text("The easiest way to invite your people — and know who's coming.")
                    .font(Typography.body)
                    .foregroundStyle(Palette.textSecondary)
            }
            .multilineTextAlignment(.center)
            .padding(.horizontal, OnboardingMetrics.copyGutter)
            .padding(.top, OnboardingMetrics.artworkToCopyGap)

            Spacer(minLength: OnboardingMetrics.copyGap)

            doors
                .padding(.horizontal, OnboardingMetrics.gutter)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Palette.onboardingCanvas)
        .navigationBarBackButtonHidden()
    }

    private var doors: some View {
        VStack(spacing: Spacing.stackGap) {
            appleDoor

            if model.googleClientID != nil {
                ProviderButton("Continue with Google", emblem: .google) {
                    Task { await model.continueWithGoogle() }
                }
            }

            ProviderButton("Continue with Email", emblem: .symbol("envelope")) {
                model.choose(.email)
            }

            if case .failed(let message) = model.phase {
                Text(message)
                    .font(Typography.footnote)
                    .foregroundStyle(Palette.statusDeclined)
                    .multilineTextAlignment(.center)
            }

            Text("By continuing you agree to the Gathr terms and privacy policy.")
                .font(Typography.footnote)
                .foregroundStyle(Palette.textTertiary)
                .multilineTextAlignment(.center)
                .padding(.top, Spacing.unit)
        }
    }

    private var appleDoor: some View {
        SignInWithAppleButton(.continue) { request in
            request.requestedScopes = [.fullName, .email]
            request.nonce = model.hashedAppleNonce
        } onCompletion: { result in
            Task { await model.completeApple(result) }
        }
        .signInWithAppleButtonStyle(.black)
        .frame(height: OnboardingMetrics.providerButtonHeight)
        .clipShape(Capsule())
        .accessibilityLabel("Continue with Apple")
    }
}
