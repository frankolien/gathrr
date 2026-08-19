import DesignSystem
import Models
import SwiftUI

struct RSVPSheet: View {
    @Bindable var model: EventDetailModel
    @Environment(\.dismiss) private var dismiss

    private let options: [RSVPStatus] = [.going, .maybe, .declined]

    var body: some View {
        VStack(alignment: .leading, spacing: Spacing.stackGap) {
            Text("Your RSVP")
                .font(Typography.titleS)
                .foregroundStyle(Palette.textPrimary)
                .padding(.top, Spacing.gutter)

            ForEach(options, id: \.self) { option in
                Button {
                    Task { await respond(option) }
                } label: {
                    optionRow(option)
                }
                .buttonStyle(.plain)
                .disabled(model.isSubmitting)
            }

            if model.maxPlusOnes > 0 {
                Stepper(
                    "Bringing \(model.plusOnes) \(model.plusOnes == 1 ? "guest" : "guests")",
                    value: $model.plusOnes,
                    in: 0...model.maxPlusOnes
                )
                .font(Typography.subhead)
                .foregroundStyle(Palette.textPrimary)
                .padding(Spacing.cardPadding)
                .background(Palette.surfaceInset)
                .clipShape(RoundedRectangle(cornerRadius: Radius.tile, style: .continuous))
            }

            if model.isFull {
                Text("This event is full. Join the waitlist and the host will let you know if a spot opens up.")
                    .font(Typography.footnote)
                    .foregroundStyle(Palette.textSecondary)
                PrimaryButton("Join the waitlist", shape: .rounded) {
                    Task { await model.joinWaitlist(); dismiss() }
                }
            } else if let error = model.submissionError {
                Text(error)
                    .font(Typography.footnote)
                    .foregroundStyle(Palette.statusDeclined)
            }

            Spacer(minLength: 0)
        }
        .padding(.horizontal, Spacing.gutter)
        .background(Palette.canvas)
    }

    private func optionRow(_ option: RSVPStatus) -> some View {
        HStack {
            Image(systemName: option.symbol)
                .font(.system(size: 18))
                .foregroundStyle(option.tint)
            Text(option.label)
                .font(Typography.headline)
                .foregroundStyle(Palette.textPrimary)
            Spacer()
            if model.myRSVP?.status == option {
                Image(systemName: "checkmark")
                    .font(.system(size: 15, weight: .semibold))
                    .foregroundStyle(Palette.accent)
            }
        }
        .padding(Spacing.cardPadding)
        .background(Palette.surface)
        .clipShape(RoundedRectangle(cornerRadius: Radius.tile, style: .continuous))
    }

    private func respond(_ option: RSVPStatus) async {
        await model.submit(option)
        if !model.isFull && model.submissionError == nil {
            dismiss()
        }
    }
}
