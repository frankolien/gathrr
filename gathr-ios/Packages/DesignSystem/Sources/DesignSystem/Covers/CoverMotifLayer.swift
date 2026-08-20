import SwiftUI

struct CoverMotifLayer: View {
    let motif: CoverMotif
    let palette: CoverPalette
    let seed: String

    var body: some View {
        Canvas(opaque: false) { context, size in
            switch motif {
            case .wash:
                break
            case let .stripes(count):
                stripes(count, in: &context, size: size)
            case let .checks(count):
                checks(count, in: &context, size: size)
            case let .dots(count):
                dots(count, in: &context, size: size)
            case let .grid(count):
                grid(count, in: &context, size: size)
            case let .rays(count):
                rays(count, in: &context, size: size)
            case let .waves(count):
                waves(count, in: &context, size: size)
            case let .arcs(count):
                arcs(count, in: &context, size: size)
            case let .confetti(count):
                confetti(count, in: &context, size: size)
            case let .orbs(count):
                orbs(count, in: &context, size: size)
            }
        }
        .drawingGroup()
    }

    private var accent: Color { palette.accentColor }

    private func stripes(_ count: Int, in context: inout GraphicsContext, size: CGSize) {
        let width = size.width / CGFloat(count)
        for index in stride(from: 0, to: count, by: 2) {
            let rect = CGRect(x: CGFloat(index) * width, y: 0, width: width, height: size.height)
            context.fill(Path(rect), with: .color(accent))
        }
    }

    private func checks(_ count: Int, in context: inout GraphicsContext, size: CGSize) {
        let cell = size.width / CGFloat(count)
        let rows = Int((size.height / cell).rounded(.up))
        for row in 0..<rows {
            for column in 0..<count where (row + column).isMultiple(of: 2) {
                let rect = CGRect(x: CGFloat(column) * cell, y: CGFloat(row) * cell, width: cell, height: cell)
                context.fill(Path(rect), with: .color(accent))
            }
        }
    }

    private func dots(_ count: Int, in context: inout GraphicsContext, size: CGSize) {
        let cell = size.width / CGFloat(count)
        let rows = Int((size.height / cell).rounded(.up))
        let radius = cell * 0.22
        for row in 0..<rows {
            for column in 0..<count {
                let offset = row.isMultiple(of: 2) ? 0 : cell / 2
                let center = CGPoint(x: CGFloat(column) * cell + offset + cell / 2, y: CGFloat(row) * cell + cell / 2)
                let rect = CGRect(x: center.x - radius, y: center.y - radius, width: radius * 2, height: radius * 2)
                context.fill(Path(ellipseIn: rect), with: .color(accent))
            }
        }
    }

    private func grid(_ count: Int, in context: inout GraphicsContext, size: CGSize) {
        let cell = size.width / CGFloat(count)
        let line = max(size.width * 0.004, 0.6)
        var path = Path()
        for index in 0...count {
            let position = CGFloat(index) * cell
            path.addRect(CGRect(x: position, y: 0, width: line, height: size.height))
            path.addRect(CGRect(x: 0, y: position, width: size.width, height: line))
        }
        context.fill(path, with: .color(accent.opacity(0.75)))
    }

    private func rays(_ count: Int, in context: inout GraphicsContext, size: CGSize) {
        let center = CGPoint(x: size.width / 2, y: size.height * 0.52)
        let radius = max(size.width, size.height)
        let step = 360.0 / Double(count)
        for index in stride(from: 0, to: count, by: 2) {
            var path = Path()
            path.move(to: center)
            path.addArc(
                center: center,
                radius: radius,
                startAngle: .degrees(step * Double(index)),
                endAngle: .degrees(step * Double(index + 1)),
                clockwise: false
            )
            path.closeSubpath()
            context.fill(path, with: .color(accent))
        }
    }

    private func waves(_ count: Int, in context: inout GraphicsContext, size: CGSize) {
        let band = size.height / CGFloat(count)
        let amplitude = band * 0.55
        for index in 0..<count {
            var path = Path()
            let baseline = CGFloat(index) * band + band / 2
            path.move(to: CGPoint(x: 0, y: baseline))
            for step in stride(from: 0.0, through: Double(size.width), by: 4) {
                let progress = step / Double(size.width)
                let y = baseline + amplitude * CGFloat(sin(progress * .pi * 2 + Double(index) * 0.7))
                path.addLine(to: CGPoint(x: CGFloat(step), y: y))
            }
            path.addLine(to: CGPoint(x: size.width, y: size.height))
            path.addLine(to: CGPoint(x: 0, y: size.height))
            path.closeSubpath()
            let fade = 0.20 + 0.65 * Double(index) / Double(max(count - 1, 1))
            context.fill(path, with: .color(accent.opacity(fade)))
        }
    }

    private func arcs(_ count: Int, in context: inout GraphicsContext, size: CGSize) {
        let center = CGPoint(x: size.width * 0.5, y: size.height * 0.5)
        let step = min(size.width, size.height) / CGFloat(count * 2)
        let line = step * 0.5
        for index in 1...count {
            let radius = step * CGFloat(index)
            let rect = CGRect(
                x: center.x - radius,
                y: center.y - radius,
                width: radius * 2,
                height: radius * 2
            )
            context.stroke(Path(ellipseIn: rect), with: .color(accent.opacity(0.85)), lineWidth: line)
        }
    }

    private func confetti(_ count: Int, in context: inout GraphicsContext, size: CGSize) {
        var random = CoverRandom(seed: seed)
        let unit = size.width * 0.09
        for _ in 0..<count {
            let center = CGPoint(
                x: CGFloat(random.between(0.04, 0.96)) * size.width,
                y: CGFloat(random.between(0.04, 0.96)) * size.height
            )
            let width = unit * CGFloat(random.between(0.4, 1.1))
            let height = unit * CGFloat(random.between(0.16, 0.34))
            let angle = Angle.degrees(random.between(0, 360))
            let rect = CGRect(x: -width / 2, y: -height / 2, width: width, height: height)
            let shape = Path(roundedRect: rect, cornerRadius: height / 2)
            let tone = random.unit() > 0.5 ? accent : palette.lit
            context.drawLayer { layer in
                layer.translateBy(x: center.x, y: center.y)
                layer.rotate(by: angle)
                layer.fill(shape, with: .color(tone))
            }
        }
    }

    private func orbs(_ count: Int, in context: inout GraphicsContext, size: CGSize) {
        var random = CoverRandom(seed: seed)
        for index in 0..<count {
            let radius = size.width * CGFloat(random.between(0.16, 0.42))
            let center = CGPoint(
                x: CGFloat(random.between(0, 1)) * size.width,
                y: CGFloat(random.between(0, 1)) * size.height
            )
            let rect = CGRect(
                x: center.x - radius,
                y: center.y - radius,
                width: radius * 2,
                height: radius * 2
            )
            let tone = index.isMultiple(of: 2) ? accent : palette.lit
            context.fill(Path(ellipseIn: rect), with: .color(tone.opacity(0.55)))
        }
    }
}
