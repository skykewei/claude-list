use crate::error::CliError;
use crate::model::{ClaudeList, DetailItem};
use crate::output::{DetailFormatter, Formatter};
use serde_json;

pub struct JsonFormatter;

impl JsonFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Formatter for JsonFormatter {
    fn format(&self, data: &ClaudeList) -> Result<String, CliError> {
        let json = serde_json::to_string_pretty(data)?;
        Ok(json)
    }
}

impl DetailFormatter for JsonFormatter {
    fn format_detail(&self, item: &DetailItem, raw: bool) -> Result<String, CliError> {
        if raw {
            // For raw mode with JSON formatter, we still return JSON but add raw_content field
            let mut value = serde_json::to_value(item)?;

            // If it's a skill, add raw content
            if let DetailItem::Skill(skill) = item {
                let raw_content = std::fs::read_to_string(&skill.path)?;
                if let Some(obj) = value.as_object_mut() {
                    match obj.get_mut("Skill") {
                        Some(skill_obj) => {
                            if let Some(so) = skill_obj.as_object_mut() {
                                so.insert("raw_content".to_string(), serde_json::Value::String(raw_content));
                            }
                        }
                        None => {
                            // Try "Mcp"
                            if let Some(mcp) = value.as_object_mut() {
                                mcp.insert("raw_content".to_string(), serde_json::Value::String(raw_content));
                            }
                        }
                    }
                }
            }

            return Ok(serde_json::to_string_pretty(&value)?);
        }

        // Normal JSON output
        let json = serde_json::to_string_pretty(item)?;
        Ok(json)
    }
}
