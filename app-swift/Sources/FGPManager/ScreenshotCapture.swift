import AppKit
import SwiftUI

// MARK: - Screenshot Capture Utility

/// Captures screenshots of SwiftUI views for visual testing
enum ScreenshotCapture {

    /// Captures a screenshot of a SwiftUI view and saves it to a file
    @MainActor
    static func capture<V: View>(_ view: V, size: CGSize, filename: String, directory: URL? = nil) throws -> URL {
        let hostingView = NSHostingView(rootView: view)
        hostingView.frame = CGRect(origin: .zero, size: size)
        hostingView.wantsLayer = true
        hostingView.layer?.backgroundColor = NSColor.windowBackgroundColor.cgColor
        hostingView.layoutSubtreeIfNeeded()

        guard let bitmapRep = hostingView.bitmapImageRepForCachingDisplay(in: hostingView.bounds) else {
            throw ScreenshotError.failedToCreateBitmap
        }
        hostingView.cacheDisplay(in: hostingView.bounds, to: bitmapRep)

        guard let pngData = bitmapRep.representation(using: .png, properties: [:]) else {
            throw ScreenshotError.failedToConvertToPNG
        }

        let dir = directory ?? defaultArtifactsDirectory()
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)

        let filePath = dir.appendingPathComponent("\(filename).png")
        try pngData.write(to: filePath)
        return filePath
    }

    static func defaultArtifactsDirectory() -> URL {
        FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent(".fgp")
            .appendingPathComponent("test-artifacts")
    }

    enum ScreenshotError: Error, LocalizedError {
        case failedToCreateBitmap
        case failedToConvertToPNG

        var errorDescription: String? {
            switch self {
            case .failedToCreateBitmap: return "Failed to create bitmap representation"
            case .failedToConvertToPNG: return "Failed to convert image to PNG"
            }
        }
    }
}

// MARK: - Visual Test Runner

/// Runs visual tests and generates screenshot artifacts
@MainActor
enum VisualTestRunner {

    /// Runs all visual tests and generates an HTML report
    static func runAllTests() async throws {
        let artifactsDir = ScreenshotCapture.defaultArtifactsDirectory()
        print("🧪 FGP Manager Visual Tests")
        print("━━━━━━━━━━━━━━━━━━━━━━━━━━━")
        print("📁 Artifacts: \(artifactsDir.path)")
        print("")

        // Clean old artifacts
        try? FileManager.default.removeItem(at: artifactsDir)
        try FileManager.default.createDirectory(at: artifactsDir, withIntermediateDirectories: true)

        var screenshots: [String] = []

        // Test 1: Daemon Card - Running State
        do {
            let daemon = DaemonInfo(name: "browser", status: "running", version: "1.0.0", uptimeSeconds: 3600, isRunning: true, hasManifest: true)
            let appState = AppState()
            appState.daemonStore.daemons = [daemon]

            let view = DaemonCard(daemon: daemon)
                .environmentObject(appState)
                .frame(width: 350, height: 140)
                .background(Color(NSColor.windowBackgroundColor))

            let path = try ScreenshotCapture.capture(view, size: CGSize(width: 350, height: 140), filename: "daemon_card_running")
            screenshots.append("daemon_card_running.png")
            print("✅ daemon_card_running.png")
        }

        // Test 2: Daemon Card - Stopped State
        do {
            let daemon = DaemonInfo(name: "github", status: "stopped", version: nil, uptimeSeconds: nil, isRunning: false, hasManifest: true)
            let appState = AppState()
            appState.daemonStore.daemons = [daemon]

            let view = DaemonCard(daemon: daemon)
                .environmentObject(appState)
                .frame(width: 350, height: 140)
                .background(Color(NSColor.windowBackgroundColor))

            let path = try ScreenshotCapture.capture(view, size: CGSize(width: 350, height: 140), filename: "daemon_card_stopped")
            screenshots.append("daemon_card_stopped.png")
            print("✅ daemon_card_stopped.png")
        }

        // Test 3: Daemon Card - Not Responding State
        do {
            let daemon = DaemonInfo(name: "gmail", status: "not_responding", version: nil, uptimeSeconds: nil, isRunning: false, hasManifest: false)
            let appState = AppState()
            appState.daemonStore.daemons = [daemon]

            let view = DaemonCard(daemon: daemon)
                .environmentObject(appState)
                .frame(width: 350, height: 140)
                .background(Color(NSColor.windowBackgroundColor))

            let path = try ScreenshotCapture.capture(view, size: CGSize(width: 350, height: 140), filename: "daemon_card_not_responding")
            screenshots.append("daemon_card_not_responding.png")
            print("✅ daemon_card_not_responding.png")
        }

        // Test 4: Toggle Comparison - ON vs OFF
        do {
            let runningDaemon = DaemonInfo(name: "running-daemon", status: "running", version: "1.0.0", uptimeSeconds: 100, isRunning: true, hasManifest: true)
            let stoppedDaemon = DaemonInfo(name: "stopped-daemon", status: "stopped", version: nil, uptimeSeconds: nil, isRunning: false, hasManifest: true)
            let appState = AppState()
            appState.daemonStore.daemons = [runningDaemon, stoppedDaemon]

            let view = HStack(spacing: 20) {
                VStack {
                    Text("Toggle ON").font(.caption).foregroundColor(.green)
                    DaemonCard(daemon: runningDaemon)
                        .frame(width: 320, height: 140)
                }
                VStack {
                    Text("Toggle OFF").font(.caption).foregroundColor(.gray)
                    DaemonCard(daemon: stoppedDaemon)
                        .frame(width: 320, height: 140)
                }
            }
            .environmentObject(appState)
            .padding(20)
            .background(Color(NSColor.windowBackgroundColor))

            let path = try ScreenshotCapture.capture(view, size: CGSize(width: 720, height: 200), filename: "toggle_comparison")
            screenshots.append("toggle_comparison.png")
            print("✅ toggle_comparison.png")
        }

        // Test 5: Daemons View with Error Banner
        do {
            let daemons = [
                DaemonInfo(name: "browser", status: "running", version: "1.0.0", uptimeSeconds: 3600, isRunning: true, hasManifest: true),
                DaemonInfo(name: "github", status: "stopped", version: nil, uptimeSeconds: nil, isRunning: false, hasManifest: true),
                DaemonInfo(name: "gmail", status: "not_responding", version: nil, uptimeSeconds: nil, isRunning: false, hasManifest: false),
                DaemonInfo(name: "calendar", status: "stopped", version: nil, uptimeSeconds: nil, isRunning: false, hasManifest: true)
            ]
            let appState = AppState()
            appState.daemonStore.daemons = daemons
            appState.daemonStore.error = "imessage: Service 'imessage' is not installed. Run 'fgp install <path>' first."

            let view = DaemonsView()
                .environmentObject(appState)
                .frame(width: 800, height: 500)

            let path = try ScreenshotCapture.capture(view, size: CGSize(width: 800, height: 500), filename: "daemons_view_with_error")
            screenshots.append("daemons_view_with_error.png")
            print("✅ daemons_view_with_error.png")
        }

        // Test 6: Daemons View - Empty State
        do {
            let appState = AppState()
            appState.daemonStore.daemons = []

            let view = DaemonsView()
                .environmentObject(appState)
                .frame(width: 800, height: 400)

            let path = try ScreenshotCapture.capture(view, size: CGSize(width: 800, height: 400), filename: "daemons_view_empty")
            screenshots.append("daemons_view_empty.png")
            print("✅ daemons_view_empty.png")
        }

        // Test 7: All Daemon States Grid
        do {
            let daemons = [
                DaemonInfo(name: "running", status: "running", version: "1.0.0", uptimeSeconds: 3600, isRunning: true, hasManifest: true),
                DaemonInfo(name: "healthy", status: "healthy", version: "2.0.0", uptimeSeconds: 7200, isRunning: true, hasManifest: true),
                DaemonInfo(name: "stopped", status: "stopped", version: nil, uptimeSeconds: nil, isRunning: false, hasManifest: true),
                DaemonInfo(name: "not-responding", status: "not_responding", version: nil, uptimeSeconds: nil, isRunning: false, hasManifest: false)
            ]
            let appState = AppState()
            appState.daemonStore.daemons = daemons

            let view = LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 16) {
                ForEach(daemons) { daemon in
                    VStack(alignment: .leading) {
                        Text(daemon.status.uppercased())
                            .font(.caption2.bold())
                            .foregroundColor(.secondary)
                        DaemonCard(daemon: daemon)
                            .frame(height: 140)
                    }
                }
            }
            .environmentObject(appState)
            .padding(20)
            .frame(width: 700)
            .background(Color(NSColor.windowBackgroundColor))

            let path = try ScreenshotCapture.capture(view, size: CGSize(width: 700, height: 380), filename: "all_daemon_states")
            screenshots.append("all_daemon_states.png")
            print("✅ all_daemon_states.png")
        }

