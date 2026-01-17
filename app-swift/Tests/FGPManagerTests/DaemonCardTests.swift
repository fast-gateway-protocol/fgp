import XCTest
import SwiftUI
import SnapshotTesting
@testable import FGPManager

/// Visual tests for DaemonCard component
/// These tests capture screenshots of the daemon card in various states
/// to verify toggle styling, status indicators, and error handling.
@MainActor
final class DaemonCardTests: XCTestCase {

    // Standard card size for testing
    let cardSize = CGSize(width: 350, height: 140)

    override func setUp() async throws {
        // Record mode: set to true to generate new reference snapshots
        // isRecording = true
    }

    // MARK: - Toggle State Tests

    func testDaemonCard_RunningState() async throws {
        let daemon = MockDaemon.running(name: "browser", version: "1.0.0", uptime: 3600)
        let appState = createMockAppState(daemons: [daemon])

        let view = DaemonCard(daemon: daemon)
            .environmentObject(appState)
            .frame(width: cardSize.width, height: cardSize.height)
            .background(Color(NSColor.windowBackgroundColor))

        // Snapshot test (compares with reference image)
        assertSnapshot(of: view, as: .swiftUIImage(size: cardSize), named: "daemon_card_running")

        // Also save as artifact for manual review
        let artifactPath = try saveArtifact(view, name: "daemon_card_running", size: cardSize)
        print("📸 Artifact saved: \(artifactPath.path)")
    }

    func testDaemonCard_StoppedState() async throws {
        let daemon = MockDaemon.stopped(name: "github")
        let appState = createMockAppState(daemons: [daemon])

        let view = DaemonCard(daemon: daemon)
            .environmentObject(appState)
            .frame(width: cardSize.width, height: cardSize.height)
            .background(Color(NSColor.windowBackgroundColor))

        assertSnapshot(of: view, as: .swiftUIImage(size: cardSize), named: "daemon_card_stopped")

        let artifactPath = try saveArtifact(view, name: "daemon_card_stopped", size: cardSize)
        print("📸 Artifact saved: \(artifactPath.path)")
    }

    func testDaemonCard_NotRespondingState() async throws {
        let daemon = MockDaemon.notResponding(name: "gmail")
        let appState = createMockAppState(daemons: [daemon])

        let view = DaemonCard(daemon: daemon)
            .environmentObject(appState)
            .frame(width: cardSize.width, height: cardSize.height)
            .background(Color(NSColor.windowBackgroundColor))

        assertSnapshot(of: view, as: .swiftUIImage(size: cardSize), named: "daemon_card_not_responding")

        let artifactPath = try saveArtifact(view, name: "daemon_card_not_responding", size: cardSize)
        print("📸 Artifact saved: \(artifactPath.path)")
    }

    func testDaemonCard_HealthyState() async throws {
        let daemon = MockDaemon.healthy(name: "screen-time", version: "1.0.0", uptime: 86400)
        let appState = createMockAppState(daemons: [daemon])

        let view = DaemonCard(daemon: daemon)
            .environmentObject(appState)
            .frame(width: cardSize.width, height: cardSize.height)
            .background(Color(NSColor.windowBackgroundColor))

        assertSnapshot(of: view, as: .swiftUIImage(size: cardSize), named: "daemon_card_healthy")

        let artifactPath = try saveArtifact(view, name: "daemon_card_healthy", size: cardSize)
        print("📸 Artifact saved: \(artifactPath.path)")
    }

    // MARK: - Loading State Tests

    func testDaemonCard_LoadingState() async throws {
        let daemon = MockDaemon.stopped(name: "calendar")
        let appState = createMockAppState(daemons: [daemon])
        appState.daemonStore.loadingDaemon = "calendar"  // Simulates loading

        let view = DaemonCard(daemon: daemon)
            .environmentObject(appState)
            .frame(width: cardSize.width, height: cardSize.height)
            .background(Color(NSColor.windowBackgroundColor))

        assertSnapshot(of: view, as: .swiftUIImage(size: cardSize), named: "daemon_card_loading")

        let artifactPath = try saveArtifact(view, name: "daemon_card_loading", size: cardSize)
        print("📸 Artifact saved: \(artifactPath.path)")
    }

    // MARK: - Toggle Visual Comparison

    func testToggle_OnVsOff_SideBySide() async throws {
        let runningDaemon = MockDaemon.running(name: "browser")
        let stoppedDaemon = MockDaemon.stopped(name: "github")
        let appState = createMockAppState(daemons: [runningDaemon, stoppedDaemon])

        let view = HStack(spacing: 20) {
            VStack {
                Text("Toggle ON (Running)").font(.caption).foregroundColor(.secondary)
                DaemonCard(daemon: runningDaemon)
                    .frame(width: cardSize.width, height: cardSize.height)
            }
            VStack {
                Text("Toggle OFF (Stopped)").font(.caption).foregroundColor(.secondary)
                DaemonCard(daemon: stoppedDaemon)
                    .frame(width: cardSize.width, height: cardSize.height)
            }
        }
        .environmentObject(appState)
        .padding(20)
        .background(Color(NSColor.windowBackgroundColor))

        let comparisonSize = CGSize(width: 760, height: 200)
        assertSnapshot(of: view, as: .swiftUIImage(size: comparisonSize), named: "toggle_comparison_on_vs_off")

        let artifactPath = try saveArtifact(view, name: "toggle_comparison_on_vs_off", size: comparisonSize)
        print("📸 Artifact saved: \(artifactPath.path)")
    }

    // MARK: - All States Grid

    func testDaemonCard_AllStates_Grid() async throws {
        let daemons = [
            MockDaemon.running(name: "running-daemon", version: "1.0.0", uptime: 3600),
            MockDaemon.stopped(name: "stopped-daemon"),
            MockDaemon.notResponding(name: "not-responding-daemon"),
            MockDaemon.healthy(name: "healthy-daemon", version: "2.0.0", uptime: 7200)
        ]
        let appState = createMockAppState(daemons: daemons)
        let height = cardSize.height

        let view = LazyVGrid(columns: [
            GridItem(.flexible(), spacing: 16),
            GridItem(.flexible(), spacing: 16)
        ], spacing: 16) {
            ForEach(daemons) { daemon in
                VStack(alignment: .leading) {
                    Text(daemon.status.uppercased())
                        .font(.caption2.bold())
                        .foregroundColor(.secondary)
                    DaemonCard(daemon: daemon)
                        .frame(height: height)
                }
            }
        }
        .environmentObject(appState)
        .padding(20)
        .frame(width: 740)
        .background(Color(NSColor.windowBackgroundColor))

        let gridSize = CGSize(width: 740, height: 380)
        assertSnapshot(of: view, as: .swiftUIImage(size: gridSize), named: "daemon_card_all_states_grid")

        let artifactPath = try saveArtifact(view, name: "daemon_card_all_states_grid", size: gridSize)
        print("📸 Artifact saved: \(artifactPath.path)")
    }
}
