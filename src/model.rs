use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct SkillDetail {
    pub name: String,
    pub start_matter: SkillStartMatter,
    pub content: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SkillStartMatter {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct McpDetail {
    pub name: String,
    pub config: crate::model::McpConfig,
    pub source_path: PathBuf,
    pub source_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub enum DetailItem {
    Skill(SkillDetail),
    Mcp(McpDetail),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub version: Option<String>,
    pub source: SourceType,
    pub path: Option<PathBuf>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub name: String,
    pub status: ConnectionStatus,
    pub config: Option<McpConfig>,
    pub source: SourceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    Local,
    Api,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Unknown,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClaudeList {
    pub skills: Vec<Skill>,
    pub mcps: Vec<McpServer>,
}

impl Skill {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: None,
            source: SourceType::Local,
            path: None,
            description: None,
        }
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn with_source(mut self, source: SourceType) -> Self {
        self.source = source;
        self
    }

    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.path = Some(path);
        self
    }
}

impl McpServer {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: ConnectionStatus::Unknown,
            config: None,
            source: SourceType::Local,
        }
    }

    pub fn with_status(mut self, status: ConnectionStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_source(mut self, source: SourceType) -> Self {
        self.source = source;
        self
    }
}
