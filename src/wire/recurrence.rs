use std::collections::BTreeMap;

use num_enum::{FromPrimitive, IntoPrimitive};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum::{Display, EnumString};

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
