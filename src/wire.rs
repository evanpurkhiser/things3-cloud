//! Things Cloud sync protocol wire-format types.
//!
//! Observed item shape in history pages:
//! `{ uuid: { "t": operation, "e": entity, "p": properties } }`.
//! Replaying items in order by UUID yields current state.

use crate::things_id::WireId;
use num_enum::{FromPrimitive, IntoPrimitive};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use strum::{Display, EnumString};

pub type WireItem = BTreeMap<String, WireObject>;

/// A single wire object entry keyed by UUID.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WireObject {
    #[serde(rename = "t")]
    pub operation_type: OperationType,

    #[serde(rename = "e")]
    pub entity_type: Option<EntityType>,

    #[serde(rename = "p", default)]
    pub properties: BTreeMap<String, Value>,
}

/// Operation type for wire field `t`.
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Display,
    EnumString,
    FromPrimitive,
    IntoPrimitive,
)]
#[repr(i32)]
#[serde(from = "i32", into = "i32")]
pub enum OperationType {
    /// Full snapshot/create (replace current object state for UUID).
    Create = 0,
    /// Partial update (merge `p` into existing properties).
    Update = 1,
    /// Deletion event.
    Delete = 2,

    /// Unknown operation value preserved for forward compatibility.
    #[num_enum(catch_all)]
    #[strum(disabled, to_string = "{0}")]
    Unknown(i32),
}

#[expect(
    clippy::derivable_impls,
    reason = "num_enum(catch_all) conflicts with #[default]"
)]
impl Default for OperationType {
    fn default() -> Self {
        Self::Create
    }
}

/// Entity type for wire field `e`.
///
/// Values are versioned by Things (for example `Task6`, `Area3`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
#[serde(from = "String", into = "String")]
pub enum EntityType {
    /// Task/project/heading entity (current observed version).
    Task6,
    /// Checklist item entity (current observed version).
    ChecklistItem3,
    /// Tag entity (current observed version).
    Tag4,
    /// Area entity (current observed version).
    Area3,
    /// Settings entity.
    Settings5,
    /// Tombstone marker for deleted objects.
    Tombstone2,
    /// One-shot command entity.
    Command,
    /// Unknown entity name preserved for forward compatibility.
    #[strum(default, to_string = "{0}")]
    Unknown(String),
}

impl From<String> for EntityType {
    fn from(value: String) -> Self {
        value.parse().unwrap_or(Self::Unknown(value))
    }
}

impl From<EntityType> for String {
    fn from(value: EntityType) -> Self {
        value.to_string()
    }
}

