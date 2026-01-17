import AppKit
import SwiftUI
import XCTest
import SnapshotTesting

// MARK: - SwiftUI View Rendering

/// Renders a SwiftUI view to an NSImage for snapshot testing
@MainActor
func renderView<V: View>(_ view: V, size: CGSize) -> NSImage {
    let hostingView = NSHostingView(rootView: view)
    hostingView.frame = CGRect(origin: .zero, size: size)

    // Force layout
    hostingView.layoutSubtreeIfNeeded()

    // Create bitmap representation
    guard let bitmapRep = hostingView.bitmapImageRepForCachingDisplay(in: hostingView.bounds) else {
        fatalError("Failed to create bitmap representation")
    }
    hostingView.cacheDisplay(in: hostingView.bounds, to: bitmapRep)

    let image = NSImage(size: size)
    image.addRepresentation(bitmapRep)
    return image
}

/// Saves an NSImage to a file
func saveImage(_ image: NSImage, to path: String) throws {
    guard let tiffData = image.tiffRepresentation,
          let bitmapRep = NSBitmapImageRep(data: tiffData),
          let pngData = bitmapRep.representation(using: .png, properties: [:]) else {
        throw NSError(domain: "SnapshotTest", code: 1, userInfo: [
            NSLocalizedDescriptionKey: "Failed to convert image to PNG"
        ])
    }
    try pngData.write(to: URL(fileURLWithPath: path))
}

// MARK: - Artifacts Directory

/// Returns the path to the test artifacts directory
func artifactsDirectory() -> URL {
    let projectRoot = URL(fileURLWithPath: #file)
        .deletingLastPathComponent()  // SnapshotTestUtils.swift
        .deletingLastPathComponent()  // FGPManagerTests
        .deletingLastPathComponent()  // Tests

    let artifactsDir = projectRoot.appendingPathComponent("TestArtifacts")
    try? FileManager.default.createDirectory(at: artifactsDir, withIntermediateDirectories: true)
    return artifactsDir
}

/// Saves a screenshot artifact with timestamp
@MainActor
func saveArtifact<V: View>(_ view: V, name: String, size: CGSize) throws -> URL {
    let image = renderView(view, size: size)
    let timestamp = ISO8601DateFormatter().string(from: Date())
        .replacingOccurrences(of: ":", with: "-")
    let filename = "\(name)_\(timestamp).png"
    let path = artifactsDirectory().appendingPathComponent(filename)
    try saveImage(image, to: path.path)
    return path
}

// MARK: - Snapshot Strategy for SwiftUI Views

extension Snapshotting where Value: View, Format == NSImage {
    /// Creates a snapshot strategy for SwiftUI views at a specific size
    static func swiftUIImage(size: CGSize) -> Snapshotting {
        return Snapshotting<NSImage, NSImage>.image.pullback { view in
            // Run on main actor for SwiftUI
            var image: NSImage!
            let semaphore = DispatchSemaphore(value: 0)
            DispatchQueue.main.async {
                let hostingView = NSHostingView(rootView: view)
                hostingView.frame = CGRect(origin: .zero, size: size)
                hostingView.layoutSubtreeIfNeeded()

                if let bitmapRep = hostingView.bitmapImageRepForCachingDisplay(in: hostingView.bounds) {
                    hostingView.cacheDisplay(in: hostingView.bounds, to: bitmapRep)
                    image = NSImage(size: size)
                    image.addRepresentation(bitmapRep)
                } else {
                    image = NSImage(size: size)
                }
                semaphore.signal()
            }
            semaphore.wait()
            return image
        }
    }
}

// MARK: - Test Fixtures

@testable import FGPManager

/// Creates mock DaemonInfo for testing different states
enum MockDaemon {
    static func running(name: String = "test-daemon", version: String = "1.0.0", uptime: UInt64 = 3600, hasManifest: Bool = true) -> DaemonInfo {
        DaemonInfo(
            name: name,
            status: "running",
            version: version,
            uptimeSeconds: uptime,
            isRunning: true,
            hasManifest: hasManifest
        )
    }

    static func stopped(name: String = "test-daemon", hasManifest: Bool = true) -> DaemonInfo {
        DaemonInfo(
            name: name,
            status: "stopped",
            version: nil,
            uptimeSeconds: nil,
            isRunning: false,
            hasManifest: hasManifest
        )
    }

    static func notResponding(name: String = "test-daemon", hasManifest: Bool = false) -> DaemonInfo {
        DaemonInfo(
            name: name,
            status: "not_responding",
            version: nil,
            uptimeSeconds: nil,
            isRunning: false,
            hasManifest: hasManifest
        )
    }

    static func healthy(name: String = "test-daemon", version: String = "1.0.0", uptime: UInt64 = 7200, hasManifest: Bool = true) -> DaemonInfo {
        DaemonInfo(
            name: name,
            status: "healthy",
            version: version,
            uptimeSeconds: uptime,
            isRunning: true,
            hasManifest: hasManifest
        )
    }
}

/// Creates mock AppState for testing views
@MainActor
func createMockAppState(daemons: [DaemonInfo] = [], error: String? = nil) -> AppState {
    let appState = AppState()
    appState.daemonStore.daemons = daemons
    appState.daemonStore.error = error
    return appState
}

// MARK: - Test Report Generation

/// Generates an HTML report of all test artifacts
func generateTestReport(artifactsDir: URL) throws {
    let fileManager = FileManager.default
    let files = try fileManager.contentsOfDirectory(at: artifactsDir, includingPropertiesForKeys: [.creationDateKey])
        .filter { $0.pathExtension == "png" }
        .sorted { $0.lastPathComponent < $1.lastPathComponent }

    var html = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>FGP Manager Visual Test Report</title>
        <style>
            body { font-family: -apple-system, BlinkMacSystemFont, sans-serif; padding: 20px; background: #1a1a1a; color: #fff; }
            h1 { color: #fff; }
            .artifact { margin: 20px 0; padding: 20px; background: #2a2a2a; border-radius: 8px; }
            .artifact img { max-width: 100%; border: 1px solid #444; border-radius: 4px; }
            .artifact h3 { margin-top: 0; color: #4a9eff; }
            .timestamp { color: #888; font-size: 12px; }
            .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(400px, 1fr)); gap: 20px; }
        </style>
    </head>
    <body>
        <h1>FGP Manager Visual Test Report</h1>
        <p class="timestamp">Generated: \(Date())</p>
        <div class="grid">
    """

    for file in files {
        let name = file.deletingPathExtension().lastPathComponent
        html += """
            <div class="artifact">
                <h3>\(name)</h3>
                <img src="\(file.lastPathComponent)" alt="\(name)">
            </div>
        """
    }

    html += """
        </div>
    </body>
    </html>
    """

    let reportPath = artifactsDir.appendingPathComponent("report.html")
    try html.write(to: reportPath, atomically: true, encoding: .utf8)
    print("📊 Test report generated: \(reportPath.path)")
}
