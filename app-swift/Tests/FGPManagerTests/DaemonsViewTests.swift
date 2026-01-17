import XCTest
import SwiftUI
import SnapshotTesting
@testable import FGPManager

/// Visual tests for the full DaemonsView
/// Tests error banners, empty states, loading states, and daemon grids
@MainActor
final class DaemonsViewTests: XCTestCase {

    let viewSize = CGSize(width: 800, height: 600)

    // MARK: - Error Banner Tests

    func testDaemonsView_WithErrorBanner() async throws {
        let daemons = [
            MockDaemon.running(name: "browser", version: "1.0.0", uptime: 3600),
            MockDaemon.stopped(name: "github"),
            MockDaemon.notResponding(name: "gmail"),
            MockDaemon.stopped(name: "imessage")
        ]
        let appState = createMockAppState(
            daemons: daemons,
            error: "imessage: Service 'imessage' is not installed. Run 'fgp install <path>' first."
        )

        let view = DaemonsView()
            .environmentObject(appState)
            .frame(width: viewSize.width, height: viewSize.height)

        assertSnapshot(of: view, as: .swiftUIImage(size: viewSize), named: "daemons_view_with_error_banner")

        let artifactPath = try saveArtifact(view, name: "daemons_view_with_error_banner", size: viewSize)
        print("📸 Artifact saved: \(artifactPath.path)")
    }

    func testDaemonsView_NoError() async throws {
        let daemons = [
            MockDaemon.running(name: "browser", version: "1.0.0", uptime: 3600),
            MockDaemon.healthy(name: "gmail", version: "1.0.0", uptime: 7200),
            MockDaemon.stopped(name: "github"),
            MockDaemon.stopped(name: "calendar")
        ]
        let appState = createMockAppState(daemons: daemons, error: nil)

        let view = DaemonsView()
            .environmentObject(appState)
            .frame(width: viewSize.width, height: viewSize.height)

        assertSnapshot(of: view, as: .swiftUIImage(size: viewSize), named: "daemons_view_no_error")

        let artifactPath = try saveArtifact(view, name: "daemons_view_no_error", size: viewSize)
        print("📸 Artifact saved: \(artifactPath.path)")
    }

    // MARK: - Empty State Tests

    func testDaemonsView_EmptyState() async throws {
        let appState = createMockAppState(daemons: [], error: nil)

        let view = DaemonsView()
            .environmentObject(appState)
            .frame(width: viewSize.width, height: viewSize.height)

        assertSnapshot(of: view, as: .swiftUIImage(size: viewSize), named: "daemons_view_empty")

        let artifactPath = try saveArtifact(view, name: "daemons_view_empty", size: viewSize)
        print("📸 Artifact saved: \(artifactPath.path)")
    }

    func testDaemonsView_EmptyWithError() async throws {
        let appState = createMockAppState(
            daemons: [],
            error: "Failed to connect to FGP services directory"
        )

        let view = DaemonsView()
            .environmentObject(appState)
            .frame(width: viewSize.width, height: viewSize.height)

        assertSnapshot(of: view, as: .swiftUIImage(size: viewSize), named: "daemons_view_empty_with_error")

        let artifactPath = try saveArtifact(view, name: "daemons_view_empty_with_error", size: viewSize)
        print("📸 Artifact saved: \(artifactPath.path)")
    }

    // MARK: - Loading State Tests

    func testDaemonsView_Loading() async throws {
        let appState = createMockAppState(daemons: [], error: nil)
        appState.daemonStore.loading = true

        let view = DaemonsView()
            .environmentObject(appState)
            .frame(width: viewSize.width, height: viewSize.height)

        assertSnapshot(of: view, as: .swiftUIImage(size: viewSize), named: "daemons_view_loading")

        let artifactPath = try saveArtifact(view, name: "daemons_view_loading", size: viewSize)
        print("📸 Artifact saved: \(artifactPath.path)")
    }

    // MARK: - Mixed Status Grid

    func testDaemonsView_MixedStatuses() async throws {
        let daemons = [
            MockDaemon.running(name: "browser", version: "1.0.0", uptime: 157),
            MockDaemon.healthy(name: "gmail", version: "1.0.0", uptime: 3978),
            MockDaemon.healthy(name: "screen-time", version: "1.0.0", uptime: 4828),
            MockDaemon.stopped(name: "github"),
            MockDaemon.stopped(name: "calendar"),
            MockDaemon.notResponding(name: "imessage"),
            MockDaemon.notResponding(name: "fly"),
            MockDaemon.stopped(name: "vercel")
        ]
        let appState = createMockAppState(daemons: daemons, error: nil)

        let view = DaemonsView()
            .environmentObject(appState)
            .frame(width: viewSize.width, height: viewSize.height)

        assertSnapshot(of: view, as: .swiftUIImage(size: viewSize), named: "daemons_view_mixed_statuses")

        let artifactPath = try saveArtifact(view, name: "daemons_view_mixed_statuses", size: viewSize)
        print("📸 Artifact saved: \(artifactPath.path)")
    }

    // MARK: - Regression Test: Toggle Clicks Issue

    /// This test documents the toggle issue that was fixed
    /// The toggles should be visible and styled correctly
    func testDaemonsView_ToggleVisibility_RegressionTest() async throws {
        let daemons = [
            MockDaemon.running(name: "running-toggle-on", version: "1.0.0", uptime: 100),
            MockDaemon.stopped(name: "stopped-toggle-off")
        ]
        let appState = createMockAppState(daemons: daemons, error: nil)

        let view = VStack(spacing: 20) {
            Text("Toggle Regression Test")
                .font(.headline)
            Text("Toggles should be visible: green/right for ON, gray/left for OFF")
                .font(.caption)
                .foregroundColor(.secondary)

            DaemonsView()
                .frame(height: 300)
        }
        .environmentObject(appState)
        .frame(width: viewSize.width, height: 450)
        .padding()

        let size = CGSize(width: viewSize.width, height: 450)
        assertSnapshot(of: view, as: .swiftUIImage(size: size), named: "toggle_visibility_regression")

        let artifactPath = try saveArtifact(view, name: "toggle_visibility_regression", size: size)
        print("📸 Artifact saved: \(artifactPath.path)")
    }
}
