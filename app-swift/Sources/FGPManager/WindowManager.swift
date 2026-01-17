import AppKit
import SwiftUI

@MainActor
final class WindowManager: ObservableObject {
    weak var popover: NSPopover?

    private var settingsWindow: NSWindow?
    private var marketplaceWindow: NSWindow?

    func closePopover() {
        popover?.performClose(nil)
    }

    func showSettings(appState: AppState) {
        closePopover()
        if let window = settingsWindow {
            window.makeKeyAndOrderFront(nil)
            NSApp.activate(ignoringOtherApps: true)
            return
        }

        let view = SettingsView()
            .environmentObject(appState)
        let hosting = NSHostingController(rootView: view)
        let window = NSWindow(contentViewController: hosting)
        window.title = "FGP Manager Settings"
        window.setContentSize(NSSize(width: 500, height: 600))
        window.minSize = NSSize(width: 480, height: 520)
        window.styleMask = [.titled, .closable, .resizable, .miniaturizable]
        window.isReleasedWhenClosed = false
        window.center()
        window.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
        settingsWindow = window
    }

    func showMarketplace(appState: AppState) {
        closePopover()
        if let window = marketplaceWindow {
            window.makeKeyAndOrderFront(nil)
            NSApp.activate(ignoringOtherApps: true)
            return
        }

        let view = MarketplaceView()
            .environmentObject(appState)
        let hosting = NSHostingController(rootView: view)
        let window = NSWindow(contentViewController: hosting)
        window.title = "FGP Marketplace"
        window.setContentSize(NSSize(width: 800, height: 600))
        window.minSize = NSSize(width: 720, height: 520)
        window.styleMask = [.titled, .closable, .resizable, .miniaturizable]
        window.isReleasedWhenClosed = false
        window.center()
        window.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
        marketplaceWindow = window
    }
}
