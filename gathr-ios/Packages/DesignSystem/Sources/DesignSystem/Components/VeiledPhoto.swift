import SwiftUI

public struct VeiledPhoto: View {
    private let image: Image

    public init(_ image: Image) {
        self.image = image
    }

    public var body: some View {
        GeometryReader { proxy in
            ZStack {
                filled(proxy.size)
                    .blur(radius: OnboardingMetrics.veilBlur, opaque: true)
                    .clipped()
                filled(proxy.size)
                    .clipped()
                    .mask(Palette.photoClarityMask)
                Palette.photoVeil
            }
            .frame(width: proxy.size.width, height: proxy.size.height)
        }
        .accessibilityHidden(true)
    }

    private func filled(_ size: CGSize) -> some View {
        image
            .resizable()
            .scaledToFill()
            .frame(width: size.width, height: size.height)
    }
}
