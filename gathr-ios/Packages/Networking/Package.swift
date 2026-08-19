// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "Networking",
    platforms: [.iOS(.v17)],
    products: [
        .library(name: "Networking", targets: ["Networking"])
    ],
    dependencies: [
        .package(path: "../Models")
    ],
    targets: [
        .target(
            name: "Networking",
            dependencies: ["Models"],
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "NetworkingTests",
            dependencies: ["Networking"],
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
    ]
)
