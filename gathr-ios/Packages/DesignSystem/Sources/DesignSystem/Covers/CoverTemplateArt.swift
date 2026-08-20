import SwiftUI

public struct CoverTemplateArt: View {
    private let template: CoverTemplate

    public init(_ template: CoverTemplate) {
        self.template = template
    }

    public var body: some View {
        GeometryReader { proxy in
            let side = min(proxy.size.width, proxy.size.height)
            ZStack {
                LinearGradient(
                    colors: [template.palette.lit, template.palette.baseColor, template.palette.deep],
                    startPoint: .topLeading,
                    endPoint: .bottomTrailing
                )

                CoverMotifLayer(
                    motif: template.motif,
                    palette: template.palette,
                    seed: template.id
                )

                if let caption = template.caption {
                    CoverCaptionLabel(caption: caption, palette: template.palette, side: side)
                }
            }
            .frame(width: proxy.size.width, height: proxy.size.height)
        }
        .clipped()
        .accessibilityLabel(template.caption?.text ?? template.category.title)
    }
}

struct CoverCaptionLabel: View {
    let caption: CoverCaption
    let palette: CoverPalette
    let side: CGFloat

    @ViewBuilder
    var body: some View {
        switch caption.style {
        case .block:
            block
        case .stamp:
            stamp
        case .script:
            script
        }
    }

    private var block: some View {
        Text(caption.text.uppercased())
            .font(.system(size: side * 0.145, weight: .black, design: .rounded))
            .tracking(-side * 0.003)
            .lineSpacing(-side * 0.01)
            .multilineTextAlignment(.center)
            .foregroundStyle(palette.inkColor)
            .shadow(color: .black.opacity(0.22), radius: side * 0.02, y: side * 0.006)
            .minimumScaleFactor(0.4)
            .padding(.horizontal, side * 0.1)
    }

    private var stamp: some View {
        Text(caption.text.uppercased())
            .font(.system(size: side * 0.072, weight: .semibold, design: .serif))
            .tracking(side * 0.014)
            .multilineTextAlignment(.center)
            .foregroundStyle(palette.deepest)
            .minimumScaleFactor(0.4)
            .padding(.horizontal, side * 0.06)
            .padding(.vertical, side * 0.042)
            .background(
                Color.white.opacity(0.94),
                in: RoundedRectangle(cornerRadius: side * 0.022, style: .continuous)
            )
            .padding(.horizontal, side * 0.13)
    }

    private var script: some View {
        Text(caption.text)
            .font(.system(size: side * 0.125, weight: .regular, design: .serif))
            .italic()
            .multilineTextAlignment(.center)
            .foregroundStyle(palette.inkColor)
            .shadow(color: .black.opacity(0.24), radius: side * 0.022, y: side * 0.006)
            .minimumScaleFactor(0.4)
            .padding(.horizontal, side * 0.12)
    }
}
