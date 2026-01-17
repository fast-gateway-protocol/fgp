-- FGP Skill Registry Schema
-- Migration: 001_initial_schema
-- Created: 01/15/2026
--
-- This schema supports the FGP Skill Platform with quality tiers,
-- cross-agent compatibility, and FGP daemon acceleration.

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================================================
-- ENUMS
-- ============================================================================

-- Quality tier levels (higher = more trusted)
CREATE TYPE quality_tier AS ENUM ('unverified', 'community', 'trusted', 'verified');

-- Skill source types
CREATE TYPE source_type AS ENUM ('github', 'skillsmp', 'direct', 'fgp_native');

-- Supported agent targets
CREATE TYPE agent_type AS ENUM ('claude_code', 'codex', 'gemini', 'cursor', 'other');

-- ============================================================================
-- CORE TABLES
-- ============================================================================

-- Categories for organizing skills
CREATE TABLE categories (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name            TEXT NOT NULL UNIQUE,
    slug            TEXT NOT NULL UNIQUE,
    description     TEXT,
    icon            TEXT,                    -- Emoji or icon class
    skill_count     INTEGER DEFAULT 0,
    created_at      TIMESTAMPTZ DEFAULT NOW(),
    updated_at      TIMESTAMPTZ DEFAULT NOW()
);

-- Trusted organizations (auto-qualify for Tier 1)
CREATE TABLE trusted_orgs (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name            TEXT NOT NULL UNIQUE,    -- GitHub org name (e.g., 'anthropics')
    display_name    TEXT,                    -- Human-readable (e.g., 'Anthropic')
    url             TEXT,
    tier_override   quality_tier DEFAULT 'trusted',
    created_at      TIMESTAMPTZ DEFAULT NOW()
);

-- FGP daemon definitions
CREATE TABLE fgp_daemons (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name            TEXT NOT NULL UNIQUE,    -- e.g., 'gmail', 'browser', 'calendar'
    display_name    TEXT,
    description     TEXT,
    socket_path     TEXT NOT NULL,           -- e.g., '~/.fgp/services/gmail/daemon.sock'
    methods         TEXT[] NOT NULL,         -- Available methods
    version         TEXT,
    avg_speedup     FLOAT,                   -- Measured average speedup factor
    status          TEXT DEFAULT 'active',   -- 'active', 'deprecated', 'beta'
    created_at      TIMESTAMPTZ DEFAULT NOW(),
    updated_at      TIMESTAMPTZ DEFAULT NOW()
);

