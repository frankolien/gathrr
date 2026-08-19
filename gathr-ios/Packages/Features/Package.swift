// swift-tools-version: 6.0
import PackageDescription

let uiDependencies: [Target.Dependency] = ["Models", "DesignSystem", "Networking", "Routing"]
let strict: [SwiftSetting] = [.swiftLanguageMode(.v6)]

