use crate::error::CliError;
use crate::model::{McpDetail, McpServer, Skill, SkillDetail};

pub mod local;

pub trait SkillSource {
    fn list_skills(&self) -> Result<Vec<Skill>, CliError>;
    fn get_skill_detail(&self, name: &str) -> Result<SkillDetail, CliError>;
}

pub trait McpSource {
    fn list_mcps(&self) -> Result<Vec<McpServer>, CliError>;
    fn get_mcp_detail(&self, name: &str) -> Result<McpDetail, CliError>;
}

pub use local::LocalSource;