-- Main skills table
CREATE TABLE skills (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Identity
    name            TEXT NOT NULL,           -- Skill name (e.g., 'create-pr')
    slug            TEXT NOT NULL UNIQUE,    -- URL-safe identifier
    namespace       TEXT,                    -- Optional org namespace (e.g., 'n8n-io')
    full_name       TEXT GENERATED ALWAYS AS (
                        CASE WHEN namespace IS NOT NULL
                        THEN namespace || '/' || name
                        ELSE name END
                    ) STORED,

    -- Metadata
    description     TEXT,
    long_description TEXT,
    author          TEXT,
    author_url      TEXT,
    license         TEXT,
    homepage        TEXT,
    keywords        TEXT[],

    -- Versioning
    version         TEXT DEFAULT '1.0.0',
    version_major   INTEGER GENERATED ALWAYS AS (
                        CAST(SPLIT_PART(version, '.', 1) AS INTEGER)
                    ) STORED,
    version_minor   INTEGER GENERATED ALWAYS AS (
                        CAST(SPLIT_PART(version, '.', 2) AS INTEGER)
                    ) STORED,
    version_patch   INTEGER GENERATED ALWAYS AS (
                        CAST(SPLIT_PART(version, '.', 3) AS INTEGER)
                    ) STORED,

    -- Source tracking
    source          source_type NOT NULL DEFAULT 'github',
    source_url      TEXT,                    -- Full URL to source
    source_repo     TEXT,                    -- 'org/repo' for GitHub
    source_path     TEXT,                    -- Path within repo to SKILL.md
    source_branch   TEXT DEFAULT 'main',
    source_sha      TEXT,                    -- Git SHA for this version

    -- Content
    skill_md        TEXT NOT NULL,           -- Raw SKILL.md content
    skill_md_hash   TEXT NOT NULL,           -- SHA256 of skill_md for integrity
    parsed_frontmatter JSONB,                -- Parsed YAML frontmatter

    -- Quality & Trust
    tier            quality_tier NOT NULL DEFAULT 'unverified',
    tier_reason     TEXT,                    -- Why this tier was assigned
    verified_at     TIMESTAMPTZ,
    verified_by     TEXT,

    -- Agent Compatibility
    agents          agent_type[] DEFAULT ARRAY['claude_code']::agent_type[],
    min_agent_versions JSONB,                -- e.g., {"claude_code": "1.0.0"}
    agent_notes     JSONB,                   -- Agent-specific notes/quirks

    -- FGP Integration
    fgp_daemon_id   UUID REFERENCES fgp_daemons(id),
    fgp_methods     TEXT[],                  -- Which daemon methods this skill uses
    fgp_speedup     FLOAT,                   -- Measured speedup for this skill
    fgp_required    BOOLEAN DEFAULT FALSE,   -- Does skill require FGP to function?

    -- GitHub Metrics (synced from GitHub API)
    github_stars    INTEGER DEFAULT 0,
    github_forks    INTEGER DEFAULT 0,
    github_watchers INTEGER DEFAULT 0,
    github_open_issues INTEGER DEFAULT 0,
    github_last_push TIMESTAMPTZ,
    github_created_at TIMESTAMPTZ,

    -- Registry Metrics
    downloads       INTEGER DEFAULT 0,
    downloads_week  INTEGER DEFAULT 0,
    downloads_month INTEGER DEFAULT 0,
    rating_avg      FLOAT,
    rating_count    INTEGER DEFAULT 0,

    -- Flags
    featured        BOOLEAN DEFAULT FALSE,
    deprecated      BOOLEAN DEFAULT FALSE,
    deprecated_reason TEXT,
    deprecated_replacement TEXT,             -- Slug of replacement skill
    hidden          BOOLEAN DEFAULT FALSE,   -- Hide from search (admin use)

    -- Security
    security_scanned BOOLEAN DEFAULT FALSE,
    security_scan_at TIMESTAMPTZ,
    security_warnings JSONB,                 -- Array of warning objects

    -- Timestamps
    created_at      TIMESTAMPTZ DEFAULT NOW(),
    updated_at      TIMESTAMPTZ DEFAULT NOW(),
    synced_at       TIMESTAMPTZ,             -- Last sync from source

    -- Constraints
    CONSTRAINT valid_version CHECK (version ~ '^\d+\.\d+\.\d+'),
    CONSTRAINT valid_tier_stars CHECK (
        -- Verified: manual review only
        -- Trusted: 100+ stars OR trusted org
        -- Community: 10+ stars
        -- Unverified: < 10 stars
        (tier = 'verified') OR
        (tier = 'trusted' AND (github_stars >= 100 OR namespace IN (SELECT name FROM trusted_orgs))) OR
        (tier = 'community' AND github_stars >= 10) OR
        (tier = 'unverified')
    )
);

-- Skill versions (for rollback and version pinning)
CREATE TABLE skill_versions (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    skill_id        UUID NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    version         TEXT NOT NULL,
    skill_md        TEXT NOT NULL,
    skill_md_hash   TEXT NOT NULL,
    source_sha      TEXT,
    changelog       TEXT,
    created_at      TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(skill_id, version)
);

-- Skill-category mapping (many-to-many)
CREATE TABLE skill_categories (
    skill_id        UUID NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    category_id     UUID NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    is_primary      BOOLEAN DEFAULT FALSE,   -- Primary category for display
    created_at      TIMESTAMPTZ DEFAULT NOW(),

    PRIMARY KEY (skill_id, category_id)
);

