use crate::error::CliError;
use crate::model::{ClaudeList, DetailItem};

pub mod json;
pub mod table;

pub trait Formatter {
    fn format(&self, data: &ClaudeList) -> Result<String, CliError>;
}

pub trait DetailFormatter {
    fn format_detail(&self, item: &DetailItem, raw: bool) -> Result<String, CliError>;
}

pub use json::JsonFormatter;
pub use table::TableFormatter;
