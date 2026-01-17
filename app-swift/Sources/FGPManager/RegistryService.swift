import CryptoKit
import Foundation

enum RegistrySource {
    case local
    case remote

    static let remoteBaseURL = URL(string: "https://registry.fgp.dev/api/v1")!
}

struct RegistryService {
    private let source: RegistrySource

    init(source: RegistrySource = .local) {
        self.source = source
    }

    func fetchRegistry() throws -> Registry {
        switch source {
        case .local:
            return try fetchLocalRegistry()
        case .remote:
            return try fetchRemoteRegistry()
        }
    }

    private func fetchLocalRegistry() throws -> Registry {
        let registryURL = try ResourcePaths.registryURL()
        let data = try Data(contentsOf: registryURL)
        var registry = try JSONDecoder().decode(Registry.self, from: data)

        let servicesDir = servicesDirectory()
        for index in registry.packages.indices {
            let package = registry.packages[index]
            let manifestPath = servicesDir
                .appendingPathComponent(package.name)
                .appendingPathComponent("manifest.json")

            if FileManager.default.fileExists(atPath: manifestPath.path) {
                registry.packages[index].installed = true

                if let manifestData = try? Data(contentsOf: manifestPath),
                   let object = try? JSONSerialization.jsonObject(with: manifestData, options: []),
                   let manifest = object as? [String: Any],
                   let version = manifest["version"] as? String {
                    registry.packages[index].installedVersion = version
                    registry.packages[index].updateAvailable = version != package.version
                }
            }
        }

        return registry
    }

    private func fetchRemoteRegistry() throws -> Registry {
        // Future: Connect to registry.fgp.dev API
        // For now, fall back to local registry
        // This will be implemented when the registry API is ready
        _ = RegistrySource.remoteBaseURL.appendingPathComponent("packages")

        // Placeholder: fetch from local until API is live
        return try fetchLocalRegistry()
    }

    func installPackage(name: String, progress: @escaping (InstallProgress) -> Void) throws {
        progress(InstallProgress(package: name, step: "Fetching package info...", progress: 0, total: 100))
        let registry = try fetchRegistry()
        guard let package = registry.packages.first(where: { $0.name == name }) else {
            throw NSError(domain: "FGP", code: 20, userInfo: [
                NSLocalizedDescriptionKey: "Package '\(name)' not found"
            ])
        }

        let servicesDir = servicesDirectory()
        try FileManager.default.createDirectory(at: servicesDir, withIntermediateDirectories: true)
        let packageDir = servicesDir.appendingPathComponent(name, isDirectory: true)

        if !FileManager.default.fileExists(atPath: packageDir.path),
           try installFromBinaryIfAvailable(package: package, into: packageDir, progress: progress) {
            progress(InstallProgress(package: name, step: "Installation complete!", progress: 100, total: 100))
            return
        }

        progress(InstallProgress(package: name, step: "Cloning repository...", progress: 20, total: 100))
        if !FileManager.default.fileExists(atPath: packageDir.path) {
            let cloneSuccess = runCommand([
                "git",
                "clone",
                package.repository,
                packageDir.path
            ])
            guard cloneSuccess else {
                throw NSError(domain: "FGP", code: 21, userInfo: [
                    NSLocalizedDescriptionKey: "Git clone failed"
                ])
            }
        }

        progress(InstallProgress(package: name, step: "Building daemon...", progress: 50, total: 100))
        let buildSuccess = runCommand([
            "cargo",
            "build",
            "--release"
        ], workingDirectory: packageDir)
        guard buildSuccess else {
            throw NSError(domain: "FGP", code: 22, userInfo: [
                NSLocalizedDescriptionKey: "Build failed"
            ])
        }

        progress(InstallProgress(package: name, step: "Installation complete!", progress: 100, total: 100))
    }

    func uninstallPackage(name: String) throws {
        let servicesDir = servicesDirectory()
        let packageDir = servicesDir.appendingPathComponent(name, isDirectory: true)
        if FileManager.default.fileExists(atPath: packageDir.path) {
            try FileManager.default.removeItem(at: packageDir)
        }
    }

    private func servicesDirectory() -> URL {
        FileManager.default.homeDirectoryForCurrentUser.appendingPathComponent(".fgp/services", isDirectory: true)
    }

    private func installFromBinaryIfAvailable(
        package: RegistryPackage,
        into packageDir: URL,
        progress: @escaping (InstallProgress) -> Void
    ) throws -> Bool {
        guard let platformKey = currentPlatformKey(),
              let platform = package.platforms?[platformKey],
              let url = URL(string: platform.url) else {
            return false
        }

        progress(InstallProgress(package: package.name, step: "Downloading binary...", progress: 30, total: 100))
        let data = try Data(contentsOf: url)

        if let sha = platform.sha256, !sha.isEmpty, sha != "placeholder" {
            let checksum = SHA256.hash(data: data)
            let hex = checksum.map { String(format: "%02x", $0) }.joined()
            guard hex == sha.lowercased() else {
                throw NSError(domain: "FGP", code: 23, userInfo: [
                    NSLocalizedDescriptionKey: "Binary checksum mismatch"
                ])
            }
        }

        try FileManager.default.createDirectory(at: packageDir, withIntermediateDirectories: true)
        let archiveURL = FileManager.default.temporaryDirectory.appendingPathComponent("\(package.name).tar.gz")
        try data.write(to: archiveURL)

        progress(InstallProgress(package: package.name, step: "Extracting package...", progress: 60, total: 100))
        let extractSuccess = runCommand([
            "tar",
            "-xzf",
            archiveURL.path,
            "-C",
            packageDir.path
        ])
        try? FileManager.default.removeItem(at: archiveURL)

        guard extractSuccess else {
            throw NSError(domain: "FGP", code: 24, userInfo: [
                NSLocalizedDescriptionKey: "Failed to extract binary archive"
            ])
        }
        return true
    }

    private func currentPlatformKey() -> String? {
        var system = utsname()
        uname(&system)
        let machine = withUnsafePointer(to: &system.machine) { ptr -> String in
            let data = Data(bytes: ptr, count: Int(_SYS_NAMELEN))
            return String(decoding: data, as: UTF8.self).trimmingCharacters(in: .controlCharacters)
        }

        if machine.contains("arm64") {
            return "darwin-aarch64"
        }
        if machine.contains("x86_64") {
            return "darwin-x86_64"
        }
        return nil
    }

    private func runCommand(_ arguments: [String], workingDirectory: URL? = nil) -> Bool {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/env")
        process.arguments = arguments
        process.currentDirectoryURL = workingDirectory

        do {
            try process.run()
        } catch {
            return false
        }
        process.waitUntilExit()
        return process.terminationStatus == 0
    }
}
