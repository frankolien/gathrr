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

enum AppConfiguration {
    static var baseURL: URL {
        let configured = Bundle.main.object(forInfoDictionaryKey: "GathrAPIBaseURL") as? String
        return URL(string: configured ?? "") ?? URL(string: "http://127.0.0.1:8080")!
    }

    static var googleClientID: String? {
        let configured = Bundle.main.object(forInfoDictionaryKey: "GathrGoogleClientID") as? String
        return configured.flatMap { $0.isEmpty ? nil : $0 }
    }

    static var allowsDevelopmentSignIn: Bool {
        Bundle.main.object(forInfoDictionaryKey: "GathrAllowDevSignIn") as? Bool ?? false
    }

    static var startsSignedOut: Bool {
        allowsDevelopmentSignIn
            && ProcessInfo.processInfo.arguments.contains("-gathr-signed-out")
    }
}
