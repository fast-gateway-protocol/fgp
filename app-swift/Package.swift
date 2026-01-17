// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "FGPManager",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .executable(name: "FGPManager", targets: ["FGPManager"])
    ],
    dependencies: [
        // Snapshot testing for visual regression tests
        .package(url: "https://github.com/pointfreeco/swift-snapshot-testing", from: "1.15.0")
    ],
    targets: [
        .executableTarget(
            name: "FGPManager",
            resources: [
                .copy("Resources")
            ]
        ),
        .testTarget(
            name: "FGPManagerTests",
            dependencies: [
                "FGPManager",
                .product(name: "SnapshotTesting", package: "swift-snapshot-testing")
            ],
            resources: [
                .copy("__Snapshots__")
            ]
        )
    ]
)
