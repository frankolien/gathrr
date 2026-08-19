// swift-tools-version: 6.0
import PackageDescription

let uiDependencies: [Target.Dependency] = ["Models", "DesignSystem", "Networking", "Routing"]
let strict: [SwiftSetting] = [.swiftLanguageMode(.v6)]

let package = Package(
    name: "Features",
    platforms: [.iOS(.v17)],
    products: [
        .library(name: "Features", targets: ["AppShell", "Onboarding", "ProfileSetup", "SignIn"])
    ],
    dependencies: [
        .package(path: "../Models"),
        .package(path: "../DesignSystem"),
        .package(path: "../Networking"),
    ],
    targets: [
        .target(name: "Routing", dependencies: ["Models", "DesignSystem"], swiftSettings: strict),
        .target(name: "Home", dependencies: uiDependencies, swiftSettings: strict),
        .target(name: "Calendar", dependencies: uiDependencies + ["Home"], swiftSettings: strict),
        .target(name: "EventDetail", dependencies: uiDependencies, swiftSettings: strict),
        .target(
            name: "Onboarding",
            dependencies: uiDependencies,
            resources: [.process("Resources")],
            swiftSettings: strict
        ),
        .target(name: "CreateEvent", dependencies: uiDependencies, swiftSettings: strict),
        .target(name: "Notifications", dependencies: uiDependencies, swiftSettings: strict),
        .target(name: "JoinEvent", dependencies: uiDependencies, swiftSettings: strict),
        .target(name: "Profile", dependencies: uiDependencies, swiftSettings: strict),
        .target(name: "ProfileSetup", dependencies: uiDependencies, swiftSettings: strict),
        .target(name: "SignIn", dependencies: uiDependencies + ["Onboarding"], swiftSettings: strict),
        .target(
            name: "AppShell",
            dependencies: uiDependencies + [
                "Home", "Calendar", "EventDetail", "CreateEvent", "JoinEvent", "Notifications", "Profile",
            ],
            swiftSettings: strict
        ),
        .testTarget(
            name: "FeaturesTests",
            dependencies: ["Home", "EventDetail", "CreateEvent", "JoinEvent", "Notifications", "Onboarding", "Profile", "ProfileSetup", "Routing", "SignIn"],
            swiftSettings: strict
        ),
    ]
)
