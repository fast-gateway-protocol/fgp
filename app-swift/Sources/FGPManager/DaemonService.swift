import Darwin
import Foundation
import os.log

struct DaemonService {
    private let fileManager = FileManager.default
    private let logger = Logger(subsystem: "com.fgp.manager", category: "DaemonService")

    func listDaemons() async throws -> [DaemonInfo] {
        let servicesDir = servicesDirectory()
        guard fileManager.fileExists(atPath: servicesDir.path) else {
            return []
        }

        let entries = try fileManager.contentsOfDirectory(
            at: servicesDir,
            includingPropertiesForKeys: [.isDirectoryKey],
            options: [.skipsHiddenFiles]
        )

        var daemons: [DaemonInfo] = []

        for entry in entries {
            let values = try entry.resourceValues(forKeys: [.isDirectoryKey])
            guard values.isDirectory == true else { continue }

            let name = entry.lastPathComponent
            let manifestPath = entry.appendingPathComponent("manifest.json")
            let hasManifest = fileManager.fileExists(atPath: manifestPath.path)
            let socketPath = serviceSocketPath(name).path

            if fileManager.fileExists(atPath: socketPath) {
                do {
                    let client = FgpClient(socketPath: socketPath)
                    let response = try client.health()
                    if response.ok, let result = response.result {
                        let status = (result["status"] as? String) ?? "running"
                        let version = result["version"] as? String
                        let uptime = numberToUInt64(result["uptime_seconds"])
                        daemons.append(
                            DaemonInfo(
                                name: name,
                                status: status,
                                version: version,
                                uptimeSeconds: uptime,
                                isRunning: true,
                                hasManifest: hasManifest
                            )
                        )
                    } else {
                        daemons.append(
                            DaemonInfo(
                                name: name,
                                status: "not_responding",
                                version: nil,
                                uptimeSeconds: nil,
                                isRunning: false,
                                hasManifest: hasManifest
                            )
                        )
                    }
                } catch {
                    daemons.append(
                        DaemonInfo(
                            name: name,
                            status: "not_responding",
                            version: nil,
                            uptimeSeconds: nil,
                            isRunning: false,
                            hasManifest: hasManifest
                        )
                    )
                }
            } else {
                daemons.append(
                    DaemonInfo(
                        name: name,
                        status: "stopped",
                        version: nil,
                        uptimeSeconds: nil,
                        isRunning: false,
                        hasManifest: hasManifest
                    )
                )
            }
        }

        daemons.sort { $0.name < $1.name }
        return daemons
    }

    func getDaemonHealth(name: String) async throws -> HealthInfo {
        let socketPath = serviceSocketPath(name)
        guard fileManager.fileExists(atPath: socketPath.path) else {
            throw NSError(domain: "FGP", code: 1, userInfo: [
                NSLocalizedDescriptionKey: "Daemon '\(name)' is not running"
            ])
        }

        let client = FgpClient(socketPath: socketPath.path)
        let response = try client.health()
        guard response.ok, let result = response.result else {
            let message = (response.error?["message"] as? String) ?? "Unknown error"
            throw NSError(domain: "FGP", code: 2, userInfo: [
                NSLocalizedDescriptionKey: message
            ])
        }

        var dependencies: [String: DependencyHealth] = [:]
        if let deps = result["dependencies"] as? [String: Any] {
            for (key, value) in deps {
                guard let info = value as? [String: Any] else { continue }
                dependencies[key] = DependencyHealth(
                    ok: (info["ok"] as? Bool) ?? false,
                    latencyMs: numberToDouble(info["latency_ms"]),
                    message: info["message"] as? String
                )
            }
        }

        return HealthInfo(
            status: (result["status"] as? String) ?? "unknown",
            version: result["version"] as? String,
            uptimeSeconds: numberToUInt64(result["uptime_seconds"]),
            dependencies: dependencies
        )
    }

