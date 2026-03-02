// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "LocalGPTWrapper",
    platforms: [
        .iOS(.v16)
    ],
    products: [
        .library(
            name: "LocalGPTWrapper",
            targets: ["LocalGPTWrapper"]
        ),
    ],
    targets: [
        .target(
            name: "LocalGPTWrapper",
            dependencies: ["LocalGPTCore"],
            path: "Sources/LocalGPTWrapper",
            exclude: ["include"]
        ),
        .binaryTarget(
            name: "LocalGPTCore",
            path: "LocalGPTCore.xcframework"
        ),
    ]
)
