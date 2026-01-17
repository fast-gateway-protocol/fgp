import SwiftUI

struct PopoverView: View {
    @EnvironmentObject var appState: AppState
    @EnvironmentObject var windowManager: WindowManager

    var body: some View {
        ZStack {
            VisualEffectView(material: .hudWindow, blendingMode: .behindWindow, state: .active)
                .ignoresSafeArea()
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
        }
        .frame(width: 320, height: 400)
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
                windowManager.showMarketplace(appState: appState)
            } label: {
                Image(systemName: "bolt.fill")
            }
            .buttonStyle(.borderless)
            .help("Marketplace")

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
                    Image(systemName: "tray")
                        .font(.system(size: 28))
                        .foregroundColor(.secondary)
                    Text("No daemons installed")
                        .font(.footnote)
                        .foregroundColor(.secondary)
                    Button {
                        windowManager.showMarketplace(appState: appState)
                    } label: {
                        Label("Browse Marketplace", systemImage: "bolt.fill")
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
            Toggle("", isOn: Binding(
                get: { daemon.isRunning },
                set: { newValue in
                    try? "Toggle SET called for \(daemon.name), newValue: \(newValue)\n".write(toFile: "/tmp/fgp-toggle-set.txt", atomically: false, encoding: .utf8)
                    Task { await appState.daemonStore.toggleDaemon(daemon) }
                }
            ))
            .toggleStyle(.switch)
            .labelsHidden()
            .disabled(appState.daemonStore.loadingDaemon == daemon.name)
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
                windowManager.showMarketplace(appState: appState)
            } label: {
                Label("Get more", systemImage: "plus")
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
