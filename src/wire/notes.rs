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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ps: Vec<StructuredTaskNotePatch>,
    #[serde(flatten)]
    pub unknown_fields: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructuredTaskNotePatch {
    #[serde(rename = "p")]
    pub position: usize,
    #[serde(rename = "l")]
    pub length: usize,
    #[serde(rename = "r", default)]
    pub replacement: String,
    #[serde(rename = "ch", default)]
    pub checksum: Option<u32>,
    #[serde(flatten)]
    pub unknown_fields: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskNotesApplyError {
    UnsupportedFormat(i32),
    UnknownFormat,
    InvalidRange {
        position: usize,
        length: usize,
        text_length: usize,
    },
    InvalidUtf8,
    ChecksumMismatch {
        expected: u32,
        actual: u32,
    },
}

impl TaskNotes {
    pub fn to_plain_text(&self) -> Option<String> {
        self.apply_to(None).ok().flatten()
    }

    pub fn apply_to(&self, current: Option<&str>) -> Result<Option<String>, TaskNotesApplyError> {
        match self {
            Self::Plain(value) => Ok(non_empty(value.clone())),
            Self::Structured(structured) => match structured.format_type {
                1 => Ok(structured.v.clone().and_then(non_empty)),
                2 => apply_patches(current, &structured.ps),
                format_type => Err(TaskNotesApplyError::UnsupportedFormat(format_type)),
            },
            Self::Unknown(_) => Err(TaskNotesApplyError::UnknownFormat),
        }
    }
}

fn apply_patches(
    current: Option<&str>,
    patches: &[StructuredTaskNotePatch],
) -> Result<Option<String>, TaskNotesApplyError> {
    let mut text = current.unwrap_or_default().as_bytes().to_vec();

    for patch in patches {
        let Some(end) = patch.position.checked_add(patch.length) else {
            return Err(invalid_range(patch, text.len()));
        };
        if end > text.len() {
            return Err(invalid_range(patch, text.len()));
        }

        text.splice(
            patch.position..end,
            patch.replacement.as_bytes().iter().copied(),
        );

        if std::str::from_utf8(&text).is_err() {
            return Err(TaskNotesApplyError::InvalidUtf8);
        }

        if let Some(expected) = patch.checksum {
            let actual = crc32fast::hash(&text);
            if actual != expected {
                return Err(TaskNotesApplyError::ChecksumMismatch { expected, actual });
            }
        }
    }

    let text = String::from_utf8(text).map_err(|_| TaskNotesApplyError::InvalidUtf8)?;
    Ok(non_empty(text))
}

fn invalid_range(patch: &StructuredTaskNotePatch, text_length: usize) -> TaskNotesApplyError {
    TaskNotesApplyError::InvalidRange {
        position: patch.position,
        length: patch.length,
        text_length,
    }
}

fn non_empty(value: String) -> Option<String> {
    if value.is_empty() { None } else { Some(value) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn delta(position: usize, length: usize, replacement: &str, result: &str) -> TaskNotes {
        TaskNotes::Structured(StructuredTaskNotes {
            object_type: Some("tx".to_string()),
            format_type: 2,
            ch: None,
            v: None,
            ps: vec![StructuredTaskNotePatch {
                position,
                length,
                replacement: replacement.to_string(),
                checksum: Some(crc32fast::hash(result.as_bytes())),
                unknown_fields: BTreeMap::new(),
            }],
            unknown_fields: BTreeMap::new(),
        })
    }

    #[test]
    fn applies_delta_positions_as_utf8_byte_offsets() {
        let notes = delta(5, 0, "!", "café! todo");

        assert_eq!(
            notes.apply_to(Some("café todo")),
            Ok(Some("café! todo".to_string()))
        );
    }

    #[test]
    fn rejects_a_delta_that_splits_a_utf8_character() {
        let notes = delta(4, 1, "x", "unused");

        assert_eq!(
            notes.apply_to(Some("café")),
            Err(TaskNotesApplyError::InvalidUtf8)
        );
    }

    #[test]
    fn rejects_a_delta_with_the_wrong_checksum() {
        let mut notes = delta(0, 4, "done", "done");
        let TaskNotes::Structured(structured) = &mut notes else {
            unreachable!();
        };
        structured.ps[0].checksum = Some(0);

        assert!(matches!(
            notes.apply_to(Some("todo")),
            Err(TaskNotesApplyError::ChecksumMismatch { .. })
        ));
    }
}
