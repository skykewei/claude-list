use crate::error::{CliError, LocalSourceError};
use crate::model::{
    ConnectionStatus, McpConfig, McpDetail, McpServer, Skill, SkillDetail, SkillStartMatter,
    SourceType,
};
use crate::source::{McpSource, SkillSource};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub struct LocalSource {
    claude_dir: PathBuf,
}

impl LocalSource {
    pub fn new() -> Result<Self, CliError> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| {
                CliError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Home directory not found",
                ))
            })?;
        let claude_dir = PathBuf::from(home).join(".claude");
        Ok(Self { claude_dir })
    }

    pub fn with_path(path: PathBuf) -> Self {
        Self { claude_dir: path }
    }
}

#[derive(Debug, Deserialize)]
struct McpSettings {
    #[serde(rename = "mcpServers", default)]
    mcp_servers: Option<HashMap<String, McpServerConfig>>,
}

#[derive(Debug, Deserialize)]
struct McpServerConfig {
    command: Option<String>,
    args: Option<Vec<String>>,
    env: Option<HashMap<String, String>>,
}

/// Parse YAML frontmatter from markdown content
/// Returns the description field from the frontmatter
fn parse_skill_md(content: &str) -> Option<String> {
    let (start_matter, _) = parse_skill_md_full(content);
    start_matter.description
}

/// Parse YAML frontmatter and body from markdown content
fn parse_skill_md_full(content: &str) -> (SkillStartMatter, String) {
    let mut start_matter = SkillStartMatter::default();

    // Check if content starts with "---"
    if !content.starts_with("---") {
        // No frontmatter, return entire content as body
        return (start_matter, content.to_string());
    }

    // Find the end of frontmatter (second "---")
    let end_marker = match content[3..].find("---") {
        Some(pos) => pos,
        None => {
            // Malformed frontmatter, return entire content
            return (start_matter, content.to_string());
        }
    };

    let frontmatter = &content[3..3 + end_marker];
    let body_start = 3 + end_marker + 3;
    let body = content[body_start..].trim_start().to_string();

    // Parse frontmatter fields
    let mut current_key: Option<&str> = None;
    let mut current_value = String::new();

    for line in frontmatter.lines() {
        let trimmed = line.trim();

        // Check for new key
        if let Some((key, value)) = trimmed.split_once(':') {
            // Save previous key if exists
            if let Some(k) = current_key {
                let cleaned_value = clean_value(&current_value);
                if k == "name" {
                    start_matter.name = Some(cleaned_value);
                } else if k == "description" {
                    start_matter.description = Some(cleaned_value);
                }
            }

            current_key = Some(key.trim());
            current_value = value.trim().to_string();
        } else if current_key.is_some()
            && (trimmed.starts_with('-') || (!trimmed.is_empty() && !trimmed.contains(':')))
        {
            // Multi-line value or list item
            if trimmed.starts_with('-') {
                current_value.push('\n');
            } else {
                current_value.push(' ');
            }
            current_value.push_str(trimmed);
        }
    }

    // Save last key
    if let Some(k) = current_key {
        let cleaned_value = clean_value(&current_value);
        if k == "name" {
            start_matter.name = Some(cleaned_value);
        } else if k == "description" {
            start_matter.description = Some(cleaned_value);
        }
    }

    (start_matter, body)
}

