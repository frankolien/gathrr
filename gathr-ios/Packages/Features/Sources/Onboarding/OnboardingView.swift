import DesignSystem
import SwiftUI

public struct OnboardingView: View {
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @State private var shownCopy = 0
    private let onGetStarted: () -> Void

    public init(onGetStarted: @escaping () -> Void) {
        self.onGetStarted = onGetStarted
    }

    public var body: some View {
        VStack(spacing: 0) {
            DriftingArtwork()
                .padding(.top, OnboardingMetrics.collageTopInset)
            copy
                .padding(.top, OnboardingMetrics.artworkToCopyGap)
            Spacer(minLength: OnboardingMetrics.copyGap)
            PrimaryButton("Get Started", action: onGetStarted)
                .padding(.horizontal, OnboardingMetrics.gutter)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Palette.onboardingCanvas)
        .task { await cycleCopy() }
    }

    private var copy: some View {
        VStack(spacing: OnboardingMetrics.copyGap) {
            Text(OnboardingCopy.all[shownCopy].headline)
                .font(Typography.onboardingHeadline)
                .foregroundStyle(Palette.textPrimary)
            Text(OnboardingCopy.all[shownCopy].subhead)
                .font(Typography.body)
                .foregroundStyle(Palette.textSecondary)
        }
        .multilineTextAlignment(.center)
        .frame(maxWidth: .infinity)
        .padding(.horizontal, OnboardingMetrics.copyGutter)
        .id(shownCopy)
        .transition(.opacity.combined(with: .blurReplace))
        .accessibilityElement(children: .combine)
    }

    private func cycleCopy() async {
        guard !reduceMotion else { return }
        while !Task.isCancelled {
            try? await Task.sleep(for: .seconds(OnboardingMetrics.copyDwell))
            guard !Task.isCancelled else { return }
            withAnimation(.smooth(duration: 0.55)) {
                shownCopy = (shownCopy + 1) % OnboardingCopy.all.count
            }
        }
    }
}
