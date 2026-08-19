import DesignSystem
import AppShell
import SwiftUI

@main
struct GathrApp: App {
    @State private var session = AppSession(baseURL: AppConfiguration.baseURL)

    var body: some Scene {
        WindowGroup {
            RootView(session: session)
                .task {
                    if AppConfiguration.startsSignedOut { await session.signOut() }
                    await session.restore()
                }
        }
    }
}

