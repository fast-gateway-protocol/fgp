import AppKit
import Foundation

@MainActor
final class DaemonStore: ObservableObject {
    @Published var daemons: [DaemonInfo] = []
    @Published var loading = false
    @Published var loadingDaemon: String?
    @Published var error: String?
    @Published var lastToggleSuccess: String?

    private let service: DaemonService
    private var monitorTask: Task<Void, Never>?

    init(service: DaemonService) {
        self.service = service
    }

    func startMonitoring() {
        monitorTask?.cancel()
        monitorTask = Task {
            while !Task.isCancelled {
                await refresh()
                try? await Task.sleep(nanoseconds: 5_000_000_000)
            }
        }
    }

    func refresh() async {
        loading = true
        // DON'T clear error at start - preserve toggle errors until successful refresh
        defer { loading = false }
        do {
            let service = service
            daemons = try await Task.detached { try await service.listDaemons() }.value
            // Only clear error if it was a refresh error, not a toggle error
            if error?.contains("Service") != true {
                error = nil
            }
        } catch {
            self.error = error.localizedDescription
            // DON'T clear daemons on refresh error - keep showing the last known state
        }
    }

    func toggleDaemon(_ daemon: DaemonInfo) async {
        loadingDaemon = daemon.name
        error = nil
        defer { loadingDaemon = nil }
        do {
            let service = service
            try await Task.detached {
                if daemon.isRunning {
                    try await service.stopDaemon(name: daemon.name)
                } else {
                    try await service.startDaemon(name: daemon.name)
                }
            }.value
            lastToggleSuccess = daemon.name
            DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) { [weak self] in
                if self?.lastToggleSuccess == daemon.name {
                    self?.lastToggleSuccess = nil
                }
            }
            await refresh()
        } catch {
            self.error = "\(daemon.name): \(error.localizedDescription)"
            // Don't refresh on error - state didn't change and refresh would clear the error
        }
    }

    func stopDaemon(named name: String) async throws {
        let service = service
        try await Task.detached { try await service.stopDaemon(name: name) }.value
    }

    var runningCount: Int {
        daemons.filter { $0.isRunning }.count
    }

    var stoppedCount: Int {
        max(daemons.count - runningCount, 0)
    }
}

@MainActor
final class RegistryStore: ObservableObject {
    @Published var registry: Registry?
    @Published var loading = false
    @Published var error: String?
    @Published var selectedCategory: String?
    @Published var installingPackage: String?
    @Published var installProgress: InstallProgress?

    private let service: RegistryService
    private let daemonStore: DaemonStore

    init(service: RegistryService, daemonStore: DaemonStore) {
        self.service = service
        self.daemonStore = daemonStore
    }

    var filteredPackages: [RegistryPackage] {
        guard let registry else { return [] }
        if let selectedCategory {
            return registry.packages.filter { $0.category == selectedCategory }
        }
        return registry.packages
    }

    var featuredPackages: [RegistryPackage] {
        registry?.packages.filter { $0.featured } ?? []
    }

    func fetchRegistry() async {
        loading = true
        error = nil
        defer { loading = false }
        do {
            let service = service
            registry = try await Task.detached { try service.fetchRegistry() }.value
        } catch {
            self.error = error.localizedDescription
        }
    }

    func installPackage(name: String) async {
        installingPackage = name
        error = nil
        do {
            let service = service
            try await Task.detached { [weak self] in
                try service.installPackage(name: name) { progress in
                    Task { @MainActor in
                        self?.installProgress = progress
                    }
                }
            }.value
            installProgress = nil
            installingPackage = nil
            await fetchRegistry()
            await daemonStore.refresh()
        } catch {
            self.error = error.localizedDescription
            installProgress = nil
            installingPackage = nil
        }
    }

    func uninstallPackage(name: String) async {
        error = nil
        do {
            try await daemonStore.stopDaemon(named: name)
            let service = service
            try await Task.detached { try service.uninstallPackage(name: name) }.value
            await fetchRegistry()
            await daemonStore.refresh()
        } catch {
            self.error = error.localizedDescription
        }
    }
}

@MainActor
final class AgentsStore: ObservableObject {
    @Published var agents: [AgentInfo] = []
    @Published var loading = false
    @Published var error: String?
    @Published var actionLoading: String?
    @Published var mcpConfig: String?
    @Published var mcpConfigError: String?
    @Published var copied = false
    @Published var autostartEnabled = false
    @Published var autostartLoading = false
    @Published var autostartError: String?

    private let agentService: AgentService
    private let autostartService: AutostartService

    init(agentService: AgentService, autostartService: AutostartService) {
        self.agentService = agentService
        self.autostartService = autostartService
    }

    func refresh() async {
        loading = true
        error = nil
        defer { loading = false }
        do {
            let agentService = agentService
            agents = try await Task.detached { try agentService.detectAgents() }.value
        } catch {
            self.error = error.localizedDescription
        }
    }

    func loadMcpConfig() async {
        mcpConfigError = nil
        do {
            let agentService = agentService
            mcpConfig = try await Task.detached { try agentService.mcpConfigString() }.value
        } catch {
            mcpConfigError = error.localizedDescription
            mcpConfig = nil
        }
    }

    func checkAutostart() {
        autostartEnabled = autostartService.isEnabled()
    }

    func toggleAutostart() async {
        autostartLoading = true
        autostartError = nil
        defer { autostartLoading = false }
        do {
            if autostartEnabled {
                try autostartService.disable()
            } else {
                try autostartService.enable()
            }
            autostartEnabled.toggle()
        } catch {
            self.autostartError = error.localizedDescription
        }
    }

    func register(agentId: String) async {
        actionLoading = agentId
        error = nil
        defer { actionLoading = nil }
        do {
            let agentService = agentService
            try await Task.detached { try agentService.registerMcp(agentId: agentId) }.value
            await refresh()
        } catch {
            self.error = error.localizedDescription
        }
    }

    func unregister(agentId: String) async {
        actionLoading = agentId
        error = nil
        defer { actionLoading = nil }
        do {
            let agentService = agentService
            try await Task.detached { try agentService.unregisterMcp(agentId: agentId) }.value
            await refresh()
        } catch {
            self.error = error.localizedDescription
        }
    }

    func copyConfig() {
        guard let mcpConfig else { return }
        let pasteboard = NSPasteboard.general
        pasteboard.clearContents()
        pasteboard.setString(mcpConfig, forType: .string)
        copied = true
        DispatchQueue.main.asyncAfter(deadline: .now() + 2) { [weak self] in
            self?.copied = false
        }
    }
}

@MainActor
final class AppState: ObservableObject {
    let daemonStore: DaemonStore
    let registryStore: RegistryStore
    let agentsStore: AgentsStore

    init() {
        let daemonService = DaemonService()
        let registryService = RegistryService()
        let agentService = AgentService()
        let autostartService = AutostartService()

        self.daemonStore = DaemonStore(service: daemonService)
        self.registryStore = RegistryStore(service: registryService, daemonStore: daemonStore)
        self.agentsStore = AgentsStore(agentService: agentService, autostartService: autostartService)
    }
}
