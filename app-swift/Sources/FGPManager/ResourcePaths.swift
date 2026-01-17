import Foundation

enum ResourcePaths {
    static func baseResourcesURL() throws -> URL {
        #if SWIFT_PACKAGE
        // Bundle.module.resourceURL already points to the Resources folder
        guard let baseURL = Bundle.module.resourceURL else {
            throw CocoaError(.fileNoSuchFile)
        }
        return baseURL
        #else
        guard let baseURL = Bundle.main.resourceURL else {
            throw CocoaError(.fileNoSuchFile)
        }
        return baseURL.appendingPathComponent("Resources", isDirectory: true)
        #endif
    }

    static func registryURL() throws -> URL {
        try baseResourcesURL().appendingPathComponent("registry.json")
    }

    static func mcpServerURL() throws -> URL {
        try baseResourcesURL().appendingPathComponent("mcp/fgp-mcp-server.py")
    }

    static func mcpInstallScriptURL() throws -> URL {
        try baseResourcesURL().appendingPathComponent("mcp/install.sh")
    }

    static func skillMarkdownURL() throws -> URL {
        try baseResourcesURL().appendingPathComponent("skill/skill.md")
    }

    static func trayIconURL() throws -> URL {
        try baseResourcesURL().appendingPathComponent("tray-icon@2x.png")
    }
}
