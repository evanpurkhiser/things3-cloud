use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ids::ThingsId;

/// Tombstone properties that mark a deleted object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TombstoneProps {
    /// `dloid`: deleted object UUID.
    #[serde(rename = "dloid")]
    pub deleted_object_id: ThingsId,

    /// `dld`: deletion timestamp.
    #[serde(rename = "dld", default)]
    pub delete_date: Option<f64>,
}

impl Default for TombstoneProps {
    fn default() -> Self {
        Self {
            deleted_object_id: ThingsId::default(),
            delete_date: None,
        }
    }
}

/// One-shot command properties.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CommandProps {
    /// `tp`: command type.
    #[serde(rename = "tp", default)]
    pub command_type: i32,

    /// `cd`: creation timestamp.
    #[serde(rename = "cd", default)]
    pub creation_date: Option<i64>,

    /// `if`: initial field payload for command execution.
    #[serde(rename = "if", default)]
    pub initial_fields: Option<BTreeMap<String, Value>>,
}
