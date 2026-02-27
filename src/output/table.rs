use crate::error::CliError;
use crate::model::{ClaudeList, DetailItem, McpDetail, SkillDetail};
use crate::output::{DetailFormatter, Formatter};

pub struct TableFormatter {
    verbose: bool,
}

impl TableFormatter {
    pub fn new() -> Self {
        Self { verbose: false }
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    fn format_table(&self, headers: &[&str], rows: &[Vec<String>]) -> String {
        if rows.is_empty() {
            return String::new();
        }

        // Calculate column widths
        let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();

        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }

        // Build table
        let mut output = String::new();

        // Top border
        output.push('+');
        for w in &widths {
            output.push_str(&"-".repeat(w + 2));
            output.push('+');
        }
        output.push('\n');

        // Header row
        output.push('|');
        for (i, header) in headers.iter().enumerate() {
            output.push(' ');
            output.push_str(header);
            output.push_str(&" ".repeat(widths[i].saturating_sub(header.len())));
            output.push_str(" |");
        }
        output.push('\n');

        // Separator
        output.push('+');
        for w in &widths {
            output.push_str(&"-".repeat(w + 2));
            output.push('+');
        }
        output.push('\n');

        // Data rows
        for row in rows {
            output.push('|');
            for (i, cell) in row.iter().enumerate() {
                output.push(' ');
                output.push_str(cell);
                output.push_str(&" ".repeat(widths[i].saturating_sub(cell.len())));
                output.push_str(" |");
            }
            output.push('\n');
        }

        // Bottom border
        output.push('+');
        for w in &widths {
            output.push_str(&"-".repeat(w + 2));
            output.push('+');
        }

        output
    }
}

impl Formatter for TableFormatter {
    fn format(&self, data: &ClaudeList) -> Result<String, CliError> {
        let mut output = String::new();

        // Skills section
        if !data.skills.is_empty() {
            output.push_str("Skills:\n");

            let headers: Vec<&str>;
            let rows: Vec<Vec<String>>;

            // Helper function to truncate unicode strings safely
            fn truncate(s: &str, max_chars: usize) -> String {
                if s.chars().count() <= max_chars {
                    s.to_string()
                } else {
                    let truncated: String = s.chars().take(max_chars.saturating_sub(3)).collect();
                    format!("{}...", truncated)
                }
            }

            if self.verbose {
                headers = vec!["Name", "Version", "Source", "Description"];
                rows = data
                    .skills
                    .iter()
                    .map(|s| {
                        let description = s.description.as_deref().unwrap_or("-");
                        let desc = truncate(description, 50);
                        vec![
                            s.name.clone(),
                            s.version.clone().unwrap_or_else(|| "-".to_string()),
                            format!("{:?}", s.source).to_lowercase(),
                            desc,
                        ]
                    })
                    .collect();
            } else {
                // Check if any skill has a description
                let has_descriptions = data.skills.iter().any(|s| s.description.is_some());

                if has_descriptions {
                    headers = vec!["Name", "Description"];
                    rows = data
                        .skills
                        .iter()
                        .map(|s| {
                            let description = s.description.as_deref().unwrap_or("-");
                            let desc = truncate(description, 60);
                            vec![s.name.clone(), desc]
                        })
                        .collect();
                } else {
                    headers = vec!["Name"];
                    rows = data.skills.iter().map(|s| vec![s.name.clone()]).collect();
                }
            }

            output.push_str(&self.format_table(&headers, &rows));
            output.push_str("\n\n");
        }

        // MCP section
        if !data.mcps.is_empty() {
            output.push_str("MCP Servers:\n");

            let headers: Vec<&str>;
            let rows: Vec<Vec<String>>;

            if self.verbose {
                headers = vec!["Name", "Status", "Command"];
                rows = data
                    .mcps
                    .iter()
                    .map(|m| {
                        let status = format!("{:?}", m.status).to_lowercase();
                        let cmd = m
                            .config
                            .as_ref()
                            .and_then(|c| c.command.as_deref())
                            .unwrap_or("-")
                            .to_string();
                        vec![m.name.clone(), status, cmd]
                    })
                    .collect();
            } else {
                headers = vec!["Name", "Status"];
                rows = data
                    .mcps
                    .iter()
                    .map(|m| {
                        let status = format!("{:?}", m.status).to_lowercase();
                        vec![m.name.clone(), status]
                    })
                    .collect();
            }

            output.push_str(&self.format_table(&headers, &rows));
        }

        // Summary if both are empty
        if data.skills.is_empty() && data.mcps.is_empty() {
            output.push_str("No skills or MCP servers found.\n");
            output.push_str("Make sure Claude Code is installed and configured.");
        }

        Ok(output)
    }
}

impl DetailFormatter for TableFormatter {
    fn format_detail(&self, item: &DetailItem, raw: bool) -> Result<String, CliError> {
        match item {
            DetailItem::Skill(skill) => self.format_skill_detail(skill, raw),
            DetailItem::Mcp(mcp) => self.format_mcp_detail(mcp),
        }
    }
}

impl TableFormatter {
    fn format_skill_detail(&self, skill: &SkillDetail, raw: bool) -> Result<String, CliError> {
        if raw {
            // Raw mode: read and return entire file content
            let content = std::fs::read_to_string(&skill.path)?;
            return Ok(content);
        }

        let mut output = String::new();

        // Header with name
        output.push_str(&format!("Skill: {}\n", skill.name));
        output.push_str(&"=".repeat(skill.name.len() + 7));
        output.push_str("\n\n");

        // Start matter
        if skill.start_matter.description.is_some() {
            output.push_str("## Description\n\n");
            output.push_str(skill.start_matter.description.as_deref().unwrap_or(""));
            output.push_str("\n\n");
        }

        // Content
        output.push_str("## Content\n\n");
        output.push_str(&skill.content);
        output.push('\n');

        // Metadata footer
        output.push_str(&format!("\n---\nPath: {}\n", skill.path.display()));

        Ok(output)
    }

    fn format_mcp_detail(&self, mcp: &McpDetail) -> Result<String, CliError> {
        let mut output = String::new();

        output.push_str(&format!("MCP Server: {}\n", mcp.name));
        output.push_str(&"=".repeat(mcp.name.len() + 12));
        output.push_str("\n\n");

        output.push_str(&format!("Source: {}\n", mcp.source_type));
        output.push_str(&format!("Config Path: {}\n\n", mcp.source_path.display()));

        output.push_str("## Configuration\n\n");

        if let Some(ref cmd) = mcp.config.command {
            output.push_str(&format!("Command: `{}`\n\n", cmd));
        }

        if let Some(ref args) = mcp.config.args {
            if !args.is_empty() {
                output.push_str("Arguments:\n");
                for arg in args {
                    output.push_str(&format!("  - `{}`\n", arg));
                }
                output.push('\n');
            }
        }

        if let Some(ref env) = mcp.config.env {
            if !env.is_empty() {
                output.push_str("Environment Variables:\n");
                for (key, value) in env {
                    output.push_str(&format!("  - `{}`: `{}`\n", key, value));
                }
                output.push('\n');
            }
        }

        Ok(output)
    }
}

impl Default for TableFormatter {
    fn default() -> Self {
        Self::new()
    }
}
