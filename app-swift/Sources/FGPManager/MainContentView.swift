import SwiftUI

enum NavigationItem: String, CaseIterable, Identifiable {
    case daemons = "Daemons"
    case marketplace = "Marketplace"
    case settings = "Settings"

    var id: String { rawValue }

    var icon: String {
        switch self {
        case .daemons: return "server.rack"
        case .marketplace: return "bag"
        case .settings: return "gearshape"
        }
    }
}

struct MainContentView: View {
    @EnvironmentObject var appState: AppState
    @EnvironmentObject var appModeManager: AppModeManager
    @State private var selection: NavigationItem = .daemons

    var body: some View {
        NavigationSplitView {
            sidebar
        } detail: {
            detailView
        }
        .frame(minWidth: 900, minHeight: 600)
        .onAppear {
            appState.daemonStore.startMonitoring()
        }
        .onReceive(NotificationCenter.default.publisher(for: .navigateToDaemons)) { _ in
            selection = .daemons
        }
        .onReceive(NotificationCenter.default.publisher(for: .navigateToMarketplace)) { _ in
            selection = .marketplace
        }
        .onReceive(NotificationCenter.default.publisher(for: .navigateToSettings)) { _ in
            selection = .settings
        }
    }

    private var sidebar: some View {
        List(selection: $selection) {
            Section {
                ForEach(NavigationItem.allCases) { item in
                    NavigationLink(value: item) {
                        Label(item.rawValue, systemImage: item.icon)
                    }
                }
            }

            Section("Status") {
                statusView
            }
        }
        .listStyle(.sidebar)
        .navigationSplitViewColumnWidth(min: 200, ideal: 220, max: 280)
    }

    private var statusView: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Circle()
                    .fill(appState.daemonStore.runningCount > 0 ? Color.green : Color.gray)
                    .frame(width: 8, height: 8)
                Text("\(appState.daemonStore.runningCount) running")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            HStack {
                Circle()
                    .fill(Color.gray)
                    .frame(width: 8, height: 8)
                Text("\(appState.daemonStore.stoppedCount) stopped")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
        }
        .padding(.vertical, 4)
    }

    @ViewBuilder
    private var detailView: some View {
        switch selection {
        case .daemons:
            DaemonsView()
                .environmentObject(appState)
        case .marketplace:
            MarketplaceContentView()
                .environmentObject(appState)
        case .settings:
            SettingsContentView()
                .environmentObject(appState)
                .environmentObject(appModeManager)
        }
    }
}

// MARK: - Daemons View (Full Window Version)

