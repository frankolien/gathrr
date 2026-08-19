import Testing
import UIKit

@testable import Onboarding

@Test func everyOnboardingLineCarriesDistinctCopy() {
    let headlines = OnboardingCopy.all.map(\.headline)
    #expect(headlines.count == Set(headlines).count)
    #expect(OnboardingCopy.all.allSatisfy { !$0.headline.isEmpty && !$0.subhead.isEmpty })
}

@Test func everyOnboardingArtworkResolvesInTheModuleBundle() {
    for name in DriftingArtwork.artworkNames {
        #expect(UIImage(named: name, in: .module, with: nil) != nil, "missing artwork: \(name)")
    }
}
