//! SKILL.md parsing for skill import/publish operations.

use anyhow::{Context, Result};
use serde::Deserialize;
use crate::models::AgentType;

/// Parsed skill metadata from SKILL.md frontmatter
#[derive(Debug, Clone)]
pub struct ParsedSkillMd {
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub agents: Option<Vec<AgentType>>,
}

/// YAML frontmatter structure
#[derive(Debug, Deserialize)]
struct Frontmatter {
    name: Option<String>,
    description: Option<String>,
    version: Option<String>,
    author: Option<String>,
    license: Option<String>,
    keywords: Option<Vec<String>>,
    agents: Option<Vec<String>>,
    /// Alternative field names
    #[serde(rename = "supported_agents")]
    supported_agents: Option<Vec<String>>,
}

/// Parse a SKILL.md file and extract metadata
pub fn parse_skill_md(content: &str) -> Result<ParsedSkillMd> {
    // Extract frontmatter
    if !content.starts_with("---") {
        // No frontmatter - try to extract name from first heading
        let name = extract_name_from_content(content)
            .context("SKILL.md must have frontmatter or a heading with the skill name")?;

        return Ok(ParsedSkillMd {
            name,
            description: extract_first_paragraph(content),
            version: None,
            author: None,
            license: None,
            keywords: None,
            agents: None,
        });
    }

    // Find frontmatter end
    let rest = &content[3..];
    let end = rest.find("---")
        .context("Invalid frontmatter: missing closing ---")?;

    let frontmatter_str = &rest[..end].trim();

    // Parse YAML
    let fm: Frontmatter = serde_yaml::from_str(frontmatter_str)
        .context("Failed to parse SKILL.md frontmatter as YAML")?;

    // Get name (required)
    let name = fm.name
        .or_else(|| extract_name_from_content(&rest[end+3..]))
        .context("SKILL.md must have a 'name' field in frontmatter or a heading")?;

    // Parse agents
    let agents = fm.agents.or(fm.supported_agents).map(|agent_strings| {
        agent_strings.iter().filter_map(|s| {
            match s.to_lowercase().as_str() {
                "claude" | "claude-code" | "claude_code" => Some(AgentType::ClaudeCode),
                "codex" | "openai-codex" | "openai_codex" => Some(AgentType::Codex),
                "cursor" => Some(AgentType::Cursor),
                "gemini" | "gemini-cli" | "gemini_cli" => Some(AgentType::Gemini),
                // Map other agents to Other variant
                "windsurf" | "aider" | "cline" | "roo" | "roo-cline" | "amp" | "continue" => Some(AgentType::Other),
                _ => None,
            }
        }).collect()
    });

    Ok(ParsedSkillMd {
        name,
        description: fm.description,
        version: fm.version,
        author: fm.author,
        license: fm.license,
        keywords: fm.keywords,
        agents,
    })
}

/// Extract name from first markdown heading
fn extract_name_from_content(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if let Some(heading) = line.strip_prefix("# ") {
            return Some(heading.trim().to_string());
        }
    }
    None
}

/// Extract first paragraph from markdown content
fn extract_first_paragraph(content: &str) -> Option<String> {
    let mut in_paragraph = false;
    let mut paragraph = String::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip headings and empty lines at the start
        if line.starts_with('#') {
            if in_paragraph {
                break;
            }
            continue;
        }

        if line.is_empty() {
            if in_paragraph {
                break;
            }
            continue;
        }

        // Skip code blocks
        if line.starts_with("```") {
            break;
        }

        in_paragraph = true;
        if !paragraph.is_empty() {
            paragraph.push(' ');
        }
        paragraph.push_str(line);
    }

    if paragraph.is_empty() {
        None
    } else {
        Some(paragraph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
name: Test Skill
description: A test skill for testing
version: 1.0.0
author: Test Author
license: MIT
keywords:
  - test
  - demo
agents:
  - claude-code
  - codex
---

# Test Skill

This is the skill content.
"#;

        let parsed = parse_skill_md(content).unwrap();
        assert_eq!(parsed.name, "Test Skill");
        assert_eq!(parsed.description.unwrap(), "A test skill for testing");
        assert_eq!(parsed.version.unwrap(), "1.0.0");
        assert_eq!(parsed.author.unwrap(), "Test Author");
        assert_eq!(parsed.license.unwrap(), "MIT");
        assert!(parsed.keywords.unwrap().contains(&"test".to_string()));
        assert!(parsed.agents.unwrap().contains(&AgentType::ClaudeCode));
    }

    #[test]
    fn test_parse_no_frontmatter() {
        let content = r#"# My Awesome Skill

This skill does amazing things.

## Usage

Just use it!
"#;

        let parsed = parse_skill_md(content).unwrap();
        assert_eq!(parsed.name, "My Awesome Skill");
        assert_eq!(parsed.description.unwrap(), "This skill does amazing things.");
    }
}