struct DaemonsView: View {
    @EnvironmentObject var appState: AppState

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            content
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .onAppear {
            Task { await appState.daemonStore.refresh() }
        }
    }

    private var header: some View {
        HStack {
            VStack(alignment: .leading, spacing: 4) {
                Text("Daemons")
                    .font(.largeTitle.bold())
                Text("Manage your FGP daemons")
                    .foregroundColor(.secondary)
            }
            Spacer()
            Button {
                Task { await appState.daemonStore.refresh() }
            } label: {
                Image(systemName: appState.daemonStore.loading ? "arrow.triangle.2.circlepath" : "arrow.triangle.2.circlepath")
                    .rotationEffect(appState.daemonStore.loading ? .degrees(360) : .degrees(0))
                    .animation(appState.daemonStore.loading ? .linear(duration: 1).repeatForever(autoreverses: false) : .default, value: appState.daemonStore.loading)
            }
            .buttonStyle(.bordered)
            .help("Refresh")
        }
        .padding(24)
    }

    private var content: some View {
        Group {
            // Only show full-page error if we have no daemons to display
            // Toggle errors should show as banners above the daemon list
            if appState.daemonStore.daemons.isEmpty {
                if appState.daemonStore.loading {
                    loadingView
                } else if let error = appState.daemonStore.error {
                    errorView(error)
                } else {
                    emptyView
                }
            } else {
                // Show daemon list with optional error banner
                VStack(spacing: 0) {
                    if let error = appState.daemonStore.error {
                        errorBanner(error)
                    }
                    daemonsList
                }
            }
        }
    }

    private func errorBanner(_ error: String) -> some View {
        HStack(spacing: 12) {
            Image(systemName: "exclamationmark.triangle.fill")
                .foregroundColor(.orange)
            Text(error)
                .font(.callout)
            Spacer()
            Button("Dismiss") {
                appState.daemonStore.error = nil
            }
            .buttonStyle(.borderless)
        }
        .padding(12)
        .background(Color.orange.opacity(0.15))
    }

    private func errorView(_ error: String) -> some View {
        VStack(spacing: 16) {
            Image(systemName: "exclamationmark.triangle.fill")
                .font(.system(size: 48))
                .foregroundColor(.orange)
            Text("Error loading daemons")
                .font(.title2)
            Text(error)
                .font(.body)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
            Button("Try Again") {
                Task { await appState.daemonStore.refresh() }
            }
            .buttonStyle(.borderedProminent)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .padding()
    }

    private var loadingView: some View {
        VStack(spacing: 16) {
            ProgressView()
                .scaleEffect(1.5)
            Text("Loading daemons...")
                .font(.headline)
                .foregroundColor(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private var emptyView: some View {
        VStack(spacing: 16) {
            Image(systemName: "server.rack")
                .font(.system(size: 48))
                .foregroundColor(.secondary)
            Text("No daemons installed")
                .font(.title2)
            Text("Install daemons from the Marketplace to get started")
                .font(.body)
                .foregroundColor(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private var daemonsList: some View {
        ScrollView {
            LazyVGrid(columns: [
                GridItem(.flexible(), spacing: 16),
                GridItem(.flexible(), spacing: 16)
            ], spacing: 16) {
                ForEach(appState.daemonStore.daemons) { daemon in
                    DaemonCard(daemon: daemon)
                        .environmentObject(appState)
                }
            }
            .padding(24)
        }
    }
}

struct DaemonCard: View {
    let daemon: DaemonInfo
    @EnvironmentObject var appState: AppState

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            HStack {
                Circle()
                    .fill(statusColor)
                    .frame(width: 12, height: 12)
                Text(daemon.name)
                    .font(.title3.bold())
                Spacer()
                // Use Button styled as macOS switch (Toggle Binding has SwiftUI interaction issues)
                Button {
                    Task { await appState.daemonStore.toggleDaemon(daemon) }
                } label: {
                    ZStack(alignment: daemon.isRunning ? .trailing : .leading) {
                        Capsule()
                            .fill(daemon.isRunning ? Color.green : Color.gray.opacity(0.3))
                            .frame(width: 51, height: 31)
                        Circle()
                            .fill(Color.white)
                            .frame(width: 27, height: 27)
                            .shadow(color: .black.opacity(0.15), radius: 2, x: 0, y: 1)
                            .padding(2)
                    }
                }
                .buttonStyle(.plain)
                .disabled(appState.daemonStore.loadingDaemon == daemon.name)
                .opacity(appState.daemonStore.loadingDaemon == daemon.name ? 0.5 : 1.0)
            }

            HStack(spacing: 16) {
                if let version = daemon.version {
                    Label("v\(version)", systemImage: "tag")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                if let uptime = daemon.uptimeSeconds {
                    Label(formatUptime(uptime), systemImage: "clock")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }

            HStack {
                Text(daemon.status.capitalized)
                    .font(.caption)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(statusColor.opacity(0.2))
                    .foregroundColor(statusColor)
                    .clipShape(Capsule())
                Spacer()
                if appState.daemonStore.loadingDaemon == daemon.name {
                    ProgressView()
                        .scaleEffect(0.7)
                }
            }
        }
        .padding(16)
        .background(RoundedRectangle(cornerRadius: 12).fill(Color(NSColor.windowBackgroundColor)))
        .overlay(
            RoundedRectangle(cornerRadius: 12)
                .stroke(Color(NSColor.separatorColor), lineWidth: 1)
        )
    }

    private var statusColor: Color {
        switch daemon.status {
        case "running", "healthy":
            return .green
        case "stopped":
            return .gray
        case "degraded", "not_responding":
            return .yellow
        default:
            return .red
        }
    }

    private func formatUptime(_ seconds: UInt64) -> String {
        if seconds < 60 { return "\(seconds)s" }
        if seconds < 3600 { return "\(seconds / 60)m" }
        if seconds < 86400 { return "\(seconds / 3600)h" }
        return "\(seconds / 86400)d"
    }
}

// MARK: - Marketplace Content View (Full Window Version)

struct MarketplaceContentView: View {
    @EnvironmentObject var appState: AppState

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                header
                categoryFilter
                content
            }
            .padding(24)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .onAppear {
            Task { await appState.registryStore.fetchRegistry() }
        }
    }

    private var header: some View {
        HStack {
            VStack(alignment: .leading, spacing: 4) {
                Text("Marketplace")
                    .font(.largeTitle.bold())
                Text("Discover and install FGP daemons")
                    .foregroundColor(.secondary)
            }
            Spacer()
            Button {
                Task { await appState.registryStore.fetchRegistry() }
            } label: {
                Image(systemName: "arrow.triangle.2.circlepath")
            }
            .buttonStyle(.bordered)
        }
    }

    private var categoryFilter: some View {
        Group {
            if let registry = appState.registryStore.registry {
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 8) {
                        categoryButton(title: "All", id: nil)
                        ForEach(registry.categories) { category in
                            categoryButton(title: category.name, id: category.id)
                        }
                    }
                    .padding(.vertical, 4)
                }
            }
        }
    }

    private var content: some View {
        Group {
            if appState.registryStore.loading && appState.registryStore.registry == nil {
                ProgressView()
                    .frame(maxWidth: .infinity, minHeight: 200)
            } else if let error = appState.registryStore.error {
                VStack(alignment: .leading, spacing: 8) {
                    Text(error)
                        .foregroundColor(.red)
                    Button("Try again") {
                        Task { await appState.registryStore.fetchRegistry() }
                    }
                }
                .padding(12)
                .background(RoundedRectangle(cornerRadius: 10).fill(Color.red.opacity(0.1)))
            } else if let registry = appState.registryStore.registry {
                VStack(alignment: .leading, spacing: 20) {
                    if appState.registryStore.selectedCategory == nil && !appState.registryStore.featuredPackages.isEmpty {
                        featuredSection
                    }

                    Text(sectionTitle(for: registry))
                        .font(.title3.bold())
                    LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
                        ForEach(appState.registryStore.filteredPackages) { pkg in
                            packageCard(pkg)
                        }
                    }
                }
            }
        }
    }

    private var featuredSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Featured")
                .font(.title3.bold())
            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
                ForEach(appState.registryStore.featuredPackages) { pkg in
                    featuredCard(pkg)
                }
            }
        }
    }

    private func featuredCard(_ pkg: RegistryPackage) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack(spacing: 10) {
                Image(systemName: iconName(pkg.icon))
                    .font(.title2)
                    .foregroundColor(.accentColor)
                VStack(alignment: .leading, spacing: 2) {
                    HStack {
                        Text(pkg.displayName)
                            .font(.headline)
                        if pkg.official {
                            Text("Official")
                                .font(.caption2)
                                .foregroundColor(.accentColor)
                        }
                    }
                    Text(pkg.description)
                        .font(.caption)
                        .foregroundColor(.secondary)
                        .lineLimit(2)
                }
                Spacer()
            }

            HStack {
                Text("\(pkg.methodsCount) methods")
                    .font(.caption)
                    .foregroundColor(.secondary)
                Spacer()
                if pkg.installed {
                    Text("Installed")
                        .font(.caption2)
                        .foregroundColor(.green)
                        .padding(.horizontal, 6)
                        .padding(.vertical, 2)
                        .background(Capsule().fill(Color.green.opacity(0.2)))
                } else {
                    Button("Install") {
                        Task { await appState.registryStore.installPackage(name: pkg.name) }
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(appState.registryStore.installingPackage != nil)
                }
            }
        }
        .padding(12)
        .background(RoundedRectangle(cornerRadius: 12).fill(Color(NSColor.windowBackgroundColor)))
        .overlay(
            RoundedRectangle(cornerRadius: 12)
                .stroke(Color(NSColor.separatorColor), lineWidth: 1)
        )
    }

    private func packageCard(_ pkg: RegistryPackage) -> some View {
        HStack(spacing: 12) {
            Image(systemName: iconName(pkg.icon))
                .font(.title2)
                .foregroundColor(.secondary)
                .frame(width: 40)
            VStack(alignment: .leading, spacing: 4) {
                HStack(spacing: 8) {
                    Text(pkg.displayName)
                        .font(.headline)
                    if pkg.official {
                        Text("Official")
                            .font(.caption2)
                            .foregroundColor(.accentColor)
                    }
                    if pkg.installed && pkg.updateAvailable {
                        Text("Update")
                            .font(.caption2)
                            .foregroundColor(.orange)
                    }
                }
                Text(pkg.description)
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .lineLimit(2)
                HStack(spacing: 12) {
                    Text("v\(pkg.version)")
                    Text("\(pkg.methodsCount) methods")
                    Text(pkg.author)
                }
                .font(.caption2)
                .foregroundColor(.secondary)
            }
            Spacer()
            packageActions(pkg)
        }
        .padding(16)
        .background(RoundedRectangle(cornerRadius: 12).fill(Color(NSColor.windowBackgroundColor)))
        .overlay(
            RoundedRectangle(cornerRadius: 12)
                .stroke(Color(NSColor.separatorColor), lineWidth: 1)
        )
    }

    private func packageActions(_ pkg: RegistryPackage) -> some View {
        Group {
            if appState.registryStore.installingPackage == pkg.name {
                HStack(spacing: 8) {
                    ProgressView()
                    Text(appState.registryStore.installProgress?.step ?? "Installing...")
                        .font(.caption)
                        .foregroundColor(.secondary)
                        .lineLimit(1)
                }
            } else if pkg.installed {
                HStack(spacing: 8) {
                    Button {
                        if let url = URL(string: pkg.repository) {
                            NSWorkspace.shared.open(url)
                        }
                    } label: {
                        Image(systemName: "link")
                    }
                    .buttonStyle(.borderless)
                    Button("Uninstall") {
                        Task { await appState.registryStore.uninstallPackage(name: pkg.name) }
                    }
                    .buttonStyle(.borderless)
                    .foregroundColor(.red)
                }
            } else {
                Button("Install") {
                    Task { await appState.registryStore.installPackage(name: pkg.name) }
                }
                .buttonStyle(.borderedProminent)
                .disabled(appState.registryStore.installingPackage != nil)
            }
        }
    }

    private func categoryButton(title: String, id: String?) -> some View {
        Button {
            appState.registryStore.selectedCategory = id
        } label: {
            Text(title)
                .font(.caption)
                .padding(.horizontal, 12)
                .padding(.vertical, 6)
                .background(
                    Capsule().fill(appState.registryStore.selectedCategory == id ? Color.accentColor : Color(NSColor.controlBackgroundColor))
                )
                .foregroundColor(appState.registryStore.selectedCategory == id ? .white : .primary)
        }
        .buttonStyle(.plain)
    }

    private func iconName(_ icon: String) -> String {
        switch icon {
        case "mail": return "envelope"
        case "calendar": return "calendar"
        case "code": return "chevron.left.slash.chevron.right"
        case "cloud": return "cloud"
        case "database": return "database"
        case "triangle": return "triangle"
        default: return "globe"
        }
    }

    private func sectionTitle(for registry: Registry) -> String {
        if let selected = appState.registryStore.selectedCategory {
            return registry.categories.first(where: { $0.id == selected })?.name ?? "Packages"
        }
        return "All Packages"
    }
}

