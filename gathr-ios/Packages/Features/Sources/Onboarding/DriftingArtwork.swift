import DesignSystem
import SwiftUI

public struct DriftingArtwork: View {
    @Environment(\.accessibilityReduceMotion) private var reduceMotion

    private struct Piece: Identifiable {
        let id: String
        let aspectRatio: CGFloat
        let tilt: Angle
        let bobPhase: Double
        let carriesItsOwnChrome: Bool
    }

    static var artworkNames: [String] { pieces.map(\.id) }

    private static let pieces = [
        Piece(id: "PosterSummerParty", aspectRatio: 759.0 / 1130.0, tilt: .zero, bobPhase: 0.0, carriesItsOwnChrome: true),
        Piece(id: "PhotoCrowd", aspectRatio: 0.68, tilt: .degrees(5), bobPhase: 1.1, carriesItsOwnChrome: false),
        Piece(id: "PosterGroupTherapy", aspectRatio: 727.0 / 1130.0, tilt: .zero, bobPhase: 2.3, carriesItsOwnChrome: true),
        Piece(id: "PhotoCourtyard", aspectRatio: 0.68, tilt: .degrees(-6), bobPhase: 3.6, carriesItsOwnChrome: false),
        Piece(id: "PosterWeekendHangouts", aspectRatio: 778.0 / 1130.0, tilt: .zero, bobPhase: 4.8, carriesItsOwnChrome: true),
    ]

    public init() {}

    public var body: some View {
        GeometryReader { proxy in
            ribbon(across: proxy.size.width)
        }
        .frame(height: OnboardingMetrics.posterHeight)
        .accessibilityHidden(true)
    }

    @ViewBuilder
    private func ribbon(across width: CGFloat) -> some View {
        if reduceMotion {
            row(travel: 0, at: 0, across: width)
        } else {
            TimelineView(.animation) { context in
                let elapsed = context.date.timeIntervalSinceReferenceDate
                let progress = (elapsed / OnboardingMetrics.driftPeriod)
                    .truncatingRemainder(dividingBy: 1)
                row(travel: cycleWidth * progress, at: elapsed, across: width)
            }
        }
    }

    private func row(travel: CGFloat, at elapsed: TimeInterval, across width: CGFloat) -> some View {
        HStack(spacing: OnboardingMetrics.posterGap) {
            ForEach(0..<repetitionsCovering(width), id: \.self) { _ in
                piece(at: elapsed)
            }
        }
        .fixedSize()
        .offset(x: -travel)
        .frame(width: width, alignment: .leading)
    }

    private func repetitionsCovering(_ width: CGFloat) -> Int {
        guard cycleWidth > 0 else { return 2 }
        return max(2, Int((width / cycleWidth).rounded(.up)) + 1)
    }

    private func piece(at elapsed: TimeInterval) -> some View {
        ForEach(Self.pieces) { piece in
            artwork(piece)
                .rotationEffect(piece.tilt)
                .offset(y: bob(piece, at: elapsed))
        }
    }

    @ViewBuilder
    private func artwork(_ piece: Piece) -> some View {
        let width = OnboardingMetrics.posterHeight * piece.aspectRatio
        let height = OnboardingMetrics.posterHeight

        if piece.carriesItsOwnChrome {
            Image(piece.id, bundle: .module)
                .resizable()
                .frame(width: width, height: height)
        } else {
            Image(piece.id, bundle: .module)
                .resizable()
                .scaledToFill()
                .frame(width: width, height: height)
                .clipped()
                .overlay(Palette.photoScrim)
                .clipShape(RoundedRectangle(cornerRadius: Radius.hero, style: .continuous))
                .cardElevation()
        }
    }

    private func bob(_ piece: Piece, at elapsed: TimeInterval) -> CGFloat {
        guard !reduceMotion else { return 0 }
        let angle = elapsed * OnboardingMetrics.bobRate + piece.bobPhase
        return OnboardingMetrics.bobHeight * CGFloat(sin(angle))
    }

    private var cycleWidth: CGFloat {
        let widths = Self.pieces.reduce(0) { $0 + OnboardingMetrics.posterHeight * $1.aspectRatio }
        return widths + OnboardingMetrics.posterGap * CGFloat(Self.pieces.count)
    }
}