fn clean_value(value: &str) -> String {
    let trimmed = value.trim();
    // Remove quotes if present
    if (trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() > 1)
        || (trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() > 1)
    {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

impl SkillSource for LocalSource {
    fn list_skills(&self) -> Result<Vec<Skill>, CliError> {
        let skills_dir = self.claude_dir.join("skills");

        if !skills_dir.exists() {
            return Ok(Vec::new());
        }

        let mut skills = Vec::new();
        let entries = fs::read_dir(&skills_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();

                // Try to read SKILL.md for description
                let skill_md_path = path.join("SKILL.md");
                let description = if skill_md_path.exists() {
                    fs::read_to_string(&skill_md_path)
                        .ok()
                        .and_then(|c| parse_skill_md(&c))
                } else {
                    None
                };

                skills.push(Skill {
                    name,
                    version: None,
                    source: SourceType::Local,
                    path: Some(path),
                    description,
                });
            }
        }

        skills.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(skills)
    }

    fn get_skill_detail(&self, name: &str) -> Result<SkillDetail, CliError> {
        let skills = self.list_skills()?;
        let name_lower = name.to_lowercase();

        // Find matching skill (case-insensitive partial match)
        let matches: Vec<&Skill> = skills
            .iter()
            .filter(|s| s.name.to_lowercase().contains(&name_lower))
            .collect();

        if matches.is_empty() {
            let all_names: Vec<String> = skills.iter().map(|s| s.name.clone()).collect();
            return Err(CliError::NotFound(name.to_string(), all_names));
        }

        if matches.len() > 1 {
            // If multiple matches, check for exact match first
            if let Some(exact) = matches.iter().find(|s| s.name.to_lowercase() == name_lower) {
                return self.load_skill_detail(exact);
            }
            // Otherwise return ambiguous match error with suggestions
            let suggestions: Vec<String> = matches.iter().map(|s| s.name.clone()).collect();
            return Err(CliError::NotFound(name.to_string(), suggestions));
        }

        self.load_skill_detail(matches[0])
    }
}

impl LocalSource {
    fn load_skill_detail(&self, skill: &Skill) -> Result<SkillDetail, CliError> {
        let skill_md_path = skill
            .path
            .as_ref()
            .ok_or_else(|| LocalSourceError::ConfigNotFound(PathBuf::from("SKILL.md")))?
            .join("SKILL.md");

        let content = fs::read_to_string(&skill_md_path)?;
        let (start_matter, body) = parse_skill_md_full(&content);

        Ok(SkillDetail {
            name: skill.name.clone(),
            start_matter,
            content: body,
            path: skill_md_path,
        })
    }
}

impl McpSource for LocalSource {
    fn list_mcps(&self) -> Result<Vec<McpServer>, CliError> {
        let mut mcps = Vec::new();

        // Try to read from settings.json
        let settings_path = self.claude_dir.join("settings.json");
        if settings_path.exists() {
            let content = fs::read_to_string(&settings_path)?;
            let settings: McpSettings = serde_json::from_str(&content)
                .map_err(|e| LocalSourceError::InvalidConfig(format!("settings.json: {}", e)))?;

            if let Some(servers) = settings.mcp_servers {
                for (name, config) in servers {
                    mcps.push(McpServer {
                        name,
                        status: ConnectionStatus::Unknown,
                        config: Some(McpConfig {
                            command: config.command,
                            args: config.args,
                            env: config.env,
                        }),
                        source: SourceType::Local,
                    });
                }
            }
        }

        // Also try mcp.json
        let mcp_path = self.claude_dir.join("mcp.json");
        if mcp_path.exists() {
            let content = fs::read_to_string(&mcp_path)?;
            let settings: McpSettings = serde_json::from_str(&content)
                .map_err(|e| LocalSourceError::InvalidConfig(format!("mcp.json: {}", e)))?;

            if let Some(servers) = settings.mcp_servers {
                for (name, config) in servers {
                    // Avoid duplicates
                    if !mcps.iter().any(|m| m.name == name) {
                        mcps.push(McpServer {
                            name,
                            status: ConnectionStatus::Unknown,
                            config: Some(McpConfig {
                                command: config.command,
                                args: config.args,
                                env: config.env,
                            }),
                            source: SourceType::Local,
                        });
                    }
                }
            }
        }

        mcps.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(mcps)
    }

    fn get_mcp_detail(&self, name: &str) -> Result<McpDetail, CliError> {
        let mcps = self.list_mcps()?;
        let name_lower = name.to_lowercase();

        // Find matching MCP (case-insensitive partial match)
        let matches: Vec<&McpServer> = mcps
            .iter()
            .filter(|m| m.name.to_lowercase().contains(&name_lower))
            .collect();

        if matches.is_empty() {
            let all_names: Vec<String> = mcps.iter().map(|m| m.name.clone()).collect();
            return Err(CliError::NotFound(name.to_string(), all_names));
        }

        if matches.len() > 1 {
            if let Some(exact) = matches.iter().find(|m| m.name.to_lowercase() == name_lower) {
                return self.load_mcp_detail(exact);
            }
            let suggestions: Vec<String> = matches.iter().map(|m| m.name.clone()).collect();
            return Err(CliError::NotFound(name.to_string(), suggestions));
        }

        self.load_mcp_detail(matches[0])
    }
}

impl LocalSource {
    fn load_mcp_detail(&self, mcp: &McpServer) -> Result<McpDetail, CliError> {
        let config = mcp
            .config
            .clone()
            .ok_or_else(|| CliError::NotFound(mcp.name.clone(), vec![]))?;

        // Determine source path and type
        let settings_path = self.claude_dir.join("settings.json");
        let mcp_path = self.claude_dir.join("mcp.json");

        let (source_path, source_type) = if settings_path.exists() {
            (settings_path.clone(), "settings.json".to_string())
        } else if mcp_path.exists() {
            (mcp_path.clone(), "mcp.json".to_string())
        } else {
            (self.claude_dir.clone(), "unknown".to_string())
        };

        Ok(McpDetail {
            name: mcp.name.clone(),
            config,
            source_path,
            source_type,
        })
    }
}
