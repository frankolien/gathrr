import SwiftUI

public struct GradientHeaderScreen<Header: View, Content: View>: View {
    private let header: Header
    private let content: Content

    @State private var headerHeight: CGFloat = 0

    public init(
        @ViewBuilder header: () -> Header,
        @ViewBuilder content: () -> Content
    ) {
        self.header = header()
        self.content = content()
    }

    public var body: some View {
        GeometryReader { viewport in
            ScrollView {
                VStack(spacing: 0) {
                    header.background { measure }
                    content
                        .frame(
                            maxWidth: .infinity,
                            minHeight: max(0, viewport.size.height - headerHeight),
                            alignment: .topLeading
                        )
                        .background(Palette.canvas)
                        .clipShape(
                            UnevenRoundedRectangle(
                                topLeadingRadius: Radius.sheet,
                                topTrailingRadius: Radius.sheet,
                                style: .continuous
                            )
                        )
                }
            }
            .scrollIndicators(.hidden)
        }
        .ignoresSafeArea(edges: .bottom)
        .background(Palette.headerGradient.ignoresSafeArea())
    }

    private var measure: some View {
        GeometryReader { proxy in
            Color.clear
                .onAppear { headerHeight = proxy.size.height }
                .onChange(of: proxy.size.height) { _, height in headerHeight = height }
        }
    }
}
