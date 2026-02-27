use crate::error::CliError;
use crate::model::{ClaudeList, DetailItem, McpServer, Skill};
use crate::source::{LocalSource, McpSource, SkillSource};

pub struct ListService;

impl ListService {
    pub fn new() -> Self {
        Self
    }

    pub fn list_all(&self) -> Result<ClaudeList, CliError> {
        let local = LocalSource::new()?;
        let skills = local.list_skills()?;
        let mcps = local.list_mcps()?;

        Ok(ClaudeList { skills, mcps })
    }

    pub fn list_skills(&self) -> Result<Vec<Skill>, CliError> {
        let local = LocalSource::new()?;
        local.list_skills()
    }

    pub fn list_mcps(&self) -> Result<Vec<McpServer>, CliError> {
        let local = LocalSource::new()?;
        local.list_mcps()
    }

    /// Show detail of a skill or MCP server by name
    /// Tries to find a skill first, then falls back to MCP server
    pub fn show(&self, name: &str) -> Result<DetailItem, CliError> {
        let local = LocalSource::new()?;

        // Try to find skill first
        match local.get_skill_detail(name) {
            Ok(skill_detail) => return Ok(DetailItem::Skill(skill_detail)),
            Err(CliError::NotFound(_, _)) => {
                // Skill not found, try MCP
            }
            Err(e) => return Err(e),
        }

        // Fallback to MCP
        match local.get_mcp_detail(name) {
            Ok(mcp_detail) => Ok(DetailItem::Mcp(mcp_detail)),
            Err(e) => Err(e),
        }
    }
}

impl Default for ListService {
    fn default() -> Self {
        Self::new()
    }
}
