import Foundation
import Models
import Testing

@testable import DesignSystem

private let lagos = "Africa/Lagos"
private let posix = Locale(identifier: "en_US_POSIX")

private func date(_ iso: String) -> Date {
    try! Date(iso, strategy: .iso8601)
}

