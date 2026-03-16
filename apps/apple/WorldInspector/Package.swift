// swift-tools-version: 5.10
import PackageDescription

let package = Package(
    name: "WorldInspector",
    platforms: [
        .iOS(.v17),
        .macOS(.v14),
    ],
    products: [
        .library(name: "WorldInspector", targets: ["WorldInspector"]),
    ],
    targets: [
        .target(
            name: "WorldInspector",
            path: "Sources/WorldInspector"
        ),
    ]
)