/// Task wire properties (`p` fields for `Task6`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TaskProps {
    /// `tt`: title.
    #[serde(rename = "tt", default)]
    pub title: String,

    /// `nt`: notes payload (legacy XML or modern structured text object).
    #[serde(rename = "nt", default)]
    pub notes: Option<TaskNotes>,

    /// `tp`: task type (`Todo`, `Project`, `Heading`).
    #[serde(rename = "tp", default)]
    pub item_type: TaskType,

    /// `ss`: task status (`Incomplete`, `Canceled`, `Completed`).
    #[serde(rename = "ss", default)]
    pub status: TaskStatus,

    /// `sp`: completion/cancellation timestamp.
    #[serde(rename = "sp", default)]
    pub stop_date: Option<f64>,

    /// `st`: list location (`Inbox`, `Anytime`, `Someday`).
    #[serde(rename = "st", default)]
    pub start_location: TaskStart,

    /// `sr`: scheduled/start day timestamp.
    #[serde(rename = "sr", default)]
    pub scheduled_date: Option<i64>,

    /// `tir`: today index reference day timestamp.
    #[serde(rename = "tir", default)]
    pub today_index_reference: Option<i64>,

    /// `dd`: deadline day timestamp.
    #[serde(rename = "dd", default)]
    pub deadline: Option<i64>,

    /// `dds`: deadline suppressed day timestamp (rare/usually null in observed data).
    #[serde(rename = "dds", default)]
    pub deadline_suppressed_date: Option<Value>,

    /// `pr`: parent project IDs (typically 0 or 1).
    #[serde(rename = "pr", default)]
    pub parent_project_ids: Vec<WireId>,

    /// `ar`: area IDs (typically 0 or 1).
    #[serde(rename = "ar", default)]
    pub area_ids: Vec<WireId>,

    /// `agr`: heading/action-group IDs (typically 0 or 1).
    #[serde(rename = "agr", default)]
    pub action_group_ids: Vec<WireId>,

    /// `tg`: applied tag IDs.
    #[serde(rename = "tg", default)]
    pub tag_ids: Vec<WireId>,

    /// `ix`: structural sort index in its container.
    #[serde(rename = "ix", default)]
    pub sort_index: i32,

    /// `ti`: Today-view sort index.
    #[serde(rename = "ti", default)]
    pub today_sort_index: i32,

    /// `do`: due date offset (observed as `0` in typical payloads).
    #[serde(rename = "do", default)]
    pub due_date_offset: i32,

    /// `rr`: recurrence rule object (`null` for non-recurring).
    #[serde(rename = "rr", default)]
    pub recurrence_rule: Option<RecurrenceRule>,

    /// `rt`: recurrence template IDs (instance -> template link).
    #[serde(rename = "rt", default)]
    pub recurrence_template_ids: Vec<WireId>,

    /// `icsd`: instance creation suppressed date timestamp for recurrence templates.
    #[serde(rename = "icsd", default)]
    pub instance_creation_suppressed_date: Option<i64>,

    /// `acrd`: after-completion reference date timestamp for recurrence scheduling.
    #[serde(rename = "acrd", default)]
    pub after_completion_reference_date: Option<i64>,

    /// `icc`: checklist item count.
    #[serde(rename = "icc", default)]
    pub checklist_item_count: i32,

    /// `icp`: instance creation paused flag.
    #[serde(rename = "icp", default)]
    pub instance_creation_paused: bool,

    /// `ato`: alarm time offset in seconds from day start.
    #[serde(rename = "ato", default)]
    pub alarm_time_offset: Option<i64>,

    /// `lai`: last alarm interaction timestamp.
    #[serde(rename = "lai", default)]
    pub last_alarm_interaction: Option<f64>,

    /// `sb`: evening section bit (`1` evening, `0` normal).
    #[serde(rename = "sb", default)]
    pub evening_bit: i32,

    /// `lt`: leaves tombstone when deleted.
    #[serde(rename = "lt", default)]
    pub leaves_tombstone: bool,

    /// `tr`: trashed state.
    #[serde(rename = "tr", default)]
    pub trashed: bool,

    /// `dl`: deadline list metadata (rarely used, often empty).
    #[serde(rename = "dl", default)]
    pub deadline_list: Vec<Value>,

    /// `xx`: conflict override metadata (CRDT internals).
    #[serde(rename = "xx", default)]
    pub conflict_overrides: Option<Value>,

    /// `cd`: creation timestamp.
    #[serde(rename = "cd", default)]
    pub creation_date: Option<f64>,

    /// `md`: last user-modification timestamp.
    #[serde(rename = "md", default)]
    pub modification_date: Option<f64>,
}

/// Sparse patch fields for Task `t=1` updates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TaskPatch {
    /// `tt`: title.
    #[serde(rename = "tt", skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// `nt`: notes payload.
    #[serde(rename = "nt", skip_serializing_if = "Option::is_none")]
    pub notes: Option<TaskNotes>,

    /// `st`: start location.
    #[serde(rename = "st", skip_serializing_if = "Option::is_none")]
    pub start_location: Option<TaskStart>,

    /// `sr`: scheduled day timestamp (`null` clears date).
    #[serde(rename = "sr", skip_serializing_if = "Option::is_none")]
    pub scheduled_date: Option<Option<i64>>,

    /// `tir`: today reference day timestamp (`null` clears today placement).
    #[serde(rename = "tir", skip_serializing_if = "Option::is_none")]
    pub today_index_reference: Option<Option<i64>>,

    /// `pr`: parent project IDs.
    #[serde(rename = "pr", skip_serializing_if = "Option::is_none")]
    pub parent_project_ids: Option<Vec<WireId>>,

    /// `ar`: area IDs.
    #[serde(rename = "ar", skip_serializing_if = "Option::is_none")]
    pub area_ids: Option<Vec<WireId>>,

    /// `agr`: heading/action-group IDs.
    #[serde(rename = "agr", skip_serializing_if = "Option::is_none")]
    pub action_group_ids: Option<Vec<WireId>>,

    /// `tg`: tag IDs.
    #[serde(rename = "tg", skip_serializing_if = "Option::is_none")]
    pub tag_ids: Option<Vec<WireId>>,

    /// `sb`: evening section bit (`1` evening, `0` normal).
    #[serde(rename = "sb", skip_serializing_if = "Option::is_none")]
    pub evening_bit: Option<i32>,

    /// `md`: modification timestamp.
    #[serde(rename = "md", skip_serializing_if = "Option::is_none")]
    pub modification_date: Option<f64>,
}

