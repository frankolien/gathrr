import DesignSystem
import SwiftUI
import UserNotifications

public struct NotificationPrimerView: View {
    @State private var isAsking = false
    private let onDecided: () -> Void

    public init(onDecided: @escaping () -> Void) {
        self.onDecided = onDecided
    }

    public var body: some View {
        VStack(alignment: .leading, spacing: Spacing.sectionGap) {
            Image(systemName: "bell.badge")
                .font(.system(size: 22, weight: .medium))
                .foregroundStyle(Palette.textSecondary)
                .frame(width: 52, height: 52)
                .background(Palette.surfaceInset)
                .clipShape(RoundedRectangle(cornerRadius: Radius.tile, style: .continuous))

            VStack(alignment: .leading, spacing: Spacing.unit) {
                Text("Get Notified")
                    .font(Typography.onboardingHeadline)
                    .foregroundStyle(Palette.textPrimary)
                Text("Know when someone RSVPs, when plans change, and the day an event lands.")
                    .font(Typography.body)
                    .foregroundStyle(Palette.textSecondary)
            }

            Spacer()

            VStack(spacing: Spacing.stackGap) {
                PrimaryButton("Turn On Notifications", isEnabled: !isAsking) {
                    Task { await ask() }
                }
                Button("Not now", action: onDecided)
                    .font(Typography.headline)
                    .foregroundStyle(Palette.accent)
                    .minimumHitTarget()
            }
        }
        .padding(.horizontal, OnboardingMetrics.gutter)
        .padding(.top, Spacing.sectionGap)
        .padding(.bottom, Spacing.gutter)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Palette.onboardingCanvas)
    }

    private func ask() async {
        isAsking = true
        _ = try? await UNUserNotificationCenter.current()
            .requestAuthorization(options: [.alert, .badge, .sound])
        onDecided()
    }
}
