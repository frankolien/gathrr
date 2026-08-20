import DesignSystem
import PhotosUI
import SwiftUI

public struct CoverPickerView: View {
    @Binding private var cover: EventCover
    @State private var category: CoverTemplateCategory = .suggested
    @State private var query = ""
    @State private var isSearching = false
    @State private var photoItem: PhotosPickerItem?
    @Environment(\.dismiss) private var dismiss

    public init(cover: Binding<EventCover>) {
        _cover = cover
    }

    public var body: some View {
        ZStack(alignment: .bottom) {
            backdrop

            VStack(spacing: 0) {
                titleBar
                if isSearching {
                    searchField
                }
                categoryStrip
                gallery
            }

            libraryButton
        }
        .presentationBackground(.clear)
        .presentationDetents([.large])
        .presentationCornerRadius(Radius.sheet + 4)
    }

    private var backdrop: some View {
        ZStack {
            cover.palette.deepest
            LinearGradient(
                colors: [cover.palette.deep.opacity(0.6), .clear],
                startPoint: .top,
                endPoint: .center
            )
        }
        .ignoresSafeArea()
    }

    private var titleBar: some View {
        HStack {
            circleButton("xmark") { dismiss() }
            Spacer()
            Text("Add Cover Image")
                .font(.system(size: 21, weight: .semibold, design: .rounded))
                .foregroundStyle(.white)
            Spacer()
            circleButton(isSearching ? "magnifyingglass.circle.fill" : "magnifyingglass") {
                withAnimation(.smooth(duration: 0.28)) {
                    isSearching.toggle()
                    if !isSearching { query = "" }
                }
            }
        }
        .padding(.horizontal, Spacing.gutter)
        .padding(.top, Spacing.gutter)
        .padding(.bottom, Spacing.stackGap)
    }

    private func circleButton(_ symbol: String, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            Image(systemName: symbol)
                .font(.system(size: 17, weight: .semibold))
                .foregroundStyle(.white)
                .frame(width: 42, height: 42)
                .background(.white.opacity(0.12), in: Circle())
        }
        .buttonStyle(.plain)
        .accessibilityLabel(symbol == "xmark" ? "Close" : "Search covers")
    }

    private var searchField: some View {
        HStack(spacing: Spacing.stackGap) {
            Image(systemName: "magnifyingglass")
                .font(.system(size: 15, weight: .semibold))
                .foregroundStyle(.white.opacity(0.6))
            TextField("Search covers", text: $query)
                .font(Typography.titleS)
                .foregroundStyle(.white)
                .tint(.white)
                .submitLabel(.search)
        }
        .padding(.horizontal, Spacing.gutter)
        .frame(height: 46)
        .background(.white.opacity(0.12), in: Capsule())
        .padding(.horizontal, Spacing.gutter)
        .padding(.bottom, Spacing.stackGap)
    }

    private var categoryStrip: some View {
        ScrollView(.horizontal) {
            HStack(spacing: Spacing.unit) {
                ForEach(CoverTemplateCategory.allCases) { item in
                    Button {
                        withAnimation(.smooth(duration: 0.3)) { category = item }
                    } label: {
                        VStack(spacing: 5) {
                            Image(systemName: item.symbol)
                                .font(.system(size: 20, weight: .regular))
                            Text(item.title)
                                .font(Typography.footnote)
                        }
                        .foregroundStyle(category == item ? .white : .white.opacity(0.55))
                        .frame(width: 78, height: 62)
                        .background {
                            if category == item {
                                RoundedRectangle(cornerRadius: Radius.card, style: .continuous)
                                    .fill(.white.opacity(0.13))
                            }
                        }
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(.horizontal, Spacing.gutter - Spacing.unit)
        }
        .scrollIndicators(.hidden)
        .padding(.bottom, Spacing.stackGap)
    }

    private var gallery: some View {
        GeometryReader { proxy in
            let width = proxy.size.width - Spacing.gutter * 2
            ScrollView {
                CoverPickerGallery(
                    category: category,
                    query: isSearching ? query : "",
                    width: width,
                    onSelect: select,
                    onOpenCategory: { opened in
                        withAnimation(.smooth(duration: 0.3)) { category = opened }
                    }
                )
                .padding(.horizontal, Spacing.gutter)
                .padding(.bottom, Spacing.tabBarClearance)
            }
            .scrollIndicators(.hidden)
        }
    }

    private var libraryButton: some View {
        PhotosPicker(selection: $photoItem, matching: .images, photoLibrary: .shared()) {
            Text("Choose From Library")
                .font(Typography.titleS)
                .foregroundStyle(Palette.textPrimary)
                .frame(maxWidth: .infinity)
                .frame(height: 54)
                .background(.white.opacity(0.94), in: Capsule())
        }
        .buttonStyle(.plain)
        .padding(.horizontal, Spacing.gutter)
        .padding(.bottom, Spacing.gutter)
        .onChange(of: photoItem) { _, item in
            guard let item else { return }
            Task { await adopt(item) }
        }
    }

    private func select(_ template: CoverTemplate) {
        cover = .template(template)
        dismiss()
    }

    private func adopt(_ item: PhotosPickerItem) async {
        guard
            let data = try? await item.loadTransferable(type: Data.self),
            let picked = EventCover.photo(data)
        else { return }
        cover = picked
        dismiss()
    }
}
