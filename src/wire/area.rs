use crate::ids::ThingsId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Area wire properties.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AreaProps {
    /// `tt`: area title.
    #[serde(rename = "tt", default)]
    pub title: String,

    /// `tg`: tag IDs applied to this area.
    #[serde(rename = "tg", default)]
    pub tag_ids: Vec<ThingsId>,

    /// `ix`: sort index.
    #[serde(rename = "ix", default)]
    pub sort_index: i32,

    /// `xx`: conflict override metadata.
    #[serde(rename = "xx", default)]
    pub conflict_overrides: Option<Value>,
}

/// Sparse patch fields for Area `t=1` updates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AreaPatch {
    /// `tt`: title.
    #[serde(rename = "tt", skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// `tg`: tag IDs.
    #[serde(rename = "tg", skip_serializing_if = "Option::is_none")]
    pub tag_ids: Option<Vec<ThingsId>>,

    /// `md`: modification timestamp.
    #[serde(rename = "md", skip_serializing_if = "Option::is_none")]
    pub modification_date: Option<f64>,

    /// `ix`: sort index.
    #[serde(rename = "ix", skip_serializing_if = "Option::is_none")]
    pub sort_index: Option<i32>,
}

impl AreaPatch {
    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.tag_ids.is_none()
            && self.modification_date.is_none()
            && self.sort_index.is_none()
    }

    pub fn into_properties(self) -> BTreeMap<String, Value> {
        match serde_json::to_value(self) {
            Ok(Value::Object(map)) => map.into_iter().collect(),
            _ => BTreeMap::new(),
        }
    }
}
