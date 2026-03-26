//! Things Cloud sync protocol wire-format types.
//!
//! Observed item shape in history pages:
//! `{ uuid: { "t": operation, "e": entity, "p": properties } }`.
//! Replaying items in order by UUID yields current state.

pub mod area;
pub mod checklist;
pub mod notes;
pub mod recurrence;
pub mod tags;
pub mod task;
pub mod tombstone;
pub mod wire_object;
