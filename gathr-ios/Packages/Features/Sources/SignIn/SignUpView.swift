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
                    .font(Typography.subhead)
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
        VStack(spacing: OnboardingMetrics.providerRowGap) {
            ProviderButton("Continue with Email", emblem: .plain) {
                model.choose(.email)
            }

            OrDivider()

            HStack(spacing: OnboardingMetrics.providerRowGap) {
                ProviderButton(
                    "Continue with Apple",
                    emblem: .apple,
                    presentation: .iconOnly
                ) {
                    Task { await model.continueWithApple() }
                }

                if model.googleClientID != nil {
                    ProviderButton(
                        "Continue with Google",
                        emblem: .google,
                        presentation: .iconOnly
                    ) {
                        Task { await model.continueWithGoogle() }
                    }
                }
            }

            if case .failed(let message) = model.phase {
                Text(message)
                    .font(Typography.chip)
                    .foregroundStyle(Palette.statusDeclined)
                    .multilineTextAlignment(.center)
            }

            Text("By continuing you agree to the Gathr terms and privacy policy.")
                .font(Typography.chip)
                .foregroundStyle(Palette.textTertiary)
                .multilineTextAlignment(.center)
                .padding(.top, Spacing.unit)
        }
    }
}
