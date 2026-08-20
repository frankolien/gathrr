import DesignSystem
import Models
import Networking
import SwiftUI

public struct CreateEventView: View {
    private enum DatePickerTarget: String, Identifiable {
        case start
        case end

        var id: String { rawValue }
    }

    @State private var model: CreateEventModel
    @State private var endDate = Date().addingTimeInterval(90 * 60)
    @State private var isApprovalRequired = false
    @State private var isPublic = true
    @State private var isShowingRestoreDraft = true
    @State private var datePickerTarget: DatePickerTarget?
    @State private var isShowingCoverPicker = false
    @State private var cover = EventCover.signature
    @Environment(\.dismiss) private var dismiss

    private let onPublished: (Event) -> Void

    public init(model: CreateEventModel, onPublished: @escaping (Event) -> Void) {
        _model = State(initialValue: model)
        self.onPublished = onPublished
    }

    public var body: some View {
        ZStack {
            composerBackground

            ScrollView {
                VStack(spacing: Spacing.sectionGap) {
                    header

                    if isShowingRestoreDraft {
                        restoreDraftPill
                    }

                    artwork
                    eventDetails
                    ticketing
                    options

                    if let error = model.errorMessage {
                        Text(error)
                            .font(Typography.footnote)
                            .foregroundStyle(.white)
                            .padding(Spacing.cardPadding)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .composerSurface()
                    }
                }
                .frame(maxWidth: .infinity)
                .padding(.horizontal, Spacing.gutter)
                .padding(.top, Spacing.unit)
                .padding(.bottom, Spacing.sectionGap * 2)
            }
            .scrollIndicators(.hidden)
        }
        .interactiveDismissDisabled(model.isPublishing)
        .sheet(item: $datePickerTarget) { target in
            datePickerSheet(target)
        }
        .sheet(isPresented: $isShowingCoverPicker) {
            CoverPickerView(cover: $cover)
        }
    }

    private var restoreDraftPill: some View {
        HStack(spacing: 6) {
            Image(systemName: "arrow.counterclockwise")
                .font(.system(size: 12, weight: .semibold))
            Text("Restore Draft?")
                .font(Typography.headline)
            Button {
                withAnimation(.smooth(duration: 0.24)) { isShowingRestoreDraft = false }
            } label: {
                Image(systemName: "xmark.circle.fill")
                    .font(.system(size: 14, weight: .semibold))
            }
            .buttonStyle(.plain)
            .accessibilityLabel("Dismiss draft")
        }
        .foregroundStyle(.white.opacity(0.78))
        .padding(.horizontal, 14)
        .padding(.vertical, 9)
        .background(.white.opacity(0.13), in: Capsule())
    }

    private var header: some View {
        HStack {
            Button(action: dismiss.callAsFunction) {
                ZStack(alignment: .bottomTrailing) {
                    Image(systemName: "person.fill")
                        .font(.system(size: 18, weight: .medium))
                        .foregroundStyle(.white.opacity(0.88))
                        .frame(width: 46, height: 46)
                        .background(.white.opacity(0.14), in: Circle())

                    Image(systemName: "chevron.down")
                        .font(.system(size: 11, weight: .bold))
                        .foregroundStyle(.white)
                        .frame(width: 24, height: 24)
                        .background(cover.palette.deepest, in: Circle())
                        .overlay { Circle().strokeBorder(.white.opacity(0.18), lineWidth: 1) }
                        .offset(x: 4, y: 3)
                }
            }
            .buttonStyle(.plain)
            .accessibilityLabel("Close")

            Spacer()

            Text("Create Event")
                .font(.system(size: 22, weight: .medium, design: .rounded))
                .foregroundStyle(.white)

            Spacer()

            Button {
                publish()
            } label: {
                Image(systemName: "checkmark")
                    .font(.system(size: 20, weight: .semibold))
                    .foregroundStyle(model.canPublish ? .white : .white.opacity(0.42))
                    .frame(width: 46, height: 46)
                    .background(.white.opacity(0.10), in: Circle())
                    .overlay { Circle().strokeBorder(.white.opacity(0.16), lineWidth: 1) }
            }
            .buttonStyle(.plain)
            .disabled(!model.canPublish)
            .accessibilityLabel("Publish event")
        }
        .overlay(alignment: .top) {
            Capsule()
                .fill(.white.opacity(0.16))
                .frame(width: 64, height: 5)
                .offset(y: -4)
        }
        .padding(.top, Spacing.unit)
    }

