import DesignSystem
import Models
import Networking
import SwiftUI

public struct CreateEventView: View {
    @State private var model: CreateEventModel
    @Environment(\.dismiss) private var dismiss
    private let onPublished: (Event) -> Void

    public init(model: CreateEventModel, onPublished: @escaping (Event) -> Void) {
        _model = State(initialValue: model)
        self.onPublished = onPublished
    }

    public var body: some View {
        NavigationStack {
            Form {
                Section {
                    TextField("What's the occasion?", text: $model.draft.title)
                        .font(Typography.headline)
                        .onChange(of: model.draft.title) { model.beginEditing() }
                    Picker("Category", selection: $model.draft.category) {
                        ForEach(EventCategory.allCases, id: \.self) { category in
                            Label(category.style.label, systemImage: category.style.symbol)
                                .tag(category)
                        }
                    }
                }

                Section("When and where") {
                    DatePicker("Starts", selection: $model.draft.startsAt)
                    TextField(
                        "Location",
                        text: Binding(
                            get: { model.draft.locationName ?? "" },
                            set: { model.draft.locationName = $0.isEmpty ? nil : $0 }
                        )
                    )
                }

                Section("Details") {
                    TextField(
                        "Description",
                        text: Binding(
                            get: { model.draft.description ?? "" },
                            set: { model.draft.description = $0.isEmpty ? nil : $0 }
                        ),
                        axis: .vertical
                    )
                    .lineLimit(3...6)

                    Toggle("Limit capacity", isOn: capacityToggle)
                    if let capacity = model.draft.capacity {
                        Stepper(
                            "\(capacity) guests",
                            value: Binding(
                                get: { capacity },
                                set: { model.draft.capacity = $0 }
                            ),
                            in: 1...500
                        )
                    }
                }

                if let error = model.errorMessage {
                    Section {
                        Text(error)
                            .font(Typography.footnote)
                            .foregroundStyle(Palette.statusDeclined)
                    }
                }
            }
            .navigationTitle("New Event")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") { dismiss() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button("Publish") {
                        Task {
                            await model.publish()
                            if let event = model.published {
                                onPublished(event)
                                dismiss()
                            }
                        }
                    }
                    .disabled(!model.canPublish)
                }
            }
        }
    }

    private var capacityToggle: Binding<Bool> {
        Binding(
            get: { model.draft.capacity != nil },
            set: { model.draft.capacity = $0 ? 20 : nil }
        )
    }
}
