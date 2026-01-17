import Foundation

struct AgentService {
    func detectAgents() throws -> [AgentInfo] {
        var agents: [AgentInfo] = []
        let home = FileManager.default.homeDirectoryForCurrentUser

        let claudeInstalled = commandExists("claude")
        let claudeConfig = home.appendingPathComponent(".claude.json")
        let claudeRegistered = isMcpRegistered(in: claudeConfig)

        agents.append(
            AgentInfo(
                name: "Claude Code",
                identifier: "claude-code",
                installed: claudeInstalled,
                configPath: claudeConfig.path,
                registered: claudeRegistered
            )
        )

        let cursorConfig = home.appendingPathComponent(".cursor/mcp.json")
        let cursorInstalled = FileManager.default.fileExists(atPath: cursorConfig.deletingLastPathComponent().path)
        let cursorRegistered = isMcpRegistered(in: cursorConfig)

        agents.append(
            AgentInfo(
                name: "Cursor",
                identifier: "cursor",
                installed: cursorInstalled,
                configPath: cursorConfig.path,
                registered: cursorRegistered
            )
        )

        let claudeDesktopConfig = home.appendingPathComponent("Library/Application Support/Claude/claude_desktop_config.json")
        let claudeDesktopInstalled = FileManager.default.fileExists(atPath: claudeDesktopConfig.deletingLastPathComponent().path)
        let claudeDesktopRegistered = isMcpRegistered(in: claudeDesktopConfig)

        agents.append(
            AgentInfo(
                name: "Claude Desktop",
                identifier: "claude-desktop",
                installed: claudeDesktopInstalled,
                configPath: claudeDesktopConfig.path,
                registered: claudeDesktopRegistered
            )
        )

        return agents
    }

    func registerMcp(agentId: String) throws {
        let mcpServerPath = try ResourcePaths.mcpServerURL().path
        let mcpConfig = mcpServerConfig(path: mcpServerPath)
        let home = FileManager.default.homeDirectoryForCurrentUser

        switch agentId {
        case "claude-code":
            if !runCommand(["claude", "mcp", "add", "fgp", "--", "python3", mcpServerPath]) {
                _ = runCommand(["claude", "mcp", "remove", "fgp"])
                guard runCommand(["claude", "mcp", "add", "fgp", "--", "python3", mcpServerPath]) else {
                    throw NSError(domain: "FGP", code: 10, userInfo: [
                        NSLocalizedDescriptionKey: "Failed to register via claude CLI"
                    ])
                }
            }
        case "cursor":
            let configPath = home.appendingPathComponent(".cursor/mcp.json")
            try updateMcpConfig(at: configPath, with: mcpConfig)
        case "claude-desktop":
            let configPath = home.appendingPathComponent("Library/Application Support/Claude/claude_desktop_config.json")
            try updateMcpConfig(at: configPath, with: mcpConfig)
        default:
            throw NSError(domain: "FGP", code: 11, userInfo: [
                NSLocalizedDescriptionKey: "Unknown agent: \(agentId)"
            ])
        }
    }

    func unregisterMcp(agentId: String) throws {
        let home = FileManager.default.homeDirectoryForCurrentUser

        switch agentId {
        case "claude-code":
            _ = runCommand(["claude", "mcp", "remove", "fgp"])
        case "cursor":
            let configPath = home.appendingPathComponent(".cursor/mcp.json")
            try removeMcpConfig(at: configPath)
        case "claude-desktop":
            let configPath = home.appendingPathComponent("Library/Application Support/Claude/claude_desktop_config.json")
            try removeMcpConfig(at: configPath)
        default:
            throw NSError(domain: "FGP", code: 12, userInfo: [
                NSLocalizedDescriptionKey: "Unknown agent: \(agentId)"
            ])
        }
    }

    func mcpConfigString() throws -> String {
        let mcpServerPath = try ResourcePaths.mcpServerURL().path
        let config: [String: Any] = [
            "mcpServers": [
                "fgp": [
                    "command": "python3",
                    "args": [mcpServerPath]
                ]
            ]
        ]
        let data = try JSONSerialization.data(withJSONObject: config, options: [.prettyPrinted])
        return String(data: data, encoding: .utf8) ?? ""
    }

    private func isMcpRegistered(in url: URL) -> Bool {
        guard let data = try? Data(contentsOf: url),
              let object = try? JSONSerialization.jsonObject(with: data, options: []),
              let json = object as? [String: Any],
              let servers = json["mcpServers"] as? [String: Any] else {
            return false
        }
        return servers["fgp"] != nil
    }

    private func updateMcpConfig(at url: URL, with config: [String: Any]) throws {
        var json: [String: Any] = [:]
        if let data = try? Data(contentsOf: url),
           let object = try? JSONSerialization.jsonObject(with: data, options: []),
           let existing = object as? [String: Any] {
            json = existing
        }

        var servers = (json["mcpServers"] as? [String: Any]) ?? [:]
        servers["fgp"] = config
        json["mcpServers"] = servers

        let data = try JSONSerialization.data(withJSONObject: json, options: [.prettyPrinted])
        let directory = url.deletingLastPathComponent()
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        try data.write(to: url)
    }

    private func removeMcpConfig(at url: URL) throws {
        guard let data = try? Data(contentsOf: url),
              let object = try? JSONSerialization.jsonObject(with: data, options: []),
              var json = object as? [String: Any] else {
            return
        }

        if var servers = json["mcpServers"] as? [String: Any] {
            servers.removeValue(forKey: "fgp")
            json["mcpServers"] = servers
        }

        let output = try JSONSerialization.data(withJSONObject: json, options: [.prettyPrinted])
        try output.write(to: url)
    }

    private func mcpServerConfig(path: String) -> [String: Any] {
        [
            "command": "python3",
            "args": [path]
        ]
    }

    private func commandExists(_ name: String) -> Bool {
        runCommand(["/usr/bin/env", "which", name])
    }

    private func runCommand(_ arguments: [String]) -> Bool {
        guard !arguments.isEmpty else { return false }
        let process = Process()
        if arguments[0].hasPrefix("/") {
            process.executableURL = URL(fileURLWithPath: arguments[0])
            process.arguments = Array(arguments.dropFirst())
        } else {
            process.executableURL = URL(fileURLWithPath: "/usr/bin/env")
            process.arguments = arguments
        }

        do {
            try process.run()
        } catch {
            return false
        }
        process.waitUntilExit()
        return process.terminationStatus == 0
    }
}
