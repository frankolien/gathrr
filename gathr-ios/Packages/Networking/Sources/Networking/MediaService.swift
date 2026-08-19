import Foundation
import Models

public protocol MediaService: Sendable {
    func uploadAvatar(_ imageData: Data) async throws -> UUID
}