-- Skill dependencies (if skills can depend on other skills)
CREATE TABLE skill_dependencies (
    skill_id        UUID NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    depends_on_id   UUID NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    version_constraint TEXT,                 -- SemVer constraint, e.g., ">=1.0.0"
    optional        BOOLEAN DEFAULT FALSE,
    created_at      TIMESTAMPTZ DEFAULT NOW(),

    PRIMARY KEY (skill_id, depends_on_id),
    CHECK (skill_id != depends_on_id)
);

-- ============================================================================
-- ANALYTICS TABLES
-- ============================================================================

-- Installation events (anonymized)
CREATE TABLE installations (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    skill_id        UUID NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    skill_version   TEXT NOT NULL,
    agent           agent_type NOT NULL,
    machine_hash    TEXT,                    -- Anonymized machine identifier
    fgp_version     TEXT,
    os              TEXT,
    installed_at    TIMESTAMPTZ DEFAULT NOW()
);

-- Daily aggregated stats (for efficient queries)
CREATE TABLE skill_stats_daily (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    skill_id        UUID NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    date            DATE NOT NULL,
    downloads       INTEGER DEFAULT 0,
    unique_machines INTEGER DEFAULT 0,

    UNIQUE(skill_id, date)
);

-- ============================================================================
-- SEARCH SUPPORT
-- ============================================================================

-- Full-text search index
ALTER TABLE skills ADD COLUMN search_vector tsvector
    GENERATED ALWAYS AS (
        setweight(to_tsvector('english', coalesce(name, '')), 'A') ||
        setweight(to_tsvector('english', coalesce(namespace, '')), 'A') ||
        setweight(to_tsvector('english', coalesce(description, '')), 'B') ||
        setweight(to_tsvector('english', coalesce(array_to_string(keywords, ' '), '')), 'B') ||
        setweight(to_tsvector('english', coalesce(long_description, '')), 'C')
    ) STORED;

CREATE INDEX idx_skills_search ON skills USING GIN(search_vector);

-- ============================================================================
-- INDEXES
-- ============================================================================

-- Skills
CREATE INDEX idx_skills_tier ON skills(tier);
CREATE INDEX idx_skills_namespace ON skills(namespace);
CREATE INDEX idx_skills_github_stars ON skills(github_stars DESC);
CREATE INDEX idx_skills_downloads ON skills(downloads DESC);
CREATE INDEX idx_skills_updated ON skills(updated_at DESC);
CREATE INDEX idx_skills_source_repo ON skills(source_repo);
CREATE INDEX idx_skills_fgp_daemon ON skills(fgp_daemon_id) WHERE fgp_daemon_id IS NOT NULL;
CREATE INDEX idx_skills_featured ON skills(featured) WHERE featured = TRUE;
CREATE INDEX idx_skills_not_hidden ON skills(id) WHERE hidden = FALSE AND deprecated = FALSE;

-- Skill versions
CREATE INDEX idx_skill_versions_skill ON skill_versions(skill_id);

-- Installations
CREATE INDEX idx_installations_skill ON installations(skill_id);
CREATE INDEX idx_installations_date ON installations(installed_at);

-- Stats
CREATE INDEX idx_skill_stats_date ON skill_stats_daily(date);

-- ============================================================================
-- FUNCTIONS
-- ============================================================================