    private var artwork: some View {
        EventCoverArt(cover)
            .frame(
                width: OnboardingMetrics.composerArtworkSize,
                height: OnboardingMetrics.composerArtworkSize
            )
            .clipShape(RoundedRectangle(cornerRadius: Radius.sheet, style: .continuous))
            .overlay(alignment: .bottomTrailing) {
                Image(systemName: "photo.badge.plus")
                    .font(.system(size: 19, weight: .medium))
                    .foregroundStyle(.white)
                    .frame(width: 44, height: 44)
                    .background(.black.opacity(0.38), in: Circle())
                    .overlay { Circle().strokeBorder(.white.opacity(0.24), lineWidth: 1) }
                    .padding(12)
            }
            .contentShape(RoundedRectangle(cornerRadius: Radius.sheet, style: .continuous))
            .onTapGesture { isShowingCoverPicker = true }
            .accessibilityElement()
            .accessibilityAddTraits(.isButton)
            .accessibilityLabel("Choose event artwork")
            .cardElevation()
    }

    private var eventDetails: some View {
        VStack(spacing: Spacing.stackGap) {
            TextField("", text: $model.draft.title)
                .font(.system(size: 22, weight: .semibold, design: .rounded))
                .foregroundStyle(.white)
                .tint(.white)
                .lineLimit(1)
                .composerPlaceholder("Event Name", isShowing: model.draft.title.isEmpty)
                .accessibilityLabel("Event Name")
                .padding(.horizontal, Spacing.gutter)
                .frame(maxWidth: .infinity)
                .frame(height: 60)
                .composerSurface(cornerRadius: 30)
                .onChange(of: model.draft.title) { model.beginEditing() }

            timeCard

            ComposerTextField(
                symbol: "mappin.and.ellipse",
                placeholder: "Choose Location",
                text: Binding(
                    get: { model.draft.locationName ?? "" },
                    set: { model.draft.locationName = $0.isEmpty ? nil : $0 }
                )
            )

            ComposerTextField(
                symbol: "text.alignleft",
                placeholder: "Add Description",
                text: Binding(
                    get: { model.draft.description ?? "" },
                    set: { model.draft.description = $0.isEmpty ? nil : $0 }
                )
            )
        }
    }

    private var timeCard: some View {
        VStack(spacing: 0) {
            ComposerTimeRow(
                title: "Start",
                value: formattedStartDate,
                isStart: true
            ) {
                datePickerTarget = .start
            }

            Divider()
                .overlay(.white.opacity(0.16))
                .padding(.leading, 64)

            ComposerTimeRow(
                title: "End",
                value: formattedEndDate,
                isStart: false
            ) {
                datePickerTarget = .end
            }
        }
        .composerSurface(cornerRadius: Radius.sheet)
    }

    private var ticketing: some View {
        VStack(alignment: .leading, spacing: Spacing.stackGap) {
            sectionTitle("Ticketing")
            VStack(spacing: 0) {
                Toggle(isOn: $isApprovalRequired) {
                    ComposerRowLabel(symbol: "lock", title: "Require Approval")
                }
                .tint(.white)
                .padding(Spacing.gutter)

                Divider()
                    .overlay(.white.opacity(0.16))
                    .padding(.leading, 64)

                HStack {
                    ComposerRowLabel(symbol: "dollarsign", title: "Price")
                    Spacer()
                    Text("Free")
                        .font(Typography.titleS)
                        .foregroundStyle(.white)
                    Image(systemName: "chevron.right")
                        .font(.system(size: 13, weight: .semibold))
                        .foregroundStyle(.white.opacity(0.62))
                }
                .padding(Spacing.gutter)
            }
            .composerSurface(cornerRadius: Radius.sheet)
        }
    }

    private var options: some View {
        VStack(alignment: .leading, spacing: Spacing.stackGap) {
            sectionTitle("Options")
            VStack(spacing: 0) {
                Menu {
                    Button("Public") { isPublic = true }
                    Button("Private") { isPublic = false }
                } label: {
                    ComposerChoiceRow(
                        symbol: "globe",
                        title: "Visibility",
                        value: isPublic ? "Public" : "Private"
                    )
                }

                Divider()
                    .overlay(.white.opacity(0.16))
                    .padding(.leading, 64)

                Menu {
                    Button("Unlimited") { model.draft.capacity = nil }
                    Button("20 guests") { model.draft.capacity = 20 }
                    Button("50 guests") { model.draft.capacity = 50 }
                } label: {
                    ComposerChoiceRow(
                        symbol: "person.2",
                        title: "Capacity",
                        value: model.draft.capacity.map { "\($0) guests" } ?? "Unlimited"
                    )
                }
            }
            .composerSurface(cornerRadius: Radius.sheet)
        }
    }

