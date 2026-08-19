import DesignSystem
import Models
import SwiftUI

public struct ProfileView: View {
    @State private var model: ProfileModel
    private let onSignOut: () -> Void

    public init(model: ProfileModel, onSignOut: @escaping () -> Void) {
        _model = State(initialValue: model)
        self.onSignOut = onSignOut
    }

    public var body: some View {
        ScrollView {
            VStack(spacing: Spacing.sectionGap) {
                if let account = model.account {
                    identity(account)
                    nameEditor
                } else {
                    ContentUnavailableView(
                        "Not signed in",
                        systemImage: "person.crop.circle.badge.questionmark"
                    )
                }

                SecondaryButton("Sign out", action: onSignOut)
            }
            .padding(.horizontal, Spacing.gutter)
        }
        .background(Palette.canvas)
    }

    private func identity(_ account: Account) -> some View {
        VStack(spacing: Spacing.stackGap) {
            Avatar(name: account.displayName, ringColor: Palette.canvas)
                .scaleEffect(2)
                .frame(height: 72)
            Text(account.displayName)
                .font(Typography.titleS)
                .foregroundStyle(Palette.textPrimary)
            if account.isGuest {
                StatusChip(status: .invited)
            }
        }
        .padding(.top, Spacing.sectionGap)
    }

    private var nameEditor: some View {
        VStack(alignment: .leading, spacing: Spacing.stackGap) {
            SectionHeader("Your name")
            Text("Guests see this on every invite you host.")
                .font(Typography.footnote)
                .foregroundStyle(Palette.textSecondary)

            TextField("Amara Chukwu", text: $model.draftName)
                .font(Typography.body)
                .textContentType(.name)
                .padding(Spacing.cardPadding)
                .background(Palette.surfaceInset)
                .clipShape(RoundedRectangle(cornerRadius: Radius.tile, style: .continuous))

            if let errorMessage = model.errorMessage {
                Text(errorMessage)
                    .font(Typography.footnote)
                    .foregroundStyle(Palette.statusDeclined)
            }

            PrimaryButton("Save name", shape: .rounded, isEnabled: model.canSave) {
                Task { await model.save() }
            }
        }
    }
}
