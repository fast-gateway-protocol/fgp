-- FGP Skill Registry Schema
-- Migration: 002_add_new_daemons
-- Created: 01/15/2026
--
-- Add Postgres, Linear, and Notion daemons to the registry.

-- ============================================================================
-- NEW FGP DAEMONS
-- ============================================================================

INSERT INTO fgp_daemons (name, display_name, description, socket_path, methods, avg_speedup, status) VALUES
    ('postgres', 'PostgreSQL', 'Direct PostgreSQL database operations', '~/.fgp/services/postgres/daemon.sock',
     ARRAY['postgres.query', 'postgres.execute', 'postgres.transaction', 'postgres.tables', 'postgres.schema', 'postgres.connections', 'postgres.version'], NULL, 'active'),

    ('linear', 'Linear', 'Linear issue tracking via GraphQL', '~/.fgp/services/linear/daemon.sock',
     ARRAY['linear.me', 'linear.teams', 'linear.issues', 'linear.issue', 'linear.create_issue', 'linear.update_issue', 'linear.comments', 'linear.add_comment', 'linear.projects', 'linear.cycles', 'linear.search', 'linear.states'], NULL, 'active'),

    ('notion', 'Notion', 'Notion pages, databases, and blocks', '~/.fgp/services/notion/daemon.sock',
     ARRAY['notion.me', 'notion.users', 'notion.search', 'notion.page', 'notion.database', 'notion.query_database', 'notion.blocks', 'notion.create_page', 'notion.update_page', 'notion.append_blocks', 'notion.comments', 'notion.add_comment'], NULL, 'active');

-- Add fast-gateway-protocol to trusted orgs
INSERT INTO trusted_orgs (name, display_name, url)
VALUES ('fast-gateway-protocol', 'Fast Gateway Protocol', 'https://github.com/fast-gateway-protocol')
ON CONFLICT (name) DO NOTHING;
