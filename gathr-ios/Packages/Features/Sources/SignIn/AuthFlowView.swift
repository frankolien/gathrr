import DesignSystem
import SwiftUI

public struct AuthFlowView: View {
    @Bindable private var model: SignInModel

    public init(model: SignInModel) {
        self.model = model
    }

    public var body: some View {
        NavigationStack(path: $model.path) {
            SignUpView(model: model)
                .navigationDestination(for: SignInModel.Step.self) { step in
                    switch step {
                    case .chooseMethod:
                        SignUpView(model: model)
                    case .destination(let channel):
                        DestinationEntryView(model: model, channel: channel)
                    case .code:
                        VerificationCodeView(model: model)
                    }
                }
        }
        .tint(Palette.accent)
    }
}
