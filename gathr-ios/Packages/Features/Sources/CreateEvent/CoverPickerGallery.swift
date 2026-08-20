import DesignSystem
import SwiftUI

struct CoverPickerGallery: View {
    let category: CoverTemplateCategory
    let query: String
    let width: CGFloat
    let onSelect: (CoverTemplate) -> Void
    let onOpenCategory: (CoverTemplateCategory) -> Void

    private var columnSide: CGFloat { (width - Spacing.stackGap * 2) / 3 }
    private var pairSide: CGFloat { (width - Spacing.stackGap) / 2 }

    var body: some View {
        VStack(alignment: .leading, spacing: Spacing.stackGap) {
            if !query.isEmpty {
                grid(CoverTemplateCatalog.matching(query))
            } else if category == .suggested {
                suggested
            } else {
                grid(CoverTemplateCatalog.templates(in: category))
            }
        }
        .padding(.top, Spacing.unit)
    }

    @ViewBuilder
    private var suggested: some View {
        featured
        LazyVGrid(columns: columns(count: 2, side: pairSide), spacing: Spacing.stackGap) {
            ForEach(CoverTemplateCatalog.browsableCategories.filter { $0 != .summer }) { item in
                categoryTile(item)
            }
        }
        sectionLabel("Gathr picks")
        grid(CoverTemplateCatalog.templates(in: .suggested))
    }

    private var featured: some View {
        let templates = CoverTemplateCatalog.templates(in: .summer)
        return Button {
            onOpenCategory(.summer)
        } label: {
            ZStack {
                LinearGradient(
                    colors: [
                        templates[1].palette.baseColor,
                        templates[1].palette.blended(with: 0x00_00_00, amount: 0.28),
                    ],
                    startPoint: .topLeading,
                    endPoint: .bottomTrailing
                )

                HStack(spacing: 0) {
                    VStack(alignment: .leading, spacing: 2) {
                        Text(CoverTemplateCategory.summer.title)
                            .font(.system(size: 25, weight: .semibold, design: .rounded))
                            .foregroundStyle(.white)
                        Text("\(templates.count) covers")
                            .font(Typography.subhead)
                            .foregroundStyle(.white.opacity(0.72))
                    }
                    Spacer(minLength: Spacing.stackGap)
                    fan(templates)
                }
                .padding(Spacing.gutter)
            }
            .frame(width: width, height: width * 0.48)
            .clipShape(RoundedRectangle(cornerRadius: Radius.card, style: .continuous))
        }
        .buttonStyle(.plain)
        .accessibilityLabel("Browse summer covers")
    }

    private func fan(_ templates: [CoverTemplate]) -> some View {
        let side = width * 0.26
        return ZStack {
            ForEach(Array(templates.prefix(3).enumerated()), id: \.element.id) { index, template in
                CoverTemplateArt(template)
                    .frame(width: side, height: side * 1.24)
                    .clipShape(RoundedRectangle(cornerRadius: Radius.thumb, style: .continuous))
                    .rotationEffect(.degrees(Double(index - 1) * 9))
                    .offset(x: CGFloat(index - 1) * side * 0.44)
                    .zIndex(index == 1 ? 1 : 0)
            }
        }
        .frame(width: side * 1.9)
    }

    private func categoryTile(_ item: CoverTemplateCategory) -> some View {
        Button {
            onOpenCategory(item)
        } label: {
            ZStack(alignment: .bottom) {
                CoverTemplateArt(CoverTemplateCatalog.templates(in: item)[0])

                Text(item.title)
                    .font(Typography.headline)
                    .foregroundStyle(.white)
                    .frame(maxWidth: .infinity)
                    .frame(height: 42)
                    .background(.ultraThinMaterial)
                    .environment(\.colorScheme, .dark)
            }
            .frame(width: pairSide, height: pairSide * 0.94)
            .clipShape(RoundedRectangle(cornerRadius: Radius.card, style: .continuous))
        }
        .buttonStyle(.plain)
    }

    private func grid(_ templates: [CoverTemplate]) -> some View {
        LazyVGrid(columns: columns(count: 3, side: columnSide), spacing: Spacing.stackGap) {
            ForEach(templates) { template in
                Button {
                    onSelect(template)
                } label: {
                    CoverTemplateArt(template)
                        .frame(width: columnSide, height: columnSide)
                        .clipShape(RoundedRectangle(cornerRadius: Radius.tile, style: .continuous))
                }
                .buttonStyle(.plain)
            }
        }
    }

    private func sectionLabel(_ title: String) -> some View {
        Text(title)
            .font(Typography.subhead)
            .foregroundStyle(.white.opacity(0.6))
            .padding(.top, Spacing.unit)
    }

    private func columns(count: Int, side: CGFloat) -> [GridItem] {
        Array(repeating: GridItem(.fixed(side), spacing: Spacing.stackGap), count: count)
    }
}
