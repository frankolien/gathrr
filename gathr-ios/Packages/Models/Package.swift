// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "Models",
    platforms: [.iOS(.v17)],
    products: [
        .library(name: "Models", targets: ["Models"])
    ],
    targets: [
        .target(
            name: "Models",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "ModelsTests",
            dependencies: ["Models"],
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
    ]
)
