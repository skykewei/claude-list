use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum CliError {
    LocalConfigError(LocalSourceError),
    Io(std::io::Error),
    Serialize(serde_json::Error),
    NotFound(String, Vec<String>),
}

#[derive(Debug)]
pub enum LocalSourceError {
    ConfigNotFound(PathBuf),
    InvalidConfig(String),
    PermissionDenied(PathBuf),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::LocalConfigError(e) => write!(f, "Failed to read local configuration: {}", e),
            CliError::Io(e) => write!(f, "IO error: {}", e),
            CliError::Serialize(e) => write!(f, "Serialization error: {}", e),
            CliError::NotFound(name, suggestions) => {
                write!(f, "Resource not found: '{}'", name)?;
                if !suggestions.is_empty() {
                    write!(f, "\nDid you mean:")?;
                    for s in suggestions.iter().take(3) {
                        write!(f, "\n  - {}", s)?;
                    }
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for CliError {}

impl fmt::Display for LocalSourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocalSourceError::ConfigNotFound(p) => {
                write!(f, "Config directory not found: {}", p.display())
            }
            LocalSourceError::InvalidConfig(s) => write!(f, "Invalid config file: {}", s),
            LocalSourceError::PermissionDenied(p) => {
                write!(f, "Permission denied: {}", p.display())
            }
        }
    }
}

impl std::error::Error for LocalSourceError {}

impl From<LocalSourceError> for CliError {
    fn from(e: LocalSourceError) -> Self {
        CliError::LocalConfigError(e)
    }
}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        CliError::Io(e)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(e: serde_json::Error) -> Self {
        CliError::Serialize(e)
    }
}