-- Update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Calculate tier based on criteria
CREATE OR REPLACE FUNCTION calculate_tier(
    p_github_stars INTEGER,
    p_namespace TEXT,
    p_has_marketplace_json BOOLEAN DEFAULT FALSE
) RETURNS quality_tier AS $$
BEGIN
    -- Check if from trusted org
    IF EXISTS (SELECT 1 FROM trusted_orgs WHERE name = p_namespace) THEN
        RETURN 'trusted';
    END IF;

    -- Check star count
    IF p_github_stars >= 100 OR p_has_marketplace_json THEN
        RETURN 'trusted';
    ELSIF p_github_stars >= 10 THEN
        RETURN 'community';
    ELSE
        RETURN 'unverified';
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Increment download counter
CREATE OR REPLACE FUNCTION increment_downloads(p_skill_id UUID)
RETURNS void AS $$
BEGIN
    UPDATE skills
    SET downloads = downloads + 1,
        downloads_week = downloads_week + 1,
        downloads_month = downloads_month + 1
    WHERE id = p_skill_id;
END;
$$ LANGUAGE plpgsql;

-- Update category skill counts
CREATE OR REPLACE FUNCTION update_category_counts()
RETURNS TRIGGER AS $$
BEGIN
    -- Update old category count if removing
    IF TG_OP = 'DELETE' OR TG_OP = 'UPDATE' THEN
        UPDATE categories SET skill_count = (
            SELECT COUNT(*) FROM skill_categories WHERE category_id = OLD.category_id
        ) WHERE id = OLD.category_id;
    END IF;

    -- Update new category count if adding
    IF TG_OP = 'INSERT' OR TG_OP = 'UPDATE' THEN
        UPDATE categories SET skill_count = (
            SELECT COUNT(*) FROM skill_categories WHERE category_id = NEW.category_id
        ) WHERE id = NEW.category_id;
    END IF;

    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- TRIGGERS
-- ============================================================================

-- Auto-update updated_at
CREATE TRIGGER trigger_skills_updated_at
    BEFORE UPDATE ON skills
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trigger_categories_updated_at
    BEFORE UPDATE ON categories
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trigger_fgp_daemons_updated_at
    BEFORE UPDATE ON fgp_daemons
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at();

-- Update category counts on skill_categories changes
CREATE TRIGGER trigger_category_counts
    AFTER INSERT OR UPDATE OR DELETE ON skill_categories
    FOR EACH ROW
    EXECUTE FUNCTION update_category_counts();

-- ============================================================================
-- SEED DATA: Categories (from SkillsMP)
-- ============================================================================

INSERT INTO categories (name, slug, description, icon) VALUES
    ('Tools', 'tools', 'General-purpose tools and utilities', '🔧'),
    ('Development', 'development', 'Software development and coding skills', '💻'),
    ('Data & AI', 'data-ai', 'Data processing, ML, and AI skills', '🤖'),
    ('Business', 'business', 'Business automation and productivity', '💼'),
    ('DevOps', 'devops', 'Infrastructure, CI/CD, and deployment', '🚀'),
    ('Testing & Security', 'testing-security', 'Testing, QA, and security tools', '🔒'),
    ('Documentation', 'documentation', 'Documentation and technical writing', '📚'),
    ('Content & Media', 'content-media', 'Content creation and media processing', '🎨'),
    ('Research', 'research', 'Research and analysis tools', '🔬'),
    ('Databases', 'databases', 'Database management and queries', '🗄️'),
    ('Lifestyle', 'lifestyle', 'Personal productivity and lifestyle', '🌟'),
    ('Blockchain', 'blockchain', 'Blockchain and Web3 development', '⛓️');

-- ============================================================================
-- SEED DATA: Trusted Organizations
-- ============================================================================