        print("")
        print("━━━━━━━━━━━━━━━━━━━━━━━━━━━")
        print("📸 Generated \(screenshots.count) screenshots")

        // Generate HTML report
        try generateHTMLReport(screenshots: screenshots, directory: artifactsDir)
        print("📊 Report: \(artifactsDir.appendingPathComponent("report.html").path)")
        print("")
        print("Open report: open \(artifactsDir.appendingPathComponent("report.html").path)")
    }

    private static func generateHTMLReport(screenshots: [String], directory: URL) throws {
        var html = """
        <!DOCTYPE html>
        <html>
        <head>
            <title>FGP Manager Visual Test Report</title>
            <style>
                * { box-sizing: border-box; }
                body {
                    font-family: -apple-system, BlinkMacSystemFont, sans-serif;
                    padding: 40px;
                    background: #1a1a1a;
                    color: #e0e0e0;
                    max-width: 1200px;
                    margin: 0 auto;
                }
                h1 { color: #fff; }
                .subtitle { color: #888; margin-bottom: 30px; }
                .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(380px, 1fr)); gap: 20px; }
                .screenshot {
                    background: #2a2a2a;
                    border-radius: 12px;
                    overflow: hidden;
                }
                .screenshot img {
                    width: 100%;
                    display: block;
                    border-bottom: 1px solid #333;
                }
                .screenshot-name {
                    padding: 15px;
                    font-weight: 500;
                    color: #4a9eff;
                }
            </style>
        </head>
        <body>
            <h1>🦑 FGP Manager Visual Tests</h1>
            <p class="subtitle">Generated: \(Date())</p>
            <div class="grid">
        """

        for filename in screenshots {
            let name = filename.replacingOccurrences(of: ".png", with: "").replacingOccurrences(of: "_", with: " ")
            html += """
                <div class="screenshot">
                    <img src="\(filename)" alt="\(name)">
                    <div class="screenshot-name">\(name)</div>
                </div>
            """
        }

        html += """
            </div>
        </body>
        </html>
        """

        let reportPath = directory.appendingPathComponent("report.html")
        try html.write(to: reportPath, atomically: true, encoding: .utf8)
    }
}
