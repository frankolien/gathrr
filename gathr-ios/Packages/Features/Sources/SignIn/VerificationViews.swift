import DesignSystem
import Models
import SwiftUI

struct DestinationEntryView: View {
    @Bindable var model: SignInModel
    let channel: VerificationChannel
    @FocusState private var isFocused: Bool

    var body: some View {
        VStack(alignment: .leading, spacing: OnboardingMetrics.ctaGap) {
            VerificationHeader(
                symbol: "envelope",
                title: "Continue with Email",
                subtitle: "Sign in or sign up with your email."
            )

            TextField("Email Address", text: $model.destination)
                .font(Typography.body)
                .textContentType(.emailAddress)
                .keyboardType(.emailAddress)
                .textInputAutocapitalization(.never)
                .autocorrectionDisabled()
                .focused($isFocused)
                .padding(Spacing.cardPadding)
                .background(Palette.surfaceInset)
                .clipShape(RoundedRectangle(cornerRadius: Radius.tile, style: .continuous))

            FailureNote(phase: model.phase)
            Spacer(minLength: 0)

            PrimaryButton("Next", isEnabled: model.canSubmitDestination && model.phase != .working) {
                Task { await model.sendCode() }
            }
        }
        .padding(.horizontal, OnboardingMetrics.gutter)
        .padding(.top, Spacing.sectionGap)
        .padding(.bottom, Spacing.gutter)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
        .background(Palette.onboardingCanvas)
        .task { isFocused = true }
    }
}

struct VerificationCodeView: View {
    @Bindable var model: SignInModel
    @FocusState private var isFocused: Bool

    var body: some View {
        VStack(alignment: .leading, spacing: OnboardingMetrics.ctaGap) {
            VerificationHeader(
                symbol: "ellipsis.message",
                title: "Enter Code",
                subtitle: "We sent a verification code to \(model.destination)."
            )

            codeBoxes
                .contentShape(Rectangle())
                .onTapGesture { isFocused = true }
                .overlay {
                    TextField("", text: $model.code)
                        .accessibilityIdentifier("verification.code")
                        .keyboardType(.numberPad)
                        .textContentType(.oneTimeCode)
                        .focused($isFocused)
                        .opacity(0.001)
                        .onChange(of: model.code) { _, entered in
                            model.code = String(
                                entered.filter(\.isNumber).prefix(VerificationRules.codeLength)
                            )
                        }
                }

            if let revealed = model.revealedCode {
                Text("Development build — your code is \(revealed)")
                    .font(Typography.footnote)
                    .foregroundStyle(Palette.textTertiary)
            }

            FailureNote(phase: model.phase)
            Spacer(minLength: 0)

            PrimaryButton("Next", isEnabled: model.canSubmitCode && model.phase != .working) {
                Task { await model.verifyCode() }
            }
        }
        .padding(.horizontal, OnboardingMetrics.gutter)
        .padding(.top, Spacing.sectionGap)
        .padding(.bottom, Spacing.gutter)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
        .background(Palette.onboardingCanvas)
        .task { isFocused = true }
    }

    private var codeBoxes: some View {
        HStack(spacing: Spacing.stackGap) {
            ForEach(0..<VerificationRules.codeLength, id: \.self) { slot in
                Text(character(at: slot))
                    .font(Typography.titleM.monospacedDigit())
                    .foregroundStyle(Palette.textPrimary)
                    .frame(maxWidth: .infinity, minHeight: Spacing.rowHeight)
                    .background(Palette.surfaceInset)
                    .clipShape(RoundedRectangle(cornerRadius: Radius.thumb, style: .continuous))
            }
        }
        .accessibilityElement(children: .ignore)
        .accessibilityLabel("Verification code")
        .accessibilityValue(model.code)
    }

    private func character(at slot: Int) -> String {
        let digits = Array(model.code)
        return slot < digits.count ? String(digits[slot]) : "–"
    }
}

