import Foundation
import Models
import Networking
import Observation

@MainActor
@Observable
public final class ProfileModel {
    public private(set) var account: Account?
    public private(set) var isSaving = false
    public private(set) var errorMessage: String?
    public var draftName: String

    private let auth: any AuthService

    public init(auth: any AuthService, account: Account?) {
        self.auth = auth
        self.account = account
        draftName = account?.displayName ?? ""
    }

    public var canSave: Bool {
        !isSaving && !trimmedName.isEmpty && trimmedName != account?.displayName
    }

    public func save() async {
        guard canSave else { return }
        isSaving = true
        errorMessage = nil
        do {
            account = try await auth.updateProfile(ProfileEdit(displayName: trimmedName))
        } catch let error as GathrError {
            errorMessage = error.userFacingMessage
        } catch {
            errorMessage = "Could not save your name."
        }
        isSaving = false
    }

    private var trimmedName: String {
        draftName.trimmingCharacters(in: .whitespacesAndNewlines)
    }
}