impl TaskPatch {
    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.notes.is_none()
            && self.start_location.is_none()
            && self.scheduled_date.is_none()
            && self.today_index_reference.is_none()
            && self.parent_project_ids.is_none()
            && self.area_ids.is_none()
            && self.action_group_ids.is_none()
            && self.tag_ids.is_none()
            && self.evening_bit.is_none()
            && self.modification_date.is_none()
    }

    pub fn into_properties(self) -> BTreeMap<String, Value> {
        match serde_json::to_value(self) {
            Ok(Value::Object(map)) => map.into_iter().collect(),
            _ => BTreeMap::new(),
        }
    }
}

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

/// Task kind used in `tp`.
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Display,
    EnumString,
    FromPrimitive,
    IntoPrimitive,
)]
#[repr(i32)]
#[serde(from = "i32", into = "i32")]
pub enum TaskType {
    /// Regular leaf task.
    Todo = 0,
    /// Project container.
    Project = 1,
    /// Heading/section under a project.
    Heading = 2,

    /// Unknown value preserved for forward compatibility.
    #[num_enum(catch_all)]
    #[strum(disabled, to_string = "{0}")]
    Unknown(i32),
}

#[expect(
    clippy::derivable_impls,
    reason = "num_enum(catch_all) conflicts with #[default]"
)]
impl Default for TaskType {
    fn default() -> Self {
        Self::Todo
    }
}

/// Task status used in `ss`.
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Display,
    EnumString,
    FromPrimitive,
    IntoPrimitive,
)]
#[repr(i32)]
#[serde(from = "i32", into = "i32")]
pub enum TaskStatus {
    /// Open/incomplete.
    Incomplete = 0,
    /// Canceled.
    Canceled = 2,
    /// Completed.
    Completed = 3,

    /// Unknown value preserved for forward compatibility.
    #[num_enum(catch_all)]
    #[strum(disabled, to_string = "{0}")]
    Unknown(i32),
}

#[expect(
    clippy::derivable_impls,
    reason = "num_enum(catch_all) conflicts with #[default]"
)]
impl Default for TaskStatus {
    fn default() -> Self {
        Self::Incomplete
    }
}

/// Start location used in `st`.
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Display,
    EnumString,
    FromPrimitive,
    IntoPrimitive,
)]
#[repr(i32)]
#[serde(from = "i32", into = "i32")]
pub enum TaskStart {
    /// Inbox list.
    Inbox = 0,
    /// Anytime list.
    Anytime = 1,
    /// Someday list.
    Someday = 2,

    /// Unknown value preserved for forward compatibility.
    #[num_enum(catch_all)]
    #[strum(disabled, to_string = "{0}")]
    Unknown(i32),
}

#[expect(
    clippy::derivable_impls,
    reason = "num_enum(catch_all) conflicts with #[default]"
)]
impl Default for TaskStart {
    fn default() -> Self {
        Self::Inbox
    }
}

