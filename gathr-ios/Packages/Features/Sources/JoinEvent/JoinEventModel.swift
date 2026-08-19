import Foundation
import Models
import Networking
import Observation

public enum InviteCodeInput {
    public static let length = 10
    private static let alphabet = Set("0123456789ABCDEFGHJKMNPQRSTVWXYZ")

    public static func normalize(_ raw: String) -> String {
        String(
            raw
                .uppercased()
                .compactMap { character -> Character? in
                    switch character {
                    case "I", "L": "1"
                    case "O": "0"
                    case "-", " ", "_": nil
                    default: alphabet.contains(character) ? character : nil
                    }
                }
                .prefix(length)
        )
    }

    public static func isComplete(_ normalized: String) -> Bool {
        normalized.count == length
    }
}

@MainActor
@Observable
public final class JoinEventModel {
    private let service: any EventService

    public private(set) var resolved: PublicInvite?
    public private(set) var errorMessage: String?
    public private(set) var isResolving = false

    public var code: String = "" {
        didSet {
            let normalized = InviteCodeInput.normalize(code)
            if normalized != code { code = normalized }
        }
    }

    public init(service: any EventService) {
        self.service = service
    }

    public var canSubmit: Bool {
        InviteCodeInput.isComplete(code) && !isResolving
    }

    public func resolve() async {
        guard canSubmit else { return }
        isResolving = true
        errorMessage = nil
        defer { isResolving = false }

        do {
            resolved = try await service.resolveInvite(code: code)
        } catch let error as GathrError {
            errorMessage = message(for: error)
        } catch {
            errorMessage = "Something went wrong."
        }
    }

    private func message(for error: GathrError) -> String {
        switch error.code {
        case .inviteInvalid, .notFound: "That code doesn't match an event."
        case .inviteExpired: "This invite has expired — ask the host for a new one."
        case .inviteExhausted: "This invite has already been used up."
        case .eventCancelled: "That event was cancelled."
        default: error.userFacingMessage
        }
    }
}
