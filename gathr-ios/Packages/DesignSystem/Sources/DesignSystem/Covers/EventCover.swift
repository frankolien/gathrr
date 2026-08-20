import SwiftUI
import UIKit

public enum EventCover: Hashable, Sendable {
    case template(CoverTemplate)
    case photo(data: Data, palette: CoverPalette)

    public static let signature = EventCover.template(CoverTemplateCatalog.signature)

    public static func photo(_ data: Data) -> EventCover? {
        guard let image = UIImage(data: data), let palette = CoverPalette(averaging: image) else { return nil }
        return .photo(data: data, palette: palette)
    }

    public var palette: CoverPalette {
        switch self {
        case let .template(template): template.palette
        case let .photo(_, palette): palette
        }
    }

    public var templateID: String? {
        switch self {
        case let .template(template): template.id
        case .photo: nil
        }
    }
}

public struct EventCoverArt: View {
    private let cover: EventCover

    public init(_ cover: EventCover) {
        self.cover = cover
    }

    @ViewBuilder
    public var body: some View {
        switch cover {
        case let .template(template):
            CoverTemplateArt(template)
        case let .photo(data, palette):
            if let image = UIImage(data: data) {
                Image(uiImage: image)
                    .resizable()
                    .scaledToFill()
            } else {
                palette.baseColor
            }
        }
    }
}

public struct CoverBackdrop: View {
    private let cover: EventCover

    public init(_ cover: EventCover) {
        self.cover = cover
    }

    public var body: some View {
        let palette = cover.palette
        ZStack {
            LinearGradient(
                colors: [
                    palette.blended(with: 0x00_00_00, amount: 0.22),
                    palette.deep,
                    palette.deepest,
                ],
                startPoint: .top,
                endPoint: .bottom
            )

            EventCoverArt(cover)
                .scaleEffect(1.8)
                .blur(radius: 72)
                .opacity(0.42)
                .saturation(1.2)

            LinearGradient(
                colors: [.clear, palette.deepest.opacity(0.78)],
                startPoint: .center,
                endPoint: .bottom
            )
        }
        .ignoresSafeArea()
    }
}