/// Recurrence rule payload (`rr`) for recurring templates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RecurrenceRule {
    /// `tp`: recurrence mode.
    #[serde(rename = "tp", default)]
    pub repeat_type: RecurrenceType,

    /// `fu`: frequency unit bitmask.
    #[serde(rename = "fu", default = "default_frequency_unit")]
    pub frequency_unit: FrequencyUnit,

    /// `fa`: frequency amount (every N units).
    #[serde(rename = "fa", default = "default_frequency_amount")]
    pub frequency_amount: i32,

    /// `of`: offsets (weekday/day/ordinal selectors).
    #[serde(rename = "of", default)]
    pub offsets: Vec<BTreeMap<String, Value>>,

    /// `sr`: recurrence start reference day timestamp.
    #[serde(rename = "sr", default)]
    pub start_reference: Option<i64>,

    /// `ia`: initial anchor day timestamp for recurrence calculations.
    #[serde(rename = "ia", default)]
    pub initial_anchor: Option<i64>,

    /// `ed`: recurrence end day timestamp (`64092211200` ~= effectively never).
    #[serde(rename = "ed", default = "default_recurrence_end_date")]
    pub end_date: i64,

    /// `rc`: repeat count.
    #[serde(rename = "rc", default)]
    pub repeat_count: i32,

    /// `ts`: task skip behavior metadata.
    #[serde(rename = "ts", default)]
    pub task_skip: i32,

    /// `rrv`: recurrence rule version.
    #[serde(rename = "rrv", default = "default_recurrence_rule_version")]
    pub recurrence_rule_version: i32,
}

/// Recurrence mode (`rr.tp`).
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Display,
    EnumString,
    FromPrimitive,
    IntoPrimitive,
)]
#[repr(i32)]
#[serde(from = "i32", into = "i32")]
pub enum RecurrenceType {
    /// Fixed schedule cadence.
    FixedSchedule = 0,
    /// Interval anchored after completion date.
    AfterCompletion = 1,

    /// Unknown value preserved for forward compatibility.
    #[num_enum(catch_all)]
    #[strum(disabled, to_string = "{0}")]
    Unknown(i32),
}

#[expect(
    clippy::derivable_impls,
    reason = "num_enum(catch_all) conflicts with #[default]"
)]
impl Default for RecurrenceType {
    fn default() -> Self {
        Self::FixedSchedule
    }
}

/// Recurrence frequency unit (`rr.fu`).
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Display,
    EnumString,
    FromPrimitive,
    IntoPrimitive,
)]
#[repr(i32)]
#[serde(from = "i32", into = "i32")]
pub enum FrequencyUnit {
    /// Daily bitmask value `8`.
    Daily = 8,
    /// Monthly bitmask value `16`.
    Monthly = 16,
    /// Weekly bitmask value `256`.
    Weekly = 256,

    /// Unknown value preserved for forward compatibility.
    #[num_enum(catch_all)]
    #[strum(disabled, to_string = "{0}")]
    Unknown(i32),
}

#[expect(
    clippy::derivable_impls,
    reason = "num_enum(catch_all) conflicts with #[default]"
)]
impl Default for FrequencyUnit {
    fn default() -> Self {
        Self::Weekly
    }
}

/// Checklist item wire properties.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ChecklistItemProps {
    /// `tt`: checklist item title.
    #[serde(rename = "tt", default)]
    pub title: String,

    /// `ss`: checklist item status.
    #[serde(rename = "ss", default)]
    pub status: TaskStatus,

    /// `sp`: completion/cancellation timestamp.
    #[serde(rename = "sp", default)]
    pub stop_date: Option<f64>,

    /// `ts`: parent task IDs (normally a single task UUID).
    #[serde(rename = "ts", default)]
    pub task_ids: Vec<WireId>,

    /// `ix`: sort index within checklist.
    #[serde(rename = "ix", default)]
    pub sort_index: i32,

    /// `cd`: creation timestamp.
    #[serde(rename = "cd", default)]
    pub creation_date: Option<f64>,

    /// `md`: modification timestamp.
    #[serde(rename = "md", default)]
    pub modification_date: Option<f64>,

    /// `lt`: leaves tombstone on delete.
    #[serde(rename = "lt", default)]
    pub leaves_tombstone: bool,

    /// `xx`: conflict override metadata.
    #[serde(rename = "xx", default)]
    pub conflict_overrides: Option<Value>,
}

/// Sparse patch fields for ChecklistItem `t=1` updates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ChecklistItemPatch {
    /// `tt`: title.
    #[serde(rename = "tt", skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// `ss`: status.
    #[serde(rename = "ss", skip_serializing_if = "Option::is_none")]
    pub status: Option<TaskStatus>,

    /// `ts`: parent task IDs.
    #[serde(rename = "ts", skip_serializing_if = "Option::is_none")]
    pub task_ids: Option<Vec<WireId>>,

    /// `ix`: sort index.
    #[serde(rename = "ix", skip_serializing_if = "Option::is_none")]
    pub sort_index: Option<i32>,

    /// `cd`: creation timestamp.
    #[serde(rename = "cd", skip_serializing_if = "Option::is_none")]
    pub creation_date: Option<f64>,

    /// `md`: modification timestamp.
    #[serde(rename = "md", skip_serializing_if = "Option::is_none")]
    pub modification_date: Option<f64>,
}

