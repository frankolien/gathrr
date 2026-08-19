import DesignSystem
import PhotosUI
import SwiftUI

public struct ProfileSetupView: View {
    @Bindable private var model: ProfileSetupModel
    @State private var photoSelection: PhotosPickerItem?
    @FocusState private var isNaming: Bool

    public init(model: ProfileSetupModel) {
        self.model = model
    }

    public var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: Spacing.sectionGap) {
                portrait
                heading
                fields
                if case .failed(let message) = model.phase {
                    Text(message)
                        .font(Typography.footnote)
                        .foregroundStyle(Palette.statusDeclined)
                }
            }
            .padding(.horizontal, OnboardingMetrics.gutter)
            .padding(.top, Spacing.sectionGap)
        }
        .background(Palette.onboardingCanvas)
        .safeAreaInset(edge: .bottom) {
            PrimaryButton("Save Profile", isEnabled: model.canSave) {
                Task { await model.save() }
            }
            .padding(.horizontal, OnboardingMetrics.gutter)
            .padding(.bottom, Spacing.stackGap)
        }
        .task { isNaming = true }
        .onChange(of: photoSelection) { _, picked in
            guard let picked else { return }
            Task {
                if let data = try? await picked.loadTransferable(type: Data.self) {
                    model.choose(data)
                }
            }
        }
    }

    private var portrait: some View {
        PhotosPicker(selection: $photoSelection, matching: .images) {
            ZStack(alignment: .bottomTrailing) {
                photoWell
                Image(systemName: "camera.fill")
                    .font(.system(size: 13, weight: .medium))
                    .foregroundStyle(Palette.onAccent)
                    .frame(width: 30, height: 30)
                    .background(Palette.accent, in: Circle())
                    .overlay { Circle().strokeBorder(Palette.onboardingCanvas, lineWidth: 2) }
            }
        }
        .buttonStyle(.plain)
        .accessibilityLabel("Choose a profile photo")
    }

    @ViewBuilder
    private var photoWell: some View {
        if let data = model.pickedPhoto, let image = UIImage(data: data) {
            Image(uiImage: image)
                .resizable()
                .scaledToFill()
                .frame(width: 88, height: 88)
                .clipShape(Circle())
        } else {
            Circle()
                .fill(Palette.surfaceInset)
                .frame(width: 88, height: 88)
                .overlay {
                    Image(systemName: "person.fill")
                        .font(.system(size: 32, weight: .medium))
                        .foregroundStyle(Palette.textTertiary)
                }
        }
    }

    private var heading: some View {
        VStack(alignment: .leading, spacing: Spacing.unit) {
            Text("Your Profile")
                .font(Typography.onboardingHeadline)
                .foregroundStyle(Palette.textPrimary)
            Text("Introduce yourself to the people you invite.")
                .font(Typography.body)
                .foregroundStyle(Palette.textSecondary)
        }
    }

    private var fields: some View {
        VStack(alignment: .leading, spacing: Spacing.gutter) {
            VStack(alignment: .leading, spacing: Spacing.unit) {
                EyebrowText("Name")
                TextField("Your name", text: $model.name)
                    .font(Typography.body)
                    .textContentType(.name)
                    .focused($isNaming)
                    .accessibilityIdentifier("profile.name")
                    .padding(Spacing.cardPadding)
                    .background(Palette.surfaceInset)
                    .clipShape(RoundedRectangle(cornerRadius: Radius.tile, style: .continuous))
            }

            VStack(alignment: .leading, spacing: Spacing.unit) {
                HStack {
                    EyebrowText("Bio")
                    Spacer()
                    Text("\(model.remainingBioCharacters)")
                        .font(Typography.footnote)
                        .monospacedDigit()
                        .foregroundStyle(
                            model.remainingBioCharacters < 0
                                ? Palette.statusDeclined
                                : Palette.textTertiary
                        )
                }
                TextField(
                    "Share a little about yourself.",
                    text: $model.bio,
                    axis: .vertical
                )
                .font(Typography.body)
                .lineLimit(3...6)
                .accessibilityIdentifier("profile.bio")
                .padding(Spacing.cardPadding)
                .background(Palette.surfaceInset)
                .clipShape(RoundedRectangle(cornerRadius: Radius.tile, style: .continuous))
            }
        }
    }
}
