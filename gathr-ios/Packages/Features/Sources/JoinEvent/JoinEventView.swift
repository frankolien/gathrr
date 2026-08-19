import DesignSystem
import Models
import SwiftUI

public struct JoinEventView: View {
    @State private var model: JoinEventModel
    @Environment(\.dismiss) private var dismiss
    @FocusState private var isCodeFocused: Bool
    private let onResolved: (UUID) -> Void

    public init(model: JoinEventModel, onResolved: @escaping (UUID) -> Void) {
        _model = State(initialValue: model)
        self.onResolved = onResolved
    }

    public var body: some View {
        NavigationStack {
            VStack(alignment: .leading, spacing: Spacing.sectionGap) {
                VStack(alignment: .leading, spacing: Spacing.stackGap) {
                    Text("Enter the invite code")
                        .font(Typography.titleS)
                        .foregroundStyle(Palette.textPrimary)
                    Text("It's ten characters, and case doesn't matter.")
                        .font(Typography.footnote)
                        .foregroundStyle(Palette.textSecondary)
                }

                TextField("ABCDEFGHJK", text: $model.code)
                    .font(Typography.numeral)
                    .textInputAutocapitalization(.characters)
                    .autocorrectionDisabled()
                    .focused($isCodeFocused)
                    .padding(Spacing.cardPadding)
                    .background(Palette.surfaceInset)
                    .clipShape(RoundedRectangle(cornerRadius: Radius.tile, style: .continuous))

                if let error = model.errorMessage {
                    Text(error)
                        .font(Typography.footnote)
                        .foregroundStyle(Palette.statusDeclined)
                }

                if let invite = model.resolved {
                    resolvedCard(invite)
                }

                Spacer()

                PrimaryButton("Find event", isEnabled: model.canSubmit) {
                    Task { await model.resolve() }
                }
            }
            .padding(Spacing.gutter)
            .background(Palette.canvas)
            .navigationTitle("Join Event")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") { dismiss() }
                }
            }
            .onAppear { isCodeFocused = true }
        }
    }

    private func resolvedCard(_ invite: PublicInvite) -> some View {
        Button {
            onResolved(invite.eventId)
            dismiss()
        } label: {
            VStack(alignment: .leading, spacing: 6) {
                CategoryChip(invite.category, treatment: .tinted)
                Text(invite.title)
                    .font(Typography.headline)
                    .foregroundStyle(Palette.textPrimary)
                Text(EventFormatting.longWhen(invite.startsAt, timezone: invite.timezone))
                    .font(Typography.footnote)
                    .foregroundStyle(Palette.textSecondary)
                Text("\(invite.goingGuests) going · hosted by \(invite.hostFirstName)")
                    .font(Typography.footnote)
                    .foregroundStyle(Palette.textSecondary)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(Spacing.cardPadding)
            .background(Palette.surface)
            .clipShape(RoundedRectangle(cornerRadius: Radius.card, style: .continuous))
        }
        .buttonStyle(.plain)
    }
}