impl ChecklistItemPatch {
    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.status.is_none()
            && self.task_ids.is_none()
            && self.sort_index.is_none()
            && self.creation_date.is_none()
            && self.modification_date.is_none()
    }

    pub fn into_properties(self) -> BTreeMap<String, Value> {
        match serde_json::to_value(self) {
            Ok(Value::Object(map)) => map.into_iter().collect(),
            _ => BTreeMap::new(),
        }
    }
}

/// Tag wire properties.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TagProps {
    /// `tt`: tag title.
    #[serde(rename = "tt", default)]
    pub title: String,

    /// `sh`: keyboard shortcut.
    #[serde(rename = "sh", default)]
    pub shortcut: Option<String>,

    /// `ix`: sort index.
    #[serde(rename = "ix", default)]
    pub sort_index: i32,

    /// `pn`: parent tag IDs (supports nesting).
    #[serde(rename = "pn", default)]
    pub parent_ids: Vec<WireId>,

    /// `xx`: conflict override metadata.
    #[serde(rename = "xx", default)]
    pub conflict_overrides: Option<Value>,
}

/// Sparse patch fields for Tag `t=1` updates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TagPatch {
    /// `tt`: title.
    #[serde(rename = "tt", skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// `pn`: parent tag IDs.
    #[serde(rename = "pn", skip_serializing_if = "Option::is_none")]
    pub parent_ids: Option<Vec<WireId>>,

    /// `md`: modification timestamp.
    #[serde(rename = "md", skip_serializing_if = "Option::is_none")]
    pub modification_date: Option<f64>,
}

impl TagPatch {
    pub fn is_empty(&self) -> bool {
        self.title.is_none() && self.parent_ids.is_none() && self.modification_date.is_none()
    }

    pub fn into_properties(self) -> BTreeMap<String, Value> {
        match serde_json::to_value(self) {
            Ok(Value::Object(map)) => map.into_iter().collect(),
            _ => BTreeMap::new(),
        }
    }
}

/// Area wire properties.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AreaProps {
    /// `tt`: area title.
    #[serde(rename = "tt", default)]
    pub title: String,

    /// `tg`: tag IDs applied to this area.
    #[serde(rename = "tg", default)]
    pub tag_ids: Vec<WireId>,

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
    pub tag_ids: Option<Vec<WireId>>,

    /// `md`: modification timestamp.
    #[serde(rename = "md", skip_serializing_if = "Option::is_none")]
    pub modification_date: Option<f64>,
}

impl AreaPatch {
    pub fn is_empty(&self) -> bool {
        self.title.is_none() && self.tag_ids.is_none() && self.modification_date.is_none()
    }

    pub fn into_properties(self) -> BTreeMap<String, Value> {
        match serde_json::to_value(self) {
            Ok(Value::Object(map)) => map.into_iter().collect(),
            _ => BTreeMap::new(),
        }
    }
}

/// Tombstone properties that mark a deleted object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TombstoneProps {
    /// `dloid`: deleted object UUID.
    #[serde(rename = "dloid")]
    pub deleted_object_id: WireId,

    /// `dld`: deletion timestamp.
    #[serde(rename = "dld", default)]
    pub delete_date: Option<f64>,
}

impl Default for TombstoneProps {
    fn default() -> Self {
        Self {
            deleted_object_id: WireId::default(),
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

/// Default recurrence frequency unit (`rr.fu`) is weekly.
fn default_frequency_unit() -> FrequencyUnit {
    FrequencyUnit::Weekly
}

/// Default recurrence frequency amount (`rr.fa`) is every 1 unit.
const fn default_frequency_amount() -> i32 {
    1
}

/// Default recurrence end date (`rr.ed`) far in the future (~year 4001).
const fn default_recurrence_end_date() -> i64 {
    64_092_211_200
}

/// Current observed recurrence rule version (`rrv`).
const fn default_recurrence_rule_version() -> i32 {
    4
}
