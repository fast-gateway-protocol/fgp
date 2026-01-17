import SwiftUI

struct SettingsView: View {
    @EnvironmentObject var appState: AppState

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 24) {
                header
                generalSection
                agentSection
                manualSection
                toolsSection
                performanceSection
            }
            .padding(24)
        }
        .frame(minWidth: 500, minHeight: 600)
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
            .padding(12)
            .background(RoundedRectangle(cornerRadius: 10).fill(Color(NSColor.windowBackgroundColor)))
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
                .buttonStyle(.borderless)
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
                .foregroundColor(agent.installed ? .accentColor : .secondary)
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
                .buttonStyle(.borderless)
                .foregroundColor(.red)
            } else {
                Button("Connect") {
                    Task { await appState.agentsStore.register(agentId: agent.identifier) }
                }
                .buttonStyle(.borderedProminent)
            }
        }
        .padding(12)
        .background(RoundedRectangle(cornerRadius: 10).fill(Color(NSColor.windowBackgroundColor)))
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
                    ScrollView(.horizontal) {
                        Text(appState.agentsStore.mcpConfig ?? "Loading...")
                            .font(.system(.caption, design: .monospaced))
                            .padding(12)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                    .background(RoundedRectangle(cornerRadius: 10).fill(Color(NSColor.controlBackgroundColor)))

                    if appState.agentsStore.mcpConfig != nil {
                        Button {
                            appState.agentsStore.copyConfig()
                        } label: {
                            Image(systemName: appState.agentsStore.copied ? "checkmark" : "doc.on.doc")
                        }
                        .buttonStyle(.borderless)
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

            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 8) {
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

            Grid(alignment: .leading, horizontalSpacing: 16, verticalSpacing: 8) {
                GridRow {
                    Text("Operation").font(.caption).foregroundColor(.secondary)
                    Text("MCP Stdio").font(.caption).foregroundColor(.secondary)
                    Text("FGP Daemon").font(.caption).foregroundColor(.secondary)
                    Text("Speedup").font(.caption).foregroundColor(.secondary)
                }
                Divider()
                GridRow {
                    Text("Browser navigate")
                    Text("2,300ms")
                    Text("8ms").foregroundColor(.green)
                    Text("292x").foregroundColor(.green)
                }
                GridRow {
                    Text("Gmail list")
                    Text("2,400ms")
                    Text("35ms").foregroundColor(.green)
                    Text("69x").foregroundColor(.green)
                }
                GridRow {
                    Text("GitHub issues")
                    Text("2,100ms")
                    Text("28ms").foregroundColor(.green)
                    Text("75x").foregroundColor(.green)
                }
            }
            .font(.caption)
            .padding(12)
            .background(RoundedRectangle(cornerRadius: 10).fill(Color(NSColor.windowBackgroundColor)))
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
        .padding(10)
        .background(RoundedRectangle(cornerRadius: 8).fill(Color(NSColor.windowBackgroundColor)))
    }

    private func agentIcon(_ id: String) -> String {
        switch id {
        case "cursor":
            return "cursorarrow.rays"
        case "claude-desktop":
            return "display"
        default:
            return "sparkles"
        }
    }
}