    func startDaemon(name: String) async throws {
        let serviceDir = servicesDirectory().appendingPathComponent(name)
        let manifestPath = serviceDir.appendingPathComponent("manifest.json")

        guard fileManager.fileExists(atPath: manifestPath.path) else {
            throw NSError(domain: "FGP", code: 3, userInfo: [
                NSLocalizedDescriptionKey: "Service '\(name)' is not installed. Run 'fgp install <path>' first."
            ])
        }

        let socketPath = serviceSocketPath(name)
        if fileManager.fileExists(atPath: socketPath.path) {
            if canConnect(socketPath: socketPath.path) {
                return
            } else {
                try? fileManager.removeItem(at: socketPath)
            }
        }

        let manifestData = try Data(contentsOf: manifestPath)
        let manifestObject = try JSONSerialization.jsonObject(with: manifestData, options: [])
        guard let manifest = manifestObject as? [String: Any],
              let daemon = manifest["daemon"] as? [String: Any],
              let entrypoint = daemon["entrypoint"] as? String else {
            throw NSError(domain: "FGP", code: 4, userInfo: [
                NSLocalizedDescriptionKey: "manifest.json missing daemon.entrypoint"
            ])
        }

        let entrypointPath = serviceDir.appendingPathComponent(entrypoint)
        guard fileManager.fileExists(atPath: entrypointPath.path) else {
            throw NSError(domain: "FGP", code: 5, userInfo: [
                NSLocalizedDescriptionKey: "Daemon entrypoint not found: \(entrypointPath.path)"
            ])
        }

        guard fileManager.isExecutableFile(atPath: entrypointPath.path) else {
            throw NSError(domain: "FGP", code: 6, userInfo: [
                NSLocalizedDescriptionKey: "Entrypoint is not executable: \(entrypointPath.path). Run: chmod +x \(entrypointPath.path)"
            ])
        }

        if let permissions = try? fileManager.attributesOfItem(atPath: entrypointPath.path)[.posixPermissions] as? NSNumber {
            if permissions.intValue & 0o002 != 0 {
                logger.warning("Security warning: entrypoint \(entrypointPath.path) is world-writable.")
            }
        }

        let process = Process()
        process.executableURL = entrypointPath
        process.currentDirectoryURL = serviceDir
        process.standardOutput = FileHandle.nullDevice
        process.standardError = FileHandle.nullDevice

        try process.run()

        let deadline = Date().addingTimeInterval(5)
        while Date() < deadline {
            if fileManager.fileExists(atPath: socketPath.path) {
                if canConnect(socketPath: socketPath.path) {
                    return
                }
            }
            try await Task.sleep(nanoseconds: 50_000_000)
        }

        throw NSError(domain: "FGP", code: 7, userInfo: [
            NSLocalizedDescriptionKey: "Service '\(name)' started but socket not ready within 5s"
        ])
    }

    func stopDaemon(name: String) async throws {
        let socketPath = serviceSocketPath(name)
        let pidPath = servicePidPath(name)

        if fileManager.fileExists(atPath: socketPath.path) {
            if let response = try? FgpClient(socketPath: socketPath.path).stop(), response.ok {
                return
            }
        }

        if let pid = readPid(from: pidPath), isProcessRunning(pid: pid) {
            let expected = try readEntrypointName(serviceName: name)
            guard pidMatchesProcess(pid: pid, expectedName: expected) else {
                throw NSError(domain: "FGP", code: 8, userInfo: [
                    NSLocalizedDescriptionKey: "Refusing to stop PID \(pid): process does not match expected entrypoint '\(expected ?? "unknown")'"
                ])
            }

            _ = kill(pid, SIGTERM)
            try await Task.sleep(nanoseconds: 500_000_000)
        }

        try? fileManager.removeItem(at: socketPath)
        try? fileManager.removeItem(at: pidPath)
    }

    func restartDaemon(name: String) async throws {
        try? await stopDaemon(name: name)
        try await startDaemon(name: name)
    }

