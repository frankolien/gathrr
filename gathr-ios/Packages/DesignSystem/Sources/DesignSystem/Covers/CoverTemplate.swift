import Foundation

public enum CoverCaptionStyle: Hashable, Sendable {
    case block
    case stamp
    case script
}

public struct CoverCaption: Hashable, Sendable {
    public let text: String
    public let style: CoverCaptionStyle

    public init(_ text: String, _ style: CoverCaptionStyle) {
        self.text = text
        self.style = style
    }
}

public enum CoverTemplateCategory: String, CaseIterable, Identifiable, Sendable {
    case suggested
    case summer
    case invites
    case party
    case sports
    case tech
    case business
    case school

    public var id: String { rawValue }

    public var title: String {
        switch self {
        case .suggested: "Suggested"
        case .summer: "Summer"
        case .invites: "Invites"
        case .party: "Party"
        case .sports: "Sports"
        case .tech: "Tech"
        case .business: "Business"
        case .school: "School"
        }
    }

    public var symbol: String {
        switch self {
        case .suggested: "sparkles"
        case .summer: "sun.max"
        case .invites: "envelope.open"
        case .party: "party.popper"
        case .sports: "basketball"
        case .tech: "cpu"
        case .business: "chart.bar"
        case .school: "graduationcap"
        }
    }
}

public struct CoverTemplate: Identifiable, Hashable, Sendable {
    public let id: String
    public let category: CoverTemplateCategory
    public let palette: CoverPalette
    public let motif: CoverMotif
    public let caption: CoverCaption?

    public init(
        id: String,
        category: CoverTemplateCategory,
        palette: CoverPalette,
        motif: CoverMotif,
        caption: CoverCaption?
    ) {
        self.id = id
        self.category = category
        self.palette = palette
        self.motif = motif
        self.caption = caption
    }

    static func entry(
        _ id: String,
        _ category: CoverTemplateCategory,
        _ base: UInt32,
        _ accent: UInt32,
        _ motif: CoverMotif,
        _ caption: String? = nil,
        _ style: CoverCaptionStyle = .block,
        ink: UInt32 = 0xFF_FF_FF
    ) -> CoverTemplate {
        CoverTemplate(
            id: id,
            category: category,
            palette: CoverPalette(base: base, accent: accent, ink: ink),
            motif: motif,
            caption: caption.map { CoverCaption($0, style) }
        )
    }

    var searchText: String {
        [caption?.text ?? "", category.title, id].joined(separator: " ").lowercased()
    }
}
