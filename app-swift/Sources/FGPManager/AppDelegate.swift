import AppKit
import Carbon.HIToolbox
import Combine
import SwiftUI

// MARK: - Global Hotkey Manager

final class GlobalHotkeyManager {
    private var eventTap: CFMachPort?
    private var runLoopSource: CFRunLoopSource?
    private var hotkeyAction: (() -> Void)?

    // Default hotkey: ⌃⌥G (Control+Option+G) for "Global FGP"
    // Key code for 'G' is 5
    private let hotkeyKeyCode: UInt16 = 5  // kVK_ANSI_G

    init(action: @escaping () -> Void) {
        self.hotkeyAction = action
    }

    func start() {
        // Request accessibility permissions if needed
        let options = [kAXTrustedCheckOptionPrompt.takeUnretainedValue(): true] as CFDictionary
        let trusted = AXIsProcessTrustedWithOptions(options)

        if !trusted {
            print("⚠️ Accessibility permissions required for global hotkeys")
            print("   Grant access in System Settings → Privacy & Security → Accessibility")
            return
        }

        setupEventTap()
    }

    func stop() {
        if let eventTap = eventTap {
            CGEvent.tapEnable(tap: eventTap, enable: false)
            if let runLoopSource = runLoopSource {
                CFRunLoopRemoveSource(CFRunLoopGetCurrent(), runLoopSource, .commonModes)
            }
            self.eventTap = nil
            self.runLoopSource = nil
        }
    }

    private func setupEventTap() {
        let eventMask = (1 << CGEventType.keyDown.rawValue)

        // Use a class to hold reference for the C callback
        class HotkeyContext {
            let keyCode: UInt16
            let action: () -> Void

            init(keyCode: UInt16, action: @escaping () -> Void) {
                self.keyCode = keyCode
                self.action = action
            }
        }

        let context = HotkeyContext(keyCode: hotkeyKeyCode, action: hotkeyAction ?? {})
        let contextPtr = Unmanaged.passRetained(context).toOpaque()

        guard let eventTap = CGEvent.tapCreate(
            tap: .cgSessionEventTap,
            place: .headInsertEventTap,
            options: .defaultTap,
            eventsOfInterest: CGEventMask(eventMask),
            callback: { (proxy, type, event, refcon) -> Unmanaged<CGEvent>? in
                guard let refcon = refcon else { return Unmanaged.passRetained(event) }

                // Handle tap disabled event
                if type == .tapDisabledByTimeout || type == .tapDisabledByUserInput {
                    return Unmanaged.passRetained(event)
                }

                let context = Unmanaged<HotkeyContext>.fromOpaque(refcon).takeUnretainedValue()
                let keyCode = UInt16(event.getIntegerValueField(.keyboardEventKeycode))
                let flags = event.flags

                // Check for Control+Option+G
                let controlPressed = flags.contains(.maskControl)
                let optionPressed = flags.contains(.maskAlternate)
                let commandPressed = flags.contains(.maskCommand)
                let shiftPressed = flags.contains(.maskShift)

                // We want Control+Option+G without Command or Shift
                if keyCode == context.keyCode &&
                    controlPressed && optionPressed &&
                    !commandPressed && !shiftPressed {

                    // Trigger the hotkey action on main thread
                    DispatchQueue.main.async {
                        context.action()
                    }

                    // Consume the event (don't pass it to other apps)
                    return nil
                }

                return Unmanaged.passRetained(event)
            },
            userInfo: contextPtr
        ) else {
            print("❌ Failed to create event tap - accessibility permissions may be missing")
            Unmanaged<HotkeyContext>.fromOpaque(contextPtr).release()
            return
        }

        self.eventTap = eventTap

        let runLoopSource = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, eventTap, 0)
        self.runLoopSource = runLoopSource

        CFRunLoopAddSource(CFRunLoopGetCurrent(), runLoopSource, .commonModes)
        CGEvent.tapEnable(tap: eventTap, enable: true)

        print("✅ Global hotkey registered: ⌃⌥G (Control+Option+G)")
    }

    deinit {
        stop()
    }
}

// MARK: - App Mode

enum AppMode: String, CaseIterable {
    case dockAndMenuBar = "dock_and_menu_bar"
    case menuBarOnly = "menu_bar_only"