    func isDaemonRunning(name: String) -> Bool {
        let socketPath = serviceSocketPath(name)
        guard fileManager.fileExists(atPath: socketPath.path) else { return false }
        return canConnect(socketPath: socketPath.path)
    }

    private func servicesDirectory() -> URL {
        let home = fileManager.homeDirectoryForCurrentUser
        return home.appendingPathComponent(".fgp/services", isDirectory: true)
    }

    private func serviceSocketPath(_ name: String) -> URL {
        servicesDirectory().appendingPathComponent("\(name)/daemon.sock")
    }

    private func servicePidPath(_ name: String) -> URL {
        servicesDirectory().appendingPathComponent("\(name)/daemon.pid")
    }

    private func canConnect(socketPath: String) -> Bool {
        let fd = socket(AF_UNIX, SOCK_STREAM, 0)
        if fd == -1 { return false }
        defer { close(fd) }

        var addr = sockaddr_un()
        addr.sun_family = sa_family_t(AF_UNIX)
        let maxLen = Int(MemoryLayout.size(ofValue: addr.sun_path))
        guard socketPath.utf8.count < maxLen else { return false }
        var pathBytes = Array(socketPath.utf8)
        pathBytes.append(0)
        withUnsafeMutablePointer(to: &addr.sun_path) { ptr in
            ptr.withMemoryRebound(to: UInt8.self, capacity: maxLen) { buffer in
                buffer.initialize(from: pathBytes, count: pathBytes.count)
            }
        }
        var addrCopy = addr
        let addrLen = socklen_t(MemoryLayout.size(ofValue: addrCopy))
        let result = withUnsafePointer(to: &addrCopy) { ptr in
            ptr.withMemoryRebound(to: sockaddr.self, capacity: 1) { sockAddr in
                connect(fd, sockAddr, addrLen)
            }
        }
        return result == 0
    }

    private func readPid(from url: URL) -> pid_t? {
        guard let content = try? String(contentsOf: url, encoding: .utf8) else { return nil }
        let trimmed = content.trimmingCharacters(in: .whitespacesAndNewlines)
        guard let pidValue = Int32(trimmed) else { return nil }
        return pid_t(pidValue)
    }

    private func readEntrypointName(serviceName: String) throws -> String? {
        let manifestPath = servicesDirectory()
            .appendingPathComponent(serviceName)
            .appendingPathComponent("manifest.json")
        guard fileManager.fileExists(atPath: manifestPath.path) else { return nil }
        let data = try Data(contentsOf: manifestPath)
        let object = try JSONSerialization.jsonObject(with: data, options: [])
        guard let manifest = object as? [String: Any],
              let daemon = manifest["daemon"] as? [String: Any],
              let entrypoint = daemon["entrypoint"] as? String else {
            return nil
        }
        return URL(fileURLWithPath: entrypoint).lastPathComponent
    }

    private func pidMatchesProcess(pid: pid_t, expectedName: String?) -> Bool {
        guard let expectedName else { return false }
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/ps")
        process.arguments = ["-p", String(pid), "-o", "comm="]

        let pipe = Pipe()
        process.standardOutput = pipe
        try? process.run()
        process.waitUntilExit()

        guard process.terminationStatus == 0 else { return false }
        let data = pipe.fileHandleForReading.readDataToEndOfFile()
        guard let output = String(data: data, encoding: .utf8) else { return false }
        return output.trimmingCharacters(in: .whitespacesAndNewlines).contains(expectedName)
    }

    private func isProcessRunning(pid: pid_t) -> Bool {
        kill(pid, 0) == 0
    }

    private func numberToUInt64(_ value: Any?) -> UInt64? {
        guard let number = value as? NSNumber else { return nil }
        return number.uint64Value
    }

    private func numberToDouble(_ value: Any?) -> Double? {
        guard let number = value as? NSNumber else { return nil }
        return number.doubleValue
    }
}