// MARK: - Settings Content View (Full Window Version)

struct SettingsContentView: View {
    @EnvironmentObject var appState: AppState
    @EnvironmentObject var appModeManager: AppModeManager

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 24) {
                header
                appearanceSection
                globalHotkeySection
                generalSection
                agentSection
                manualSection
                toolsSection
                performanceSection
            }
            .padding(24)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .onAppear {
            Task {
                await appState.agentsStore.refresh()
                await appState.agentsStore.loadMcpConfig()
                appState.agentsStore.checkAutostart()
            }
        }
    }

    private var header: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Settings")
                .font(.largeTitle.bold())
            Text("Configure FGP integrations")
                .foregroundColor(.secondary)
        }
    }

    private var appearanceSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Appearance")
                .font(.title3.bold())

            VStack(spacing: 0) {
                ForEach(AppMode.allCases, id: \.rawValue) { mode in
                    HStack {
                        VStack(alignment: .leading, spacing: 4) {
                            Text(mode.displayName)
                                .font(.headline)
                            Text(mode.description)
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                        Spacer()
                        if appModeManager.currentMode == mode {
                            Image(systemName: "checkmark.circle.fill")
                                .foregroundColor(.accentColor)
                                .font(.title2)
                        } else {
                            Image(systemName: "circle")
                                .foregroundColor(.secondary)
                                .font(.title2)
                        }
                    }
                    .padding(16)
                    .contentShape(Rectangle())
                    .onTapGesture {
                        withAnimation(.easeInOut(duration: 0.2)) {
                            appModeManager.currentMode = mode
                        }
                    }
                    if mode != AppMode.allCases.last {
                        Divider()
                            .padding(.leading, 16)
                    }
                }
            }
            .background(RoundedRectangle(cornerRadius: 12).fill(Color(NSColor.windowBackgroundColor)))
            .overlay(
                RoundedRectangle(cornerRadius: 12)
                    .stroke(Color(NSColor.separatorColor), lineWidth: 1)
            )

            Text("In Menu Bar Only mode, click the menu bar icon to show a quick access popover. Right-click for the context menu.")
                .font(.caption)
                .foregroundColor(.secondary)
        }
    }

    private var globalHotkeySection: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Global Hotkey")
                .font(.title3.bold())

            VStack(alignment: .leading, spacing: 12) {
                HStack {
                    VStack(alignment: .leading, spacing: 4) {
                        HStack(spacing: 8) {
                            Text("⌃")
                                .font(.system(.title3, design: .rounded).bold())
                                .padding(.horizontal, 8)
                                .padding(.vertical, 4)
                                .background(RoundedRectangle(cornerRadius: 6).fill(Color(NSColor.controlBackgroundColor)))
                            Text("⌥")
                                .font(.system(.title3, design: .rounded).bold())
                                .padding(.horizontal, 8)
                                .padding(.vertical, 4)
                                .background(RoundedRectangle(cornerRadius: 6).fill(Color(NSColor.controlBackgroundColor)))
                            Text("G")
                                .font(.system(.title3, design: .rounded).bold())
                                .padding(.horizontal, 8)
                                .padding(.vertical, 4)
                                .background(RoundedRectangle(cornerRadius: 6).fill(Color(NSColor.controlBackgroundColor)))
                        }
                        Text("Control + Option + G")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                    Spacer()
                    VStack(alignment: .trailing, spacing: 4) {
                        Text("Toggle FGP Manager")
                            .font(.headline)
                        Text("Works from any app")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                }
                .padding(16)

                Divider()

                HStack(spacing: 12) {
                    Image(systemName: "lock.shield")
                        .font(.title2)
                        .foregroundColor(.secondary)
                    VStack(alignment: .leading, spacing: 4) {
                        Text("Accessibility Permission Required")
                            .font(.subheadline.bold())
                        Text("Global hotkeys require accessibility access. Grant permission in System Settings → Privacy & Security → Accessibility.")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                    Spacer()
                    Button("Open Settings") {
                        if let url = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility") {
                            NSWorkspace.shared.open(url)
                        }
                    }
                    .buttonStyle(.bordered)
                }
                .padding(16)
            }
            .background(RoundedRectangle(cornerRadius: 12).fill(Color(NSColor.windowBackgroundColor)))
            .overlay(
                RoundedRectangle(cornerRadius: 12)
                    .stroke(Color(NSColor.separatorColor), lineWidth: 1)
            )
        }
    }

    private var generalSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("General")
                .font(.title3.bold())

            VStack(spacing: 8) {
                HStack {
                    Label {
                        VStack(alignment: .leading, spacing: 4) {
                            Text("Launch at Login")
                                .font(.headline)
                            Text("Start FGP Manager automatically when you log in")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                    } icon: {
                        Image(systemName: "bolt.fill")
                            .foregroundColor(.accentColor)
                    }

                    Spacer()

                    Toggle("", isOn: Binding(
                        get: { appState.agentsStore.autostartEnabled },
                        set: { _ in Task { await appState.agentsStore.toggleAutostart() } }
                    ))
                    .toggleStyle(.switch)
                    .labelsHidden()
                    .disabled(appState.agentsStore.autostartLoading)
                }

                if let error = appState.agentsStore.autostartError {
                    HStack {
                        Image(systemName: "exclamationmark.triangle.fill")
                            .foregroundColor(.orange)
                        Text(error)
                            .font(.caption)
                        Spacer()
                        Button("Retry") {
                            Task { await appState.agentsStore.toggleAutostart() }
                        }
                        .buttonStyle(.borderless)
                        .font(.caption)
                    }
                    .padding(8)
                    .background(RoundedRectangle(cornerRadius: 6).fill(Color.orange.opacity(0.15)))
                }
            }
            .padding(16)
            .background(RoundedRectangle(cornerRadius: 12).fill(Color(NSColor.windowBackgroundColor)))
            .overlay(
                RoundedRectangle(cornerRadius: 12)
                    .stroke(Color(NSColor.separatorColor), lineWidth: 1)
            )
        }
    }

    private var agentSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text("AI Agent Integration")
                        .font(.title3.bold())
                    Text("Connect FGP daemons to your AI coding assistants via MCP")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                Spacer()
                Button {
                    Task { await appState.agentsStore.refresh() }
                } label: {
                    Image(systemName: "arrow.triangle.2.circlepath")
                }
                .buttonStyle(.bordered)
            }

            if let error = appState.agentsStore.error {
                HStack {
                    Image(systemName: "exclamationmark.triangle.fill")
                        .foregroundColor(.red)
                    Text(error)
                        .font(.caption)
                    Spacer()
                    Button("Retry") {
                        Task { await appState.agentsStore.refresh() }
                    }
                    .buttonStyle(.borderless)
                    .font(.caption)
                }
                .padding(8)
                .background(RoundedRectangle(cornerRadius: 8).fill(Color.red.opacity(0.1)))
            }

            if appState.agentsStore.loading && appState.agentsStore.agents.isEmpty {
                ProgressView()
                    .frame(maxWidth: .infinity, minHeight: 80)
            } else {
                VStack(spacing: 8) {
                    ForEach(appState.agentsStore.agents) { agent in
                        agentRow(agent)
                    }
                }
            }
        }
    }

    private func agentRow(_ agent: AgentInfo) -> some View {
        HStack(spacing: 12) {
            Image(systemName: agentIcon(agent.identifier))
                .font(.title2)
                .foregroundColor(agent.installed ? .accentColor : .secondary)
                .frame(width: 40)
            VStack(alignment: .leading, spacing: 2) {
                HStack(spacing: 8) {
                    Text(agent.name)
                        .font(.headline)
                    if !agent.installed {
                        Text("Not Installed")
                            .font(.caption2)
                            .foregroundColor(.secondary)
                            .padding(.horizontal, 6)
                            .padding(.vertical, 2)
                            .background(Capsule().fill(Color.secondary.opacity(0.2)))
                    } else if agent.registered {
                        Text("Connected")
                            .font(.caption2)
                            .foregroundColor(.green)
                            .padding(.horizontal, 6)
                            .padding(.vertical, 2)
                            .background(Capsule().fill(Color.green.opacity(0.2)))
                    }
                }
                if let path = agent.configPath {
                    Text(path)
                        .font(.caption2)
                        .foregroundColor(.secondary)
                        .lineLimit(1)
                }
            }
            Spacer()
            if !agent.installed {
                Text("—")
                    .foregroundColor(.secondary)
            } else if appState.agentsStore.actionLoading == agent.identifier {
                ProgressView()
            } else if agent.registered {
                Button("Disconnect") {
                    Task { await appState.agentsStore.unregister(agentId: agent.identifier) }
                }
                .buttonStyle(.bordered)
                .foregroundColor(.red)
            } else {
                Button("Connect") {
                    Task { await appState.agentsStore.register(agentId: agent.identifier) }
                }
                .buttonStyle(.borderedProminent)
            }
        }
        .padding(16)
        .background(RoundedRectangle(cornerRadius: 12).fill(Color(NSColor.windowBackgroundColor)))
        .overlay(
            RoundedRectangle(cornerRadius: 12)
                .stroke(Color(NSColor.separatorColor), lineWidth: 1)
        )
    }

    private var manualSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Manual Configuration")
                .font(.title3.bold())
            Text("Copy this configuration for other MCP-compatible agents")
                .font(.caption)
                .foregroundColor(.secondary)

            if let error = appState.agentsStore.mcpConfigError {
                HStack {
                    Image(systemName: "exclamationmark.triangle.fill")
                        .foregroundColor(.orange)
                    Text(error)
                        .font(.caption)
                    Spacer()
                    Button("Retry") {
                        Task { await appState.agentsStore.loadMcpConfig() }
                    }
                    .buttonStyle(.borderless)
                    .font(.caption)
                }
                .padding(8)
                .background(RoundedRectangle(cornerRadius: 8).fill(Color.orange.opacity(0.15)))
            } else {
                ZStack(alignment: .topTrailing) {
                    ScrollView {
                        Text(appState.agentsStore.mcpConfig ?? "Loading...")
                            .font(.system(.caption, design: .monospaced))
                            .padding(16)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                    .frame(height: 150)
                    .background(RoundedRectangle(cornerRadius: 12).fill(Color(NSColor.controlBackgroundColor)))
                    .overlay(
                        RoundedRectangle(cornerRadius: 12)
                            .stroke(Color(NSColor.separatorColor), lineWidth: 1)
                    )

                    if appState.agentsStore.mcpConfig != nil {
                        Button {
                            appState.agentsStore.copyConfig()
                        } label: {
                            Image(systemName: appState.agentsStore.copied ? "checkmark" : "doc.on.doc")
                        }
                        .buttonStyle(.bordered)
                        .padding(8)
                    }
                }
            }
        }
    }

    private var toolsSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Available MCP Tools")
                .font(.title3.bold())
            Text("Once connected, these tools become available in your AI assistant")
                .font(.caption)
                .foregroundColor(.secondary)

            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible()), GridItem(.flexible())], spacing: 8) {
                toolCard(name: "fgp_list_daemons", description: "List installed daemons and their status")
                toolCard(name: "fgp_start_daemon", description: "Start a daemon by name")
                toolCard(name: "fgp_browser_*", description: "Browser automation (open, click, fill, screenshot)")
                toolCard(name: "fgp_gmail_*", description: "Email operations (list, read, send, search)")
                toolCard(name: "fgp_github_*", description: "GitHub operations (issues, PRs, repos)")
                toolCard(name: "fgp_calendar_*", description: "Calendar operations (list, create, update)")
            }
        }
    }

    private var performanceSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Performance Advantage")
                .font(.title3.bold())
            Text("FGP daemons eliminate cold-start latency for dramatically faster tool execution")
                .font(.caption)
                .foregroundColor(.secondary)

            Grid(alignment: .leading, horizontalSpacing: 24, verticalSpacing: 12) {
                GridRow {
                    Text("Operation").font(.caption.bold()).foregroundColor(.secondary)
                    Text("MCP Stdio").font(.caption.bold()).foregroundColor(.secondary)
                    Text("FGP Daemon").font(.caption.bold()).foregroundColor(.secondary)
                    Text("Speedup").font(.caption.bold()).foregroundColor(.secondary)
                }
                Divider()
                GridRow {
                    Text("Browser navigate")
                    Text("2,300ms")
                    Text("8ms").foregroundColor(.green)
                    Text("292x").foregroundColor(.green).bold()
                }
                GridRow {
                    Text("Gmail list")
                    Text("2,400ms")
                    Text("35ms").foregroundColor(.green)
                    Text("69x").foregroundColor(.green).bold()
                }
                GridRow {
                    Text("GitHub issues")
                    Text("2,100ms")
                    Text("28ms").foregroundColor(.green)
                    Text("75x").foregroundColor(.green).bold()
                }
            }
            .font(.body)
            .padding(16)
            .background(RoundedRectangle(cornerRadius: 12).fill(Color(NSColor.windowBackgroundColor)))
            .overlay(
                RoundedRectangle(cornerRadius: 12)
                    .stroke(Color(NSColor.separatorColor), lineWidth: 1)
            )
        }
    }

    private func toolCard(name: String, description: String) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(name)
                .font(.system(.caption, design: .monospaced))
                .foregroundColor(.accentColor)
            Text(description)
                .font(.caption2)
                .foregroundColor(.secondary)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(RoundedRectangle(cornerRadius: 8).fill(Color(NSColor.windowBackgroundColor)))
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(Color(NSColor.separatorColor), lineWidth: 1)
        )
    }

    private func agentIcon(_ id: String) -> String {
        switch id {
        case "cursor": return "cursorarrow.rays"
        case "claude-desktop": return "display"
        default: return "sparkles"
        }
    }
}
