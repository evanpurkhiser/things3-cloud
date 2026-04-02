use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum TaskNotes {
    Plain(String),
    Structured(StructuredTaskNotes),
    Unknown(Value),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructuredTaskNotes {
    #[serde(rename = "_t", default)]
    pub object_type: Option<String>,
    #[serde(rename = "t")]
    pub format_type: i32,
    #[serde(default)]
    pub ch: Option<u32>,
    #[serde(default)]
    pub v: Option<String>,
    #[serde(default)]
    pub ps: Vec<StructuredTaskNoteParagraph>,
    #[serde(flatten)]
    pub unknown_fields: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructuredTaskNoteParagraph {
    #[serde(default)]
    pub r: Option<String>,
    #[serde(flatten)]
    pub unknown_fields: BTreeMap<String, Value>,
}

impl TaskNotes {
    pub fn to_plain_text(&self) -> Option<String> {
        match self {
            Self::Plain(s) => {
                let normalized = s.replace('\u{2028}', "\n").replace('\u{2029}', "\n");
                let trimmed = normalized.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            }
            Self::Structured(structured) => match structured.format_type {
                1 => structured.v.as_ref().and_then(|s| {
                    let normalized = s.replace('\u{2028}', "\n").replace('\u{2029}', "\n");
                    let trimmed = normalized.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_string())
                    }
                }),
                2 => {
                    let lines: Vec<String> =
                        structured.ps.iter().filter_map(|p| p.r.clone()).collect();
                    let joined = lines.join("\n");
                    if joined.trim().is_empty() {
                        None
                    } else {
                        Some(joined)
                    }
                }
                _ => None,
            },
            Self::Unknown(_) => None,
        }
    }
}
