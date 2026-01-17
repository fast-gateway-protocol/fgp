import Foundation

struct DaemonInfo: Identifiable, Hashable {
    var id: String { name }
    let name: String
    let status: String
    let version: String?
    let uptimeSeconds: UInt64?
    let isRunning: Bool
    let hasManifest: Bool
}

struct HealthInfo: Hashable {
    let status: String
    let version: String?
    let uptimeSeconds: UInt64?
    let dependencies: [String: DependencyHealth]
}

struct DependencyHealth: Hashable {
    let ok: Bool
    let latencyMs: Double?
    let message: String?
}

struct Registry: Codable, Hashable {
    let schemaVersion: UInt32
    let updatedAt: String
    var packages: [RegistryPackage]
    let categories: [RegistryCategory]

    enum CodingKeys: String, CodingKey {
        case schemaVersion = "schema_version"
        case updatedAt = "updated_at"
        case packages
        case categories
    }
}

struct RegistryPackage: Codable, Hashable, Identifiable {
    var id: String { name }
    let name: String
    let displayName: String
    let version: String
    let description: String
    let icon: String
    let author: String
    let repository: String
    let platforms: [String: RegistryPlatform]?
    let methodsCount: UInt32
    let featured: Bool
    let official: Bool
    let category: String

    // Runtime-computed fields (not in JSON, populated by RegistryService)
    var installed: Bool = false
    var installedVersion: String?
    var updateAvailable: Bool = false

    // Skill Platform fields (optional, for future skill registry integration)
    var qualityTier: UInt8?           // 0=Verified, 1=Trusted, 2=Community, 3=Unverified
    var fgpDaemon: String?            // Associated daemon name if FGP-accelerated
    var fgpSpeedup: Float?            // e.g., 292.0 for browser
    var agentCompatibility: [String]? // ["claude-code", "codex", "cursor"]
    var skillMdUrl: String?           // URL to raw SKILL.md content
    var downloads: UInt32?            // Download count

    enum CodingKeys: String, CodingKey {
        case name
        case displayName = "display_name"
        case version
        case description
        case icon
        case author
        case repository
        case platforms
        case methodsCount = "methods_count"
        case featured
        case official
        case category
        // Runtime fields excluded from JSON decoding
        case qualityTier = "quality_tier"
        case fgpDaemon = "fgp_daemon"
        case fgpSpeedup = "fgp_speedup"
        case agentCompatibility = "agent_compatibility"
        case skillMdUrl = "skill_md_url"
        case downloads
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        name = try container.decode(String.self, forKey: .name)
        displayName = try container.decode(String.self, forKey: .displayName)
        version = try container.decode(String.self, forKey: .version)
        description = try container.decode(String.self, forKey: .description)
        icon = try container.decode(String.self, forKey: .icon)
        author = try container.decode(String.self, forKey: .author)
        repository = try container.decode(String.self, forKey: .repository)
        platforms = try container.decodeIfPresent([String: RegistryPlatform].self, forKey: .platforms)
        methodsCount = try container.decode(UInt32.self, forKey: .methodsCount)
        featured = try container.decode(Bool.self, forKey: .featured)
        official = try container.decode(Bool.self, forKey: .official)
        category = try container.decode(String.self, forKey: .category)
        // Runtime fields default to false/nil
        installed = false
        installedVersion = nil
        updateAvailable = false
        // Optional skill platform fields
        qualityTier = try container.decodeIfPresent(UInt8.self, forKey: .qualityTier)
        fgpDaemon = try container.decodeIfPresent(String.self, forKey: .fgpDaemon)
        fgpSpeedup = try container.decodeIfPresent(Float.self, forKey: .fgpSpeedup)
        agentCompatibility = try container.decodeIfPresent([String].self, forKey: .agentCompatibility)
        skillMdUrl = try container.decodeIfPresent(String.self, forKey: .skillMdUrl)
        downloads = try container.decodeIfPresent(UInt32.self, forKey: .downloads)
    }
}

// Quality tier definitions for the Skill Platform
enum QualityTier: UInt8, CaseIterable {
    case verified = 0    // Official/manually reviewed
    case trusted = 1     // 100+ stars, from known orgs
    case community = 2   // 10+ stars, active maintenance
    case unverified = 3  // Everything else

    var label: String {
        switch self {
        case .verified: return "Verified"
        case .trusted: return "Trusted"
        case .community: return "Community"
        case .unverified: return "Unverified"
        }
    }

    var icon: String {
        switch self {
        case .verified: return "checkmark.seal.fill"
        case .trusted: return "bolt.fill"
        case .community: return "person.2.fill"
        case .unverified: return "exclamationmark.triangle.fill"
        }
    }
}

struct RegistryPlatform: Codable, Hashable {
    let url: String
    let sha256: String?
}

struct RegistryCategory: Codable, Hashable, Identifiable {
    let id: String
    let name: String
    let icon: String
}

struct InstallProgress: Hashable {
    let package: String
    let step: String
    let progress: UInt32
    let total: UInt32
}

struct AgentInfo: Identifiable, Hashable {
    var id: String { identifier }
    let name: String
    let identifier: String
    let installed: Bool
    let configPath: String?
    let registered: Bool
}