    private var composerBackground: some View {
        CoverBackdrop(cover)
    }

    private var formattedStartDate: String {
        let date = model.draft.startsAt.formatted(
            Date.FormatStyle.dateTime.weekday(.abbreviated).month(.abbreviated).day()
        )
        let time = model.draft.startsAt.formatted(
            Date.FormatStyle.dateTime.hour(.defaultDigits(amPM: .abbreviated)).minute()
        )
        return "\(date) at \(time)"
    }

    private var formattedEndDate: String {
        endDate.formatted(date: .omitted, time: .shortened)
    }

    private func sectionTitle(_ title: String) -> some View {
        Text(title)
            .font(Typography.titleS)
            .foregroundStyle(.white.opacity(0.74))
            .frame(maxWidth: .infinity, alignment: .leading)
    }

    private func datePickerSheet(_ target: DatePickerTarget) -> some View {
        NavigationStack {
            DatePicker(
                target == .start ? "Start" : "End",
                selection: target == .start ? $model.draft.startsAt : $endDate,
                displayedComponents: [.date, .hourAndMinute]
            )
            .datePickerStyle(.graphical)
            .padding(Spacing.gutter)
            .navigationTitle(target == .start ? "Start time" : "End time")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("Done") { datePickerTarget = nil }
                }
            }
        }
        .presentationDetents([.medium])
    }

    private func publish() {
        Task {
            await model.publish()
            if let event = model.published {
                onPublished(event)
                dismiss()
            }
        }
    }
}

private struct ComposerTextField: View {
    let symbol: String
    let placeholder: String
    @Binding var text: String

    var body: some View {
        HStack(spacing: Spacing.stackGap) {
            Image(systemName: symbol)
                .font(.system(size: 19, weight: .medium))
                .foregroundStyle(.white.opacity(0.7))
                .frame(width: 26)
            TextField("", text: $text)
                .font(Typography.titleS)
                .foregroundStyle(.white)
                .tint(.white)
                .composerPlaceholder(placeholder, isShowing: text.isEmpty)
                .accessibilityLabel(placeholder)
        }
        .padding(.horizontal, Spacing.gutter)
        .frame(height: 60)
        .composerSurface(cornerRadius: Radius.sheet)
    }
}

private struct ComposerTimeRow: View {
    let title: String
    let value: String
    let isStart: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: Spacing.stackGap) {
                Image(systemName: isStart ? "circle.fill" : "circle")
                    .font(.system(size: 16, weight: .semibold))
                    .foregroundStyle(.white.opacity(0.64))
                    .frame(width: 28)
                Text(title)
                    .font(Typography.titleS)
                    .foregroundStyle(.white.opacity(0.72))
                Spacer()
                Text(value)
                    .font(Typography.titleS)
                    .foregroundStyle(.white)
                    .lineLimit(1)
            }
            .padding(.horizontal, Spacing.gutter)
            .frame(height: 72)
        }
        .buttonStyle(.plain)
    }
}

private struct ComposerRowLabel: View {
    let symbol: String
    let title: String

    var body: some View {
        HStack(spacing: Spacing.stackGap) {
            Image(systemName: symbol)
                .font(.system(size: 20, weight: .medium))
                .frame(width: 28)
            Text(title)
                .font(Typography.titleS)
        }
        .foregroundStyle(.white.opacity(0.74))
    }
}

private struct ComposerChoiceRow: View {
    let symbol: String
    let title: String
    let value: String

    var body: some View {
        HStack(spacing: Spacing.stackGap) {
            ComposerRowLabel(symbol: symbol, title: title)
            Spacer()
            Text(value)
                .font(Typography.titleS)
                .foregroundStyle(.white)
            Image(systemName: "chevron.up.chevron.down")
                .font(.system(size: 12, weight: .semibold))
                .foregroundStyle(.white.opacity(0.62))
        }
        .padding(Spacing.gutter)
    }
}

private extension View {
    func composerPlaceholder(_ text: String, isShowing: Bool) -> some View {
        overlay(alignment: .leading) {
            if isShowing {
                Text(text)
                    .foregroundStyle(.white.opacity(0.55))
                    .allowsHitTesting(false)
            }
        }
    }

    func composerSurface(cornerRadius: CGFloat = Radius.sheet) -> some View {
        background(.white.opacity(0.14), in: RoundedRectangle(cornerRadius: cornerRadius, style: .continuous))
            .overlay {
                RoundedRectangle(cornerRadius: cornerRadius, style: .continuous)
                    .strokeBorder(.white.opacity(0.08), lineWidth: 1)
            }
    }
}