INSERT INTO trusted_orgs (name, display_name, url) VALUES
    ('anthropics', 'Anthropic', 'https://anthropic.com'),
    ('openai', 'OpenAI', 'https://openai.com'),
    ('google-gemini', 'Google Gemini', 'https://deepmind.google'),
    ('pytorch', 'PyTorch', 'https://pytorch.org'),
    ('vercel', 'Vercel', 'https://vercel.com'),
    ('electron', 'Electron', 'https://electronjs.org'),
    ('oven-sh', 'Oven (Bun)', 'https://bun.sh'),
    ('n8n-io', 'n8n', 'https://n8n.io'),
    ('microsoft', 'Microsoft', 'https://microsoft.com'),
    ('github', 'GitHub', 'https://github.com'),
    ('cloudflare', 'Cloudflare', 'https://cloudflare.com'),
    ('supabase', 'Supabase', 'https://supabase.com'),
    ('prisma', 'Prisma', 'https://prisma.io'),
    ('facebook', 'Meta', 'https://meta.com'),
    ('meta', 'Meta', 'https://meta.com'),
    ('aws', 'Amazon Web Services', 'https://aws.amazon.com'),
    ('hashicorp', 'HashiCorp', 'https://hashicorp.com'),
    ('grafana', 'Grafana Labs', 'https://grafana.com'),
    ('kubernetes', 'Kubernetes', 'https://kubernetes.io'),
    ('docker', 'Docker', 'https://docker.com');

-- ============================================================================
-- SEED DATA: FGP Daemons
-- ============================================================================

INSERT INTO fgp_daemons (name, display_name, description, socket_path, methods, avg_speedup, status) VALUES
    ('gmail', 'Gmail', 'Gmail email operations', '~/.fgp/services/gmail/daemon.sock',
     ARRAY['gmail.list', 'gmail.read', 'gmail.send', 'gmail.search'], 69.0, 'active'),
    ('browser', 'Browser', 'Browser automation via CDP', '~/.fgp/services/browser/daemon.sock',
     ARRAY['browser.open', 'browser.snapshot', 'browser.click', 'browser.fill', 'browser.screenshot'], 292.0, 'active'),
    ('calendar', 'Calendar', 'Google Calendar operations', '~/.fgp/services/calendar/daemon.sock',
     ARRAY['calendar.list', 'calendar.create', 'calendar.update', 'calendar.delete'], 45.0, 'active'),
    ('github', 'GitHub', 'GitHub API operations', '~/.fgp/services/github/daemon.sock',
     ARRAY['github.issues', 'github.prs', 'github.repos', 'github.search'], 75.0, 'active'),
    ('imessage', 'iMessage', 'macOS iMessage operations', '~/.fgp/services/imessage/daemon.sock',
     ARRAY['imessage.send', 'imessage.read', 'imessage.search', 'imessage.contacts'], 480.0, 'active');

-- ============================================================================
-- VIEWS
-- ============================================================================

-- Skills with all metadata for API responses
CREATE VIEW skills_full AS
SELECT
    s.*,
    c.name as primary_category_name,
    c.slug as primary_category_slug,
    d.name as fgp_daemon_name,
    d.avg_speedup as fgp_daemon_speedup,
    ARRAY_AGG(DISTINCT cat.slug) FILTER (WHERE cat.slug IS NOT NULL) as category_slugs
FROM skills s
LEFT JOIN skill_categories sc ON s.id = sc.skill_id AND sc.is_primary = TRUE
LEFT JOIN categories c ON sc.category_id = c.id
LEFT JOIN fgp_daemons d ON s.fgp_daemon_id = d.id
LEFT JOIN skill_categories sc2 ON s.id = sc2.skill_id
LEFT JOIN categories cat ON sc2.category_id = cat.id
WHERE s.hidden = FALSE
GROUP BY s.id, c.name, c.slug, d.name, d.avg_speedup;

-- Installable skills only (Tier 0-2)
CREATE VIEW skills_installable AS
SELECT * FROM skills_full
WHERE tier IN ('verified', 'trusted', 'community')
  AND deprecated = FALSE;

-- Leaderboard view
CREATE VIEW skills_leaderboard AS
SELECT
    slug,
    full_name,
    description,
    tier,
    github_stars,
    downloads,
    fgp_speedup,
    RANK() OVER (ORDER BY github_stars DESC) as stars_rank,
    RANK() OVER (ORDER BY downloads DESC) as downloads_rank
FROM skills
WHERE hidden = FALSE AND deprecated = FALSE
ORDER BY github_stars DESC;
