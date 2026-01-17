import SwiftUI

@main
struct FGPManagerApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate

    init() {
        // Check for visual test mode
        if CommandLine.arguments.contains("--visual-tests") {
            Task { @MainActor in
                do {
                    try await VisualTestRunner.runAllTests()
                    exit(0)
                } catch {
                    print("❌ Visual tests failed: \(error)")
                    exit(1)
                }
            }
            // Keep run loop alive for async task
            RunLoop.main.run(until: Date(timeIntervalSinceNow: 30))
            exit(0)
        }
    }

    var body: some Scene {
        Settings {
            EmptyView()
        }
        .commands {
            CommandGroup(replacing: .newItem) { }

            // View menu - Navigation
            CommandGroup(after: .sidebar) {
                Button("Show Daemons") {
                    NotificationCenter.default.post(name: .navigateToDaemons, object: nil)
                }
                .keyboardShortcut("1", modifiers: .command)

                Button("Show Marketplace") {
                    NotificationCenter.default.post(name: .navigateToMarketplace, object: nil)
                }
                .keyboardShortcut("2", modifiers: .command)

                Button("Show Settings") {
                    NotificationCenter.default.post(name: .navigateToSettings, object: nil)
                }
                .keyboardShortcut(",", modifiers: .command)

                Divider()

                Button("Refresh") {
                    NotificationCenter.default.post(name: .refreshDaemons, object: nil)
                }
                .keyboardShortcut("r", modifiers: .command)
            }

            // Daemon menu
            CommandMenu("Daemons") {
                Button("Start All Daemons") {
                    NotificationCenter.default.post(name: .startAllDaemons, object: nil)
                }
                .keyboardShortcut("s", modifiers: [.command, .shift])

                Button("Stop All Daemons") {
                    NotificationCenter.default.post(name: .stopAllDaemons, object: nil)
                }
                .keyboardShortcut("x", modifiers: [.command, .shift])

                Divider()

                Button("Toggle First Daemon") {
                    NotificationCenter.default.post(name: .toggleDaemon, object: 0)
                }
                .keyboardShortcut("1", modifiers: [.command, .option])

                Button("Toggle Second Daemon") {
                    NotificationCenter.default.post(name: .toggleDaemon, object: 1)
                }
                .keyboardShortcut("2", modifiers: [.command, .option])

                Button("Toggle Third Daemon") {
                    NotificationCenter.default.post(name: .toggleDaemon, object: 2)
                }
                .keyboardShortcut("3", modifiers: [.command, .option])
            }

            // Window menu additions
            CommandGroup(after: .windowSize) {
                Divider()

                Button("Toggle App Mode") {
                    NotificationCenter.default.post(name: .toggleAppMode, object: nil)
                }
                .keyboardShortcut("m", modifiers: [.command, .shift])
            }
        }
    }
}

// MARK: - Notification Names for Keyboard Shortcuts

extension Notification.Name {
    static let navigateToDaemons = Notification.Name("navigateToDaemons")
    static let navigateToMarketplace = Notification.Name("navigateToMarketplace")
    static let navigateToSettings = Notification.Name("navigateToSettings")
    static let refreshDaemons = Notification.Name("refreshDaemons")
    static let startAllDaemons = Notification.Name("startAllDaemons")
    static let stopAllDaemons = Notification.Name("stopAllDaemons")
    static let toggleDaemon = Notification.Name("toggleDaemon")
    static let toggleAppMode = Notification.Name("toggleAppMode")
}
