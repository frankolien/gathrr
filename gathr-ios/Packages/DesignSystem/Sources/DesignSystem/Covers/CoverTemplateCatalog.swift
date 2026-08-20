import Foundation

public enum CoverTemplateCatalog {
    public static let all: [CoverTemplate] =
        suggestedSet + summerSet + invitesSet + partySet + sportsSet + techSet + businessSet + schoolSet

    public static let signature: CoverTemplate = suggestedSet[0]

    public static func templates(in category: CoverTemplateCategory) -> [CoverTemplate] {
        switch category {
        case .suggested: suggestedSet
        case .summer: summerSet
        case .invites: invitesSet
        case .party: partySet
        case .sports: sportsSet
        case .tech: techSet
        case .business: businessSet
        case .school: schoolSet
        }
    }

    public static func matching(_ query: String) -> [CoverTemplate] {
        let needle = query.trimmingCharacters(in: .whitespaces).lowercased()
        guard !needle.isEmpty else { return all }
        return all.filter { $0.searchText.contains(needle) }
    }

    public static var browsableCategories: [CoverTemplateCategory] {
        CoverTemplateCategory.allCases.filter { $0 != .suggested }
    }
}