    var displayName: String {
        switch self {
        case .dockAndMenuBar: return "Dock & Menu Bar"
        case .menuBarOnly: return "Menu Bar Only"
        }
    }

    var description: String {
        switch self {
        case .dockAndMenuBar: return "Show in Dock and menu bar"
        case .menuBarOnly: return "Hide from Dock, show only in menu bar"
        }
    }
}

@MainActor
final class AppModeManager: ObservableObject {
    @Published var currentMode: AppMode {
        didSet {
            UserDefaults.standard.set(currentMode.rawValue, forKey: "appMode")
            applyMode()
        }
    }

    init() {
        let savedMode = UserDefaults.standard.string(forKey: "appMode") ?? AppMode.dockAndMenuBar.rawValue
        self.currentMode = AppMode(rawValue: savedMode) ?? .dockAndMenuBar
    }

    func applyMode() {
        switch currentMode {
        case .dockAndMenuBar:
            NSApp.setActivationPolicy(.regular)
        case .menuBarOnly:
            NSApp.setActivationPolicy(.accessory)
        }
    }
}

// MARK: - App Delegate

@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate {
    private let appState = AppState()
    let appModeManager = AppModeManager()
    private var statusItem: NSStatusItem?
    private var mainWindow: NSWindow?
    private let popover = NSPopover()
    private var cancellables = Set<AnyCancellable>()
    private var globalHotkeyManager: GlobalHotkeyManager?

    func applicationDidFinishLaunching(_ notification: Notification) {
        appModeManager.applyMode()
        setupStatusItem()
        setupPopover()
        setupNotificationObservers()
        setupGlobalHotkey()

        // Only show main window on launch if in dock mode
        if appModeManager.currentMode == .dockAndMenuBar {
            setupMainWindow()
        }

        appState.daemonStore.startMonitoring()

        appState.daemonStore.$daemons
            .receive(on: DispatchQueue.main)
            .sink { [weak self] daemons in
                self?.updateTooltip(daemons)
            }
            .store(in: &cancellables)
    }

    private func setupGlobalHotkey() {
        globalHotkeyManager = GlobalHotkeyManager { [weak self] in
            self?.handleGlobalHotkey()
        }
        globalHotkeyManager?.start()
    }

    private func handleGlobalHotkey() {
        // Global hotkey action: show/hide app based on mode
        if appModeManager.currentMode == .menuBarOnly {
            // In menu bar mode, toggle the popover
            if let button = statusItem?.button {
                if popover.isShown {
                    popover.performClose(nil)
                } else {
                    popover.show(relativeTo: button.bounds, of: button, preferredEdge: .minY)
                    NSApp.activate(ignoringOtherApps: true)
                }
            }
        } else {
            // In dock mode, show/focus the main window
            if let window = mainWindow, window.isVisible, NSApp.isActive {
                // Window is visible and app is active, hide it
                window.orderOut(nil)
            } else {
                showMainWindow()
            }
        }
    }

    private func setupNotificationObservers() {
        // Refresh daemons
        NotificationCenter.default.addObserver(forName: .refreshDaemons, object: nil, queue: .main) { [weak self] _ in
            Task { @MainActor in
                await self?.appState.daemonStore.refresh()
            }
        }

        // Start all daemons
        NotificationCenter.default.addObserver(forName: .startAllDaemons, object: nil, queue: .main) { [weak self] _ in
            Task { @MainActor in
                guard let self else { return }
                for daemon in self.appState.daemonStore.daemons where !daemon.isRunning {
                    await self.appState.daemonStore.toggleDaemon(daemon)
                }
            }
        }

        // Stop all daemons
        NotificationCenter.default.addObserver(forName: .stopAllDaemons, object: nil, queue: .main) { [weak self] _ in
            Task { @MainActor in
                guard let self else { return }
                for daemon in self.appState.daemonStore.daemons where daemon.isRunning {
                    await self.appState.daemonStore.toggleDaemon(daemon)
                }
            }
        }

        // Toggle specific daemon by index
        NotificationCenter.default.addObserver(forName: .toggleDaemon, object: nil, queue: .main) { [weak self] notification in
            guard let index = notification.object as? Int else { return }
            Task { @MainActor in
                guard let self else { return }
                let daemons = self.appState.daemonStore.daemons
                if index < daemons.count {
                    await self.appState.daemonStore.toggleDaemon(daemons[index])
                }
            }
        }

        // Toggle app mode
        NotificationCenter.default.addObserver(forName: .toggleAppMode, object: nil, queue: .main) { [weak self] _ in
            Task { @MainActor in
                guard let self else { return }
                self.appModeManager.currentMode = self.appModeManager.currentMode == .dockAndMenuBar ? .menuBarOnly : .dockAndMenuBar
            }
        }
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        false
    }

    func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows flag: Bool) -> Bool {
        if !flag {
            showMainWindow()
        }
        return true
    }

    private func setupMainWindow() {
        if mainWindow != nil {
            showMainWindow()
            return
        }

        let contentView = MainContentView()
            .environmentObject(appState)
            .environmentObject(appModeManager)

        let hosting = NSHostingController(rootView: contentView)
        let window = NSWindow(contentViewController: hosting)
        window.title = "FGP Manager"
        window.setContentSize(NSSize(width: 1000, height: 700))
        window.minSize = NSSize(width: 800, height: 500)
        window.styleMask = [.titled, .closable, .resizable, .miniaturizable]
        window.isReleasedWhenClosed = false
        window.center()
        window.makeKeyAndOrderFront(nil)
        window.setFrameAutosaveName("FGPManagerMainWindow")

        self.mainWindow = window

        NSApp.activate(ignoringOtherApps: true)
    }

    func showMainWindow() {
        if let window = mainWindow {
            window.makeKeyAndOrderFront(nil)
            NSApp.activate(ignoringOtherApps: true)
        } else {
            setupMainWindow()
        }
    }

    private func setupPopover() {
        popover.behavior = .transient
        popover.animates = true
        popover.contentViewController = NSHostingController(
            rootView: MenuBarPopoverView(openMainWindow: { [weak self] in
                self?.popover.performClose(nil)
                self?.showMainWindow()
            })
            .environmentObject(appState)
            .environmentObject(appModeManager)
        )
    }

    private func setupStatusItem() {
        let statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        if let button = statusItem.button {
            if let iconURL = try? ResourcePaths.trayIconURL(),
               let image = NSImage(contentsOf: iconURL) {
                image.size = NSSize(width: 18, height: 18)
                image.isTemplate = true
                button.image = image
            } else {
                button.image = NSImage(systemSymbolName: "sparkles", accessibilityDescription: "FGP Manager")
            }
            button.action = #selector(statusItemClicked(_:))
            button.target = self
            button.sendAction(on: [.leftMouseUp, .rightMouseUp])
        }
        self.statusItem = statusItem
    }

    @objc private func statusItemClicked(_ sender: Any?) {
        guard let button = statusItem?.button else { return }
        guard let event = NSApp.currentEvent else { return }

        if event.type == .rightMouseUp {
            showContextMenu(relativeTo: button)
        } else {
            // In menu bar only mode, show popover; in dock mode, show main window
            if appModeManager.currentMode == .menuBarOnly {
                togglePopover(relativeTo: button)
            } else {
                showMainWindow()
            }
        }
    }

    private func togglePopover(relativeTo button: NSStatusBarButton) {
        if popover.isShown {
            popover.performClose(nil)
        } else {
            popover.show(relativeTo: button.bounds, of: button, preferredEdge: .minY)
            NSApp.activate(ignoringOtherApps: true)
        }
    }

    private func showContextMenu(relativeTo button: NSStatusBarButton) {
        let menu = NSMenu()

        // Status section
        let runningCount = appState.daemonStore.runningCount
        let totalCount = appState.daemonStore.daemons.count
        let statusMenuItem = NSMenuItem(title: "\(runningCount)/\(totalCount) daemons running", action: nil, keyEquivalent: "")
        statusMenuItem.isEnabled = false
        menu.addItem(statusMenuItem)
        menu.addItem(NSMenuItem.separator())

        // Quick toggle for daemons
        for daemon in appState.daemonStore.daemons.prefix(5) {
            let item = NSMenuItem(
                title: daemon.name,
                action: #selector(toggleDaemonFromMenu(_:)),
                keyEquivalent: ""
            )
            item.target = self
            item.representedObject = daemon.name
            item.state = daemon.isRunning ? .on : .off
            menu.addItem(item)
        }

        if appState.daemonStore.daemons.count > 5 {
            let moreItem = NSMenuItem(title: "...\(appState.daemonStore.daemons.count - 5) more", action: nil, keyEquivalent: "")
            moreItem.isEnabled = false
            menu.addItem(moreItem)
        }

        menu.addItem(NSMenuItem.separator())

        // Mode toggle submenu
        let modeMenu = NSMenu()
        for mode in AppMode.allCases {
            let item = NSMenuItem(title: mode.displayName, action: #selector(setAppMode(_:)), keyEquivalent: "")
            item.target = self
            item.representedObject = mode.rawValue
            item.state = appModeManager.currentMode == mode ? .on : .off
            modeMenu.addItem(item)
        }
        let modeMenuItem = NSMenuItem(title: "App Mode", action: nil, keyEquivalent: "")
        modeMenuItem.submenu = modeMenu
        menu.addItem(modeMenuItem)

        menu.addItem(NSMenuItem.separator())
        menu.addItem(withTitle: "Open FGP Manager", action: #selector(openMainWindow), keyEquivalent: "o").target = self
        menu.addItem(NSMenuItem.separator())
        menu.addItem(withTitle: "Quit", action: #selector(quitApp), keyEquivalent: "q").target = self

        menu.popUp(positioning: nil, at: NSPoint(x: 0, y: button.bounds.height + 4), in: button)
    }

    @objc private func setAppMode(_ sender: NSMenuItem) {
        guard let rawValue = sender.representedObject as? String,
              let mode = AppMode(rawValue: rawValue) else { return }
        appModeManager.currentMode = mode
    }

    @objc private func toggleDaemonFromMenu(_ sender: NSMenuItem) {
        guard let name = sender.representedObject as? String,
              let daemon = appState.daemonStore.daemons.first(where: { $0.name == name }) else { return }
        Task {
            await appState.daemonStore.toggleDaemon(daemon)
        }
    }

    @objc private func openMainWindow() {
        showMainWindow()
    }

    @objc private func quitApp() {
        NSApp.terminate(nil)
    }

    private func updateTooltip(_ daemons: [DaemonInfo]) {
        let running = daemons.filter { $0.isRunning }.count
        let total = daemons.count
        statusItem?.button?.toolTip = "FGP Manager - \(running) of \(total) daemons running"
    }
}

// MARK: - Menu Bar Popover View (for menu bar only mode)

struct MenuBarPopoverView: View {
    @EnvironmentObject var appState: AppState
    @EnvironmentObject var appModeManager: AppModeManager
    var openMainWindow: () -> Void

    var body: some View {
        VStack(spacing: 0) {
            header
            if let error = appState.daemonStore.error {
                errorBanner(error)
            }
            Divider()
            content
            Divider()
            footer
        }
        .frame(width: 320, height: 400)
        .background(VisualEffectView(material: .hudWindow, blendingMode: .behindWindow, state: .active))
        .onAppear {
            Task { await appState.daemonStore.refresh() }
        }
    }

    private var header: some View {
        HStack {
            Text("FGP Manager")
                .font(.headline)
            Spacer()
            Button {
                openMainWindow()
            } label: {
                Image(systemName: "arrow.up.left.and.arrow.down.right")
            }
            .buttonStyle(.borderless)
            .help("Open full window")

            Button {
                Task { await appState.daemonStore.refresh() }
            } label: {
                Image(systemName: appState.daemonStore.loading ? "arrow.triangle.2.circlepath" : "arrow.triangle.2.circlepath")
                    .rotationEffect(appState.daemonStore.loading ? .degrees(360) : .degrees(0))
                    .animation(appState.daemonStore.loading ? .linear(duration: 1).repeatForever(autoreverses: false) : .default, value: appState.daemonStore.loading)
            }
            .buttonStyle(.borderless)
            .help("Refresh")
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 10)
    }

    private var content: some View {
        Group {
            if appState.daemonStore.loading && appState.daemonStore.daemons.isEmpty {
                VStack {
                    ProgressView()
                    Text("Loading...")
                        .font(.footnote)
                        .foregroundColor(.secondary)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if appState.daemonStore.daemons.isEmpty {
                VStack(spacing: 12) {
                    Image(systemName: "server.rack")
                        .font(.system(size: 28))
                        .foregroundColor(.secondary)
                    Text("No daemons installed")
                        .font(.footnote)
                        .foregroundColor(.secondary)
                    Button {
                        openMainWindow()
                    } label: {
                        Label("Open Marketplace", systemImage: "bag")
                    }
                    .buttonStyle(.borderedProminent)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ScrollView {
                    VStack(spacing: 8) {
                        ForEach(appState.daemonStore.daemons) { daemon in
                            daemonRow(daemon)
                        }
                    }
                    .padding(10)
                }
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private func daemonRow(_ daemon: DaemonInfo) -> some View {
        let isSuccess = appState.daemonStore.lastToggleSuccess == daemon.name
        return HStack {
            VStack(alignment: .leading, spacing: 4) {
                HStack(spacing: 6) {
                    Circle()
                        .fill(statusColor(daemon.status))
                        .frame(width: 8, height: 8)
                    Text(daemon.name)
                        .font(.subheadline)
                        .fontWeight(.medium)
                    if isSuccess {
                        Image(systemName: "checkmark.circle.fill")
                            .foregroundColor(.green)
                            .font(.caption)
                            .transition(.scale.combined(with: .opacity))
                    }
                }
                HStack(spacing: 8) {
                    if let version = daemon.version {
                        Text("v\(version)")
                    }
                    if let uptime = daemon.uptimeSeconds {
                        Text(formatUptime(uptime))
                            .font(.system(.caption, design: .monospaced))
                    }
                }
                .font(.caption)
                .foregroundColor(.secondary)
            }
            Spacer()
            // Use Button styled as switch - Toggle has click issues in NSPopover
            Button {
                Task { await appState.daemonStore.toggleDaemon(daemon) }
            } label: {
                ZStack {
                    Capsule()
                        .fill(daemon.isRunning ? Color.green : Color.gray.opacity(0.3))
                        .frame(width: 44, height: 24)
                    Circle()
                        .fill(Color.white)
                        .frame(width: 20, height: 20)
                        .shadow(color: .black.opacity(0.2), radius: 1, x: 0, y: 1)
                        .offset(x: daemon.isRunning ? 10 : -10)
                }
            }
            .buttonStyle(.plain)
            .disabled(appState.daemonStore.loadingDaemon == daemon.name)
            .opacity(appState.daemonStore.loadingDaemon == daemon.name ? 0.5 : 1.0)
        }
        .padding(8)
        .background(RoundedRectangle(cornerRadius: 8).fill(Color(NSColor.windowBackgroundColor).opacity(0.6)))
        .animation(.easeInOut(duration: 0.3), value: isSuccess)
    }

    private var footer: some View {
        HStack {
            if !appState.daemonStore.daemons.isEmpty {
                Text("\(appState.daemonStore.runningCount) running · \(appState.daemonStore.stoppedCount) stopped")
                    .font(.caption)
                    .foregroundColor(.secondary)
            } else {
                Text("No daemons")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            Spacer()
            Button {
                openMainWindow()
            } label: {
                Label("Full window", systemImage: "macwindow")
                    .labelStyle(.titleAndIcon)
            }
            .buttonStyle(.link)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
    }

    private func errorBanner(_ error: String) -> some View {
        HStack(spacing: 8) {
            Image(systemName: "exclamationmark.triangle.fill")
                .foregroundColor(.orange)
            Text(error)
                .font(.caption)
                .lineLimit(2)
            Spacer()
            Button("Retry") {
                Task { await appState.daemonStore.refresh() }
            }
            .buttonStyle(.borderless)
            .font(.caption)
        }
        .padding(8)
        .background(RoundedRectangle(cornerRadius: 6).fill(Color.orange.opacity(0.15)))
        .padding(.horizontal, 12)
        .padding(.top, 4)
    }

    private func statusColor(_ status: String) -> Color {
        switch status {
        case "running", "healthy": return .green
        case "stopped": return .gray
        case "degraded", "not_responding": return .yellow
        default: return .red
        }
    }

    private func formatUptime(_ seconds: UInt64) -> String {
        if seconds < 60 { return "\(seconds)s" }
        if seconds < 3600 { return "\(seconds / 60)m" }
        if seconds < 86400 { return "\(seconds / 3600)h" }
        return "\(seconds / 86400)d"
    }
}
