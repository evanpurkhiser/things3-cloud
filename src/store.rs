use crate::things_id::{ThingsId, WireId};
use crate::wire::wire_object::WireItem;
use crate::wire::area::{AreaPatch, AreaProps};
use crate::wire::checklist::{ChecklistItemPatch, ChecklistItemProps};
use crate::wire::notes::TaskNotes;
use crate::wire::recurrence::RecurrenceRule;
use crate::wire::tags::{TagPatch, TagProps};
use crate::wire::task::{TaskPatch, TaskProps, TaskStart, TaskStatus, TaskType};
use crate::wire::wire_object::{EntityType, OperationType, Properties, WireObject};
use chrono::{DateTime, FixedOffset, Local, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateObject {
    pub entity_type: Option<EntityType>,
    pub properties: StateProperties,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StateProperties {
    Task(TaskStateProps),
    ChecklistItem(ChecklistItemStateProps),
    Area(AreaStateProps),
    Tag(TagStateProps),
    Other,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TaskStateProps {
    pub title: String,
    pub notes: Option<String>,
    pub item_type: TaskType,
    pub status: TaskStatus,
    pub stop_date: Option<f64>,
    pub start_location: TaskStart,
    pub scheduled_date: Option<f64>,
    pub today_index_reference: Option<i64>,
    pub deadline: Option<f64>,
    pub parent_project_ids: Vec<WireId>,
    pub area_ids: Vec<WireId>,
    pub action_group_ids: Vec<WireId>,
    pub tag_ids: Vec<WireId>,
    pub sort_index: i32,
    pub today_sort_index: i32,
    pub recurrence_rule: Option<RecurrenceRule>,
    pub recurrence_template_ids: Vec<WireId>,
    pub instance_creation_paused: bool,
    pub evening_bit: i32,
    pub leaves_tombstone: bool,
    pub trashed: bool,
    pub creation_date: Option<f64>,
    pub modification_date: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ChecklistItemStateProps {
    pub title: String,
    pub status: TaskStatus,
    pub task_ids: Vec<WireId>,
    pub sort_index: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AreaStateProps {
    pub title: String,
    pub tag_ids: Vec<WireId>,
    pub sort_index: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TagStateProps {
    pub title: String,
    pub shortcut: Option<String>,
    pub sort_index: i32,
    pub parent_ids: Vec<WireId>,
}

pub type RawState = HashMap<WireId, StateObject>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub uuid: WireId,
    pub title: String,
    pub shortcut: Option<String>,
    pub index: i32,
    pub parent_uuid: Option<WireId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Area {
    pub uuid: WireId,
    pub title: String,
    pub tags: Vec<WireId>,
    pub index: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChecklistItem {
    pub uuid: WireId,
    pub title: String,
    pub task_uuid: WireId,
    pub status: TaskStatus,
    pub index: i32,
}

impl ChecklistItem {
    pub fn is_incomplete(&self) -> bool {
        self.status == TaskStatus::Incomplete
    }

    pub fn is_completed(&self) -> bool {
        self.status == TaskStatus::Completed
    }

    pub fn is_canceled(&self) -> bool {
        self.status == TaskStatus::Canceled
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProjectProgress {
    pub total: i32,
    pub done: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub uuid: WireId,
    pub title: String,
    pub status: TaskStatus,
    pub start: TaskStart,
    pub item_type: TaskType,
    pub entity: String,
    pub notes: Option<String>,
    pub project: Option<WireId>,
    pub area: Option<WireId>,
    pub action_group: Option<WireId>,
    pub tags: Vec<WireId>,
    pub trashed: bool,
    pub deadline: Option<DateTime<Utc>>,
    pub start_date: Option<DateTime<Utc>>,
    pub stop_date: Option<DateTime<Utc>>,
    pub creation_date: Option<DateTime<Utc>>,
    pub modification_date: Option<DateTime<Utc>>,
    pub index: i32,
    pub today_index: i32,
    pub today_index_reference: Option<i64>,
    pub leaves_tombstone: bool,
    pub instance_creation_paused: bool,
    pub evening: bool,
    pub recurrence_rule: Option<RecurrenceRule>,
    pub recurrence_templates: Vec<WireId>,
    pub checklist_items: Vec<ChecklistItem>,
}

impl Task {
    pub fn is_incomplete(&self) -> bool {
        self.status == TaskStatus::Incomplete
    }

    pub fn is_completed(&self) -> bool {
        self.status == TaskStatus::Completed
    }

    pub fn is_canceled(&self) -> bool {
        self.status == TaskStatus::Canceled
    }

    pub fn is_todo(&self) -> bool {
        self.item_type == TaskType::Todo
    }

    pub fn is_project(&self) -> bool {
        self.item_type == TaskType::Project
    }

    pub fn is_heading(&self) -> bool {
        self.item_type == TaskType::Heading
    }

    pub fn in_someday(&self) -> bool {
        self.start == TaskStart::Someday && self.start_date.is_none()
    }

    pub fn is_today(&self, today: &DateTime<Utc>) -> bool {
        let Some(start_date) = self.start_date else {
            return false;
        };
        if self.start != TaskStart::Anytime && self.start != TaskStart::Someday {
            return false;
        }
        start_date <= *today
    }

    pub fn is_staged_for_today(&self, today: &DateTime<Utc>) -> bool {
        let Some(start_date) = self.start_date else {
            return false;
        };
        self.start == TaskStart::Someday && start_date <= *today
    }

    pub fn is_recurrence_template(&self) -> bool {
        self.recurrence_rule.is_some() && self.recurrence_templates.is_empty()
    }

    pub fn is_recurrence_instance(&self) -> bool {
        self.recurrence_rule.is_none() && !self.recurrence_templates.is_empty()
    }
}

#[derive(Debug, Default)]
pub struct ThingsStore {
    pub tasks_by_uuid: HashMap<WireId, Task>,
    pub areas_by_uuid: HashMap<WireId, Area>,
    pub tags_by_uuid: HashMap<WireId, Tag>,
    pub tags_by_title: HashMap<String, WireId>,
    pub project_progress_by_uuid: HashMap<WireId, ProjectProgress>,
    pub short_ids: HashMap<WireId, String>,
    pub markable_ids: HashSet<WireId>,
    pub markable_ids_sorted: Vec<WireId>,
    pub area_ids_sorted: Vec<WireId>,
    pub task_ids_sorted: Vec<WireId>,
}

fn ts_to_dt(ts: Option<f64>) -> Option<DateTime<Utc>> {
    let ts = ts?;
    let mut secs = ts.floor() as i64;
    let mut nanos = ((ts - secs as f64) * 1_000_000_000_f64).round() as u32;
    if nanos >= 1_000_000_000 {
        secs += 1;
        nanos = 0;
    }
    Utc.timestamp_opt(secs, nanos).single()
}

fn fixed_local_offset() -> FixedOffset {
    let seconds = Local::now().offset().local_minus_utc();
    FixedOffset::east_opt(seconds).unwrap_or_else(|| FixedOffset::east_opt(0).expect("UTC offset"))
}

fn i64_to_f64_opt(value: Option<i64>) -> Option<f64> {
    value.map(|v| v as f64)
}

fn lcp_len(a: &str, b: &str) -> usize {
    let mut i = 0usize;
    let max = a.len().min(b.len());
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    while i < max && a_bytes[i] == b_bytes[i] {
        i += 1;
    }
    i
}

fn shortest_unique_prefixes(ids: &[WireId]) -> HashMap<WireId, String> {
    if ids.is_empty() {
        return HashMap::new();
    }

    let mut ordered = ids.to_vec();
    ordered.sort();

    let mut result = HashMap::new();
    for (i, value) in ordered.iter().enumerate() {
        let left = if i > 0 {
            lcp_len(value, &ordered[i - 1])
        } else {
            0
        };
        let right = if i + 1 < ordered.len() {
            lcp_len(value, &ordered[i + 1])
        } else {
            0
        };
        let need = left.max(right) + 1;
        result.insert(value.clone(), value.chars().take(need).collect());
    }

    result
}

fn normalize_ids(value: Value) -> Value {
    match value {
        Value::String(s) => s
            .parse::<ThingsId>()
            .ok()
            .map(Into::into)
            .map(Value::String)
            .unwrap_or(Value::String(s)),
        Value::Array(values) => Value::Array(values.into_iter().map(normalize_ids).collect()),
        Value::Object(obj) => {
            let mut out = serde_json::Map::new();
            for (k, v) in obj {
                let new_key = k
                    .parse::<ThingsId>()
                    .ok()
                    .map(Into::into)
                    .unwrap_or(k);
                out.insert(new_key, normalize_ids(v));
            }
            Value::Object(out)
        }
        other => other,
    }
}

fn normalize_item_ids(item: WireItem) -> BTreeMap<WireId, WireObject> {
    let mut normalized = BTreeMap::new();
    for (uuid, mut obj) in item {
        let new_uuid = WireId::from(uuid);
        obj.payload = normalize_payload_ids(obj.payload);
        normalized.insert(new_uuid, obj);
    }
    normalized
}

fn normalize_payload_ids(payload: Properties) -> Properties {
    match payload {
        Properties::Unknown(props) => {
            let mut new_props = BTreeMap::new();
            for (k, v) in props {
                new_props.insert(k, normalize_ids(v));
            }
            Properties::Unknown(new_props)
        }
        other => other,
    }
}

fn parse_notes_from_wire(notes: &Option<TaskNotes>) -> Option<String> {
    notes.as_ref().and_then(TaskNotes::to_plain_text)
}

fn task_state_from_props(p: &BTreeMap<String, Value>) -> TaskStateProps {
    let parsed: TaskProps = serde_json::from_value(Value::Object(
        p.clone().into_iter().collect::<serde_json::Map<String, Value>>(),
    ))
    .unwrap_or_default();

    TaskStateProps {
        title: parsed.title,
        notes: parse_notes_from_wire(&parsed.notes),
        item_type: parsed.item_type,
        status: parsed.status,
        stop_date: parsed.stop_date,
        start_location: parsed.start_location,
        scheduled_date: i64_to_f64_opt(parsed.scheduled_date),
        today_index_reference: parsed.today_index_reference,
        deadline: i64_to_f64_opt(parsed.deadline),
        parent_project_ids: parsed.parent_project_ids,
        area_ids: parsed.area_ids,
        action_group_ids: parsed.action_group_ids,
        tag_ids: parsed.tag_ids,
        sort_index: parsed.sort_index,
        today_sort_index: parsed.today_sort_index,
        recurrence_rule: parsed.recurrence_rule,
        recurrence_template_ids: parsed.recurrence_template_ids,
        instance_creation_paused: parsed.instance_creation_paused,
        evening_bit: parsed.evening_bit,
        leaves_tombstone: parsed.leaves_tombstone,
        trashed: parsed.trashed,
        creation_date: parsed.creation_date,
        modification_date: parsed.modification_date,
    }
}

fn checklist_state_from_props(p: &BTreeMap<String, Value>) -> ChecklistItemStateProps {
    let parsed: ChecklistItemProps = serde_json::from_value(Value::Object(
        p.clone().into_iter().collect::<serde_json::Map<String, Value>>(),
    ))
    .unwrap_or_default();

    ChecklistItemStateProps {
        title: parsed.title,
        status: parsed.status,
        task_ids: parsed.task_ids,
        sort_index: parsed.sort_index,
    }
}

fn area_state_from_props(p: &BTreeMap<String, Value>) -> AreaStateProps {
    let parsed: AreaProps = serde_json::from_value(Value::Object(
        p.clone().into_iter().collect::<serde_json::Map<String, Value>>(),
    ))
    .unwrap_or_default();

    AreaStateProps {
        title: parsed.title,
        tag_ids: parsed.tag_ids,
        sort_index: parsed.sort_index,
    }
}

fn tag_state_from_props(p: &BTreeMap<String, Value>) -> TagStateProps {
    let parsed: TagProps = serde_json::from_value(Value::Object(
        p.clone().into_iter().collect::<serde_json::Map<String, Value>>(),
    ))
    .unwrap_or_default();

    TagStateProps {
        title: parsed.title,
        shortcut: parsed.shortcut,
        sort_index: parsed.sort_index,
        parent_ids: parsed.parent_ids,
    }
}

fn properties_from_wire(entity_type: Option<&EntityType>, p: &BTreeMap<String, Value>) -> StateProperties {
    match entity_type {
        Some(EntityType::Task6) => StateProperties::Task(task_state_from_props(p)),
        Some(EntityType::ChecklistItem3) => {
            StateProperties::ChecklistItem(checklist_state_from_props(p))
        }
        Some(EntityType::Area3) => StateProperties::Area(area_state_from_props(p)),
        Some(EntityType::Tag4) => StateProperties::Tag(tag_state_from_props(p)),
        Some(EntityType::Unknown(name)) if name.starts_with("Task") => {
            StateProperties::Task(task_state_from_props(p))
        }
        Some(EntityType::Unknown(name)) if name.starts_with("Area") => {
            StateProperties::Area(area_state_from_props(p))
        }
        Some(EntityType::Unknown(name)) if name.starts_with("Tag") => {
            StateProperties::Tag(tag_state_from_props(p))
        }
        _ => StateProperties::Other,
    }
}

fn apply_task_patch(task: &mut TaskStateProps, patch: &BTreeMap<String, Value>) {
    let patch_typed: TaskPatch = serde_json::from_value(Value::Object(
        patch
            .clone()
            .into_iter()
            .collect::<serde_json::Map<String, Value>>(),
    ))
    .unwrap_or_default();

    if let Some(title) = patch_typed.title {
        task.title = title;
    }
    if let Some(notes) = patch_typed.notes {
        task.notes = notes.to_plain_text();
    }
    if let Some(start_location) = patch_typed.start_location {
        task.start_location = start_location;
    }
    if let Some(scheduled_date) = patch_typed.scheduled_date {
        task.scheduled_date = scheduled_date.map(|v| v as f64);
    }
    if let Some(today_index_reference) = patch_typed.today_index_reference {
        task.today_index_reference = today_index_reference;
    }
    if let Some(parent_project_ids) = patch_typed.parent_project_ids {
        task.parent_project_ids = parent_project_ids;
    }
    if let Some(area_ids) = patch_typed.area_ids {
        task.area_ids = area_ids;
    }
    if let Some(action_group_ids) = patch_typed.action_group_ids {
        task.action_group_ids = action_group_ids;
    }
    if let Some(tag_ids) = patch_typed.tag_ids {
        task.tag_ids = tag_ids;
    }
    if let Some(evening_bit) = patch_typed.evening_bit {
        task.evening_bit = evening_bit;
    }
    if let Some(modification_date) = patch_typed.modification_date {
        task.modification_date = Some(modification_date);
    }

    if let Some(item_type) = patch_typed.item_type {
        task.item_type = item_type;
    }
    if let Some(status) = patch_typed.status {
        task.status = status;
    }
    if let Some(stop_date) = patch_typed.stop_date {
        task.stop_date = stop_date;
    }
    if let Some(deadline) = patch_typed.deadline {
        task.deadline = deadline;
    }
    if let Some(sort_index) = patch_typed.sort_index {
        task.sort_index = sort_index;
    }
    if let Some(today_sort_index) = patch_typed.today_sort_index {
        task.today_sort_index = today_sort_index;
    }
    if let Some(recurrence_rule) = patch_typed.recurrence_rule {
        task.recurrence_rule = recurrence_rule;
    }
    if let Some(recurrence_template_ids) = patch_typed.recurrence_template_ids {
        task.recurrence_template_ids = recurrence_template_ids;
    }
    if let Some(instance_creation_paused) = patch_typed.instance_creation_paused {
        task.instance_creation_paused = instance_creation_paused;
    }
    if let Some(leaves_tombstone) = patch_typed.leaves_tombstone {
        task.leaves_tombstone = leaves_tombstone;
    }
    if let Some(trashed) = patch_typed.trashed {
        task.trashed = trashed;
    }
    if let Some(creation_date) = patch_typed.creation_date {
        task.creation_date = creation_date;
    }
}

fn apply_checklist_patch(item: &mut ChecklistItemStateProps, patch: &BTreeMap<String, Value>) {
    let parsed: ChecklistItemPatch = serde_json::from_value(Value::Object(
        patch
            .clone()
            .into_iter()
            .collect::<serde_json::Map<String, Value>>(),
    ))
    .unwrap_or_default();

    if let Some(title) = parsed.title {
        item.title = title;
    }
    if let Some(status) = parsed.status {
        item.status = status;
    }
    if let Some(task_ids) = parsed.task_ids {
        item.task_ids = task_ids;
    }
    if let Some(sort_index) = parsed.sort_index {
        item.sort_index = sort_index;
    }
}

fn apply_area_patch(area: &mut AreaStateProps, patch: &BTreeMap<String, Value>) {
    let parsed: AreaPatch = serde_json::from_value(Value::Object(
        patch
            .clone()
            .into_iter()
            .collect::<serde_json::Map<String, Value>>(),
    ))
    .unwrap_or_default();

    if let Some(title) = parsed.title {
        area.title = title;
    }
    if let Some(tag_ids) = parsed.tag_ids {
        area.tag_ids = tag_ids;
    }

    if let Some(sort_index) = parsed.sort_index {
        area.sort_index = sort_index;
    }
}

fn apply_tag_patch(tag: &mut TagStateProps, patch: &BTreeMap<String, Value>) {
    let parsed: TagPatch = serde_json::from_value(Value::Object(
        patch
            .clone()
            .into_iter()
            .collect::<serde_json::Map<String, Value>>(),
    ))
    .unwrap_or_default();

    if let Some(title) = parsed.title {
        tag.title = title;
    }
    if let Some(parent_ids) = parsed.parent_ids {
        tag.parent_ids = parent_ids;
    }
    if let Some(shortcut) = parsed.shortcut {
        tag.shortcut = shortcut;
    }
    if let Some(sort_index) = parsed.sort_index {
        tag.sort_index = sort_index;
    }
}

pub fn fold_item(item: WireItem, state: &mut RawState) {
    let normalized = normalize_item_ids(item);

    for (uuid, obj) in normalized {
        match obj.operation_type {
            OperationType::Create => {
                let properties = match obj.properties() {
                    Ok(Properties::TaskCreate(props)) => StateProperties::Task(TaskStateProps {
                        title: props.title,
                        notes: parse_notes_from_wire(&props.notes),
                        item_type: props.item_type,
                        status: props.status,
                        stop_date: props.stop_date,
                        start_location: props.start_location,
                        scheduled_date: i64_to_f64_opt(props.scheduled_date),
                        today_index_reference: props.today_index_reference,
                        deadline: i64_to_f64_opt(props.deadline),
                        parent_project_ids: props.parent_project_ids,
                        area_ids: props.area_ids,
                        action_group_ids: props.action_group_ids,
                        tag_ids: props.tag_ids,
                        sort_index: props.sort_index,
                        today_sort_index: props.today_sort_index,
                        recurrence_rule: props.recurrence_rule,
                        recurrence_template_ids: props.recurrence_template_ids,
                        instance_creation_paused: props.instance_creation_paused,
                        evening_bit: props.evening_bit,
                        leaves_tombstone: props.leaves_tombstone,
                        trashed: props.trashed,
                        creation_date: props.creation_date,
                        modification_date: props.modification_date,
                    }),
                    Ok(Properties::ChecklistCreate(props)) => {
                        StateProperties::ChecklistItem(ChecklistItemStateProps {
                            title: props.title,
                            status: props.status,
                            task_ids: props.task_ids,
                            sort_index: props.sort_index,
                        })
                    }
                    Ok(Properties::AreaCreate(props)) => StateProperties::Area(AreaStateProps {
                        title: props.title,
                        tag_ids: props.tag_ids,
                        sort_index: props.sort_index,
                    }),
                    Ok(Properties::TagCreate(props)) => StateProperties::Tag(TagStateProps {
                        title: props.title,
                        shortcut: props.shortcut,
                        sort_index: props.sort_index,
                        parent_ids: props.parent_ids,
                    }),
                    Ok(Properties::Unknown(_))
                    | Ok(Properties::Delete)
                    | Ok(Properties::TaskUpdate(_))
                    | Ok(Properties::ChecklistUpdate(_))
                    | Ok(Properties::AreaUpdate(_))
                    | Ok(Properties::TagUpdate(_))
                    | Ok(Properties::TombstoneCreate(_))
                    | Ok(Properties::CommandCreate(_))
                    | Err(_) => properties_from_wire(obj.entity_type.as_ref(), &obj.properties_map()),
                };
                state.insert(
                    uuid,
                    StateObject {
                        entity_type: obj.entity_type.clone(),
                        properties,
                    },
                );
            }
            OperationType::Update => {
                if let Some(existing) = state.get_mut(&uuid) {
                    let typed = obj.properties();
                    match (&mut existing.properties, typed) {
                        (StateProperties::Task(task), Ok(Properties::TaskUpdate(patch))) => {
                            let raw = patch.into_properties();
                            apply_task_patch(task, &raw);
                        }
                        (
                            StateProperties::ChecklistItem(item),
                            Ok(Properties::ChecklistUpdate(patch)),
                        ) => {
                            let raw = patch.into_properties();
                            apply_checklist_patch(item, &raw);
                        }
                        (StateProperties::Area(area), Ok(Properties::AreaUpdate(patch))) => {
                            let raw = patch.into_properties();
                            apply_area_patch(area, &raw);
                        }
                        (StateProperties::Tag(tag), Ok(Properties::TagUpdate(patch))) => {
                            let raw = patch.into_properties();
                            apply_tag_patch(tag, &raw);
                        }
                        (_, Ok(Properties::Unknown(_))) | (_, Err(_)) => {
                            existing.properties =
                                properties_from_wire(obj.entity_type.as_ref(), &obj.properties_map());
                        }
                        (StateProperties::Other, _) => {
                            existing.properties =
                                properties_from_wire(obj.entity_type.as_ref(), &obj.properties_map());
                        }
                        _ => {}
                    }
                    if obj.entity_type.is_some() {
                        existing.entity_type = obj.entity_type.clone();
                    }
                } else {
                    let properties = properties_from_wire(obj.entity_type.as_ref(), &obj.properties_map());
                    state.insert(
                        uuid,
                        StateObject {
                            entity_type: obj.entity_type,
                            properties,
                        },
                    );
                }
            }
            OperationType::Delete => {
                state.remove(&uuid);
            }
            OperationType::Unknown(_) => {}
        }
    }
}

pub fn fold_items(items: impl IntoIterator<Item = WireItem>) -> RawState {
    let mut state = RawState::new();
    for item in items {
        fold_item(item, &mut state);
    }
    state
}

impl ThingsStore {
    pub fn from_raw_state(raw_state: &RawState) -> Self {
        let mut store = Self::default();
        store.build(raw_state);
        store.build_project_progress_index();
        store.short_ids = shortest_unique_prefixes(&store.short_id_domain(raw_state));
        store.build_mark_indexes();
        store.area_ids_sorted = store.areas_by_uuid.keys().cloned().collect();
        store.area_ids_sorted.sort();
        store.task_ids_sorted = store.tasks_by_uuid.keys().cloned().collect();
        store.task_ids_sorted.sort();
        store
    }

    fn short_id_domain(&self, raw_state: &RawState) -> Vec<WireId> {
        let mut ids = Vec::new();
        for (uuid, obj) in raw_state {
            match obj.entity_type.as_ref() {
                Some(EntityType::Tombstone2) => continue,
                Some(EntityType::Unknown(s)) if s == "Tombstone" => continue,
                _ => {}
            }

            if uuid.as_str().starts_with("TOMBSTONE-") {
                continue;
            }

            ids.push(uuid.clone());
        }
        ids
    }

    fn build_mark_indexes(&mut self) {
        let markable: Vec<&Task> = self
            .tasks_by_uuid
            .values()
            .filter(|task| !task.trashed && !task.is_heading() && task.entity == "Task6")
            .collect();

        self.markable_ids = markable.iter().map(|t| t.uuid.clone()).collect();
        self.markable_ids_sorted = self.markable_ids.iter().cloned().collect();
        self.markable_ids_sorted.sort();
    }

    fn build_project_progress_index(&mut self) {
        let mut totals: HashMap<WireId, i32> = HashMap::new();
        let mut dones: HashMap<WireId, i32> = HashMap::new();

        for task in self.tasks_by_uuid.values() {
            if task.trashed || !task.is_todo() {
                continue;
            }

            let Some(project_uuid) = self.effective_project_uuid(task) else {
                continue;
            };

            *totals.entry(project_uuid.clone()).or_insert(0) += 1;
            if task.is_completed() {
                *dones.entry(project_uuid).or_insert(0) += 1;
            }
        }

        self.project_progress_by_uuid = totals
            .into_iter()
            .map(|(project_uuid, total)| {
                let done = *dones.get(&project_uuid).unwrap_or(&0);
                (project_uuid, ProjectProgress { total, done })
            })
            .collect();
    }

    fn build(&mut self, raw_state: &RawState) {
        let mut checklist_items: Vec<ChecklistItem> = Vec::new();

        for (uuid, obj) in raw_state {
            let is_task = matches!(obj.entity_type.as_ref(), Some(EntityType::Task6))
                || matches!(obj.entity_type.as_ref(), Some(EntityType::Unknown(s)) if s.starts_with("Task"));
            let is_area = matches!(obj.entity_type.as_ref(), Some(EntityType::Area3))
                || matches!(obj.entity_type.as_ref(), Some(EntityType::Unknown(s)) if s.starts_with("Area"));
            let is_tag = matches!(obj.entity_type.as_ref(), Some(EntityType::Tag4))
                || matches!(obj.entity_type.as_ref(), Some(EntityType::Unknown(s)) if s.starts_with("Tag"));

            match obj.entity_type.as_ref() {
                _ if is_task => {
                    let entity = match obj.entity_type.as_ref() {
                        Some(EntityType::Task6) => "Task6".to_string(),
                        Some(EntityType::Unknown(s)) => s.clone(),
                        Some(other) => String::from(other.clone()),
                        None => "Task6".to_string(),
                    };
                    let StateProperties::Task(props) = &obj.properties else {
                        continue;
                    };
                    let task = self.parse_task(uuid, props, &entity);
                    self.tasks_by_uuid.insert(uuid.clone(), task);
                }
                _ if is_area => {
                    let StateProperties::Area(props) = &obj.properties else {
                        continue;
                    };
                    let area = self.parse_area(uuid, props);
                    self.areas_by_uuid.insert(uuid.clone(), area);
                }
                _ if is_tag => {
                    let StateProperties::Tag(props) = &obj.properties else {
                        continue;
                    };
                    let tag = self.parse_tag(uuid, props);
                    if !tag.title.is_empty() {
                        self.tags_by_title
                            .insert(tag.title.clone(), tag.uuid.clone());
                    }
                    self.tags_by_uuid.insert(uuid.clone(), tag);
                }
                Some(EntityType::ChecklistItem3) => {
                    if let StateProperties::ChecklistItem(props) = &obj.properties {
                        checklist_items.push(self.parse_checklist_item(uuid, props));
                    }
                }
                _ => {}
            }
        }

        let mut by_task: HashMap<WireId, Vec<ChecklistItem>> = HashMap::new();
        for item in checklist_items {
            if self.tasks_by_uuid.contains_key(&item.task_uuid) {
                by_task
                    .entry(item.task_uuid.clone())
                    .or_default()
                    .push(item);
            }
        }

        for (task_uuid, items) in by_task.iter_mut() {
            items.sort_by_key(|i| i.index);
            if let Some(task) = self.tasks_by_uuid.get_mut(task_uuid) {
                task.checklist_items = items.clone();
            }
        }
    }

    fn parse_task(&self, uuid: &WireId, p: &TaskStateProps, entity: &str) -> Task {
        Task {
            uuid: uuid.clone(),
            title: p.title.clone(),
            status: p.status,
            start: p.start_location,
            item_type: p.item_type,
            entity: entity.to_string(),
            notes: p.notes.clone(),
            project: p.parent_project_ids.first().cloned(),
            area: p.area_ids.first().cloned(),
            action_group: p.action_group_ids.first().cloned(),
            tags: p.tag_ids.clone(),
            trashed: p.trashed,
            deadline: ts_to_dt(p.deadline),
            start_date: ts_to_dt(p.scheduled_date),
            stop_date: ts_to_dt(p.stop_date),
            creation_date: ts_to_dt(p.creation_date),
            modification_date: ts_to_dt(p.modification_date),
            index: p.sort_index,
            today_index: p.today_sort_index,
            today_index_reference: p.today_index_reference,
            leaves_tombstone: p.leaves_tombstone,
            instance_creation_paused: p.instance_creation_paused,
            evening: p.evening_bit != 0,
            recurrence_rule: p.recurrence_rule.clone(),
            recurrence_templates: p.recurrence_template_ids.clone(),
            checklist_items: Vec::new(),
        }
    }

    fn parse_checklist_item(&self, uuid: &WireId, p: &ChecklistItemStateProps) -> ChecklistItem {
        ChecklistItem {
            uuid: uuid.clone(),
            title: p.title.clone(),
            task_uuid: p.task_ids.first().cloned().unwrap_or_default(),
            status: p.status,
            index: p.sort_index,
        }
    }

    fn parse_area(&self, uuid: &WireId, p: &AreaStateProps) -> Area {
        Area {
            uuid: uuid.clone(),
            title: p.title.clone(),
            tags: p.tag_ids.clone(),
            index: p.sort_index,
        }
    }

    fn parse_tag(&self, uuid: &WireId, p: &TagStateProps) -> Tag {
        Tag {
            uuid: uuid.clone(),
            title: p.title.clone(),
            shortcut: p.shortcut.clone(),
            index: p.sort_index,
            parent_uuid: p.parent_ids.first().cloned(),
        }
    }

    pub fn tasks(
        &self,
        status: Option<TaskStatus>,
        trashed: Option<bool>,
        item_type: Option<TaskType>,
    ) -> Vec<Task> {
        let mut out: Vec<Task> = self
            .tasks_by_uuid
            .values()
            .filter(|task| {
                if let Some(expect_trashed) = trashed
                    && task.trashed != expect_trashed
                {
                    return false;
                }
                if let Some(expect_status) = status
                    && task.status != expect_status
                {
                    return false;
                }
                if let Some(expect_type) = item_type
                    && task.item_type != expect_type
                {
                    return false;
                }
                if task.is_heading() {
                    return false;
                }
                true
            })
            .cloned()
            .collect();
        out.sort_by_key(|t| t.index);
        out
    }

    pub fn today(&self, today: &DateTime<Utc>) -> Vec<Task> {
        let mut out: Vec<Task> = self
            .tasks_by_uuid
            .values()
            .filter(|t| {
                !t.trashed
                    && t.status == TaskStatus::Incomplete
                    && !t.is_heading()
                    && !t.is_project()
                    && !t.title.trim().is_empty()
                    && t.entity == "Task6"
                    && t.is_today(today)
            })
            .cloned()
            .collect();

        out.sort_by_key(|task| {
            if task.today_index == 0 {
                let sr_ts = task.start_date.map(|d| d.timestamp()).unwrap_or(0);
                (0i32, Reverse(sr_ts), Reverse(task.index))
            } else {
                (1i32, Reverse(task.today_index as i64), Reverse(task.index))
            }
        });
        out
    }

    pub fn inbox(&self) -> Vec<Task> {
        let mut out: Vec<Task> = self
            .tasks_by_uuid
            .values()
            .filter(|t| {
                !t.trashed
                    && t.status == TaskStatus::Incomplete
                    && t.start == TaskStart::Inbox
                    && self.effective_project_uuid(t).is_none()
                    && self.effective_area_uuid(t).is_none()
                    && !t.is_project()
                    && !t.is_heading()
                    && !t.title.trim().is_empty()
                    && t.creation_date.is_some()
                    && t.entity == "Task6"
            })
            .cloned()
            .collect();
        out.sort_by_key(|t| t.index);
        out
    }

    pub fn anytime(&self, today: &DateTime<Utc>) -> Vec<Task> {
        let project_visible = |task: &Task, store: &ThingsStore| {
            let Some(project_uuid) = store.effective_project_uuid(task) else {
                return true;
            };
            let Some(project) = store.tasks_by_uuid.get(&project_uuid) else {
                return true;
            };
            if project.trashed || project.status != TaskStatus::Incomplete {
                return false;
            }
            if project.start == TaskStart::Someday {
                return false;
            }
            if let Some(start_date) = project.start_date
                && start_date > *today
            {
                return false;
            }
            true
        };

        let mut out: Vec<Task> = self
            .tasks_by_uuid
            .values()
            .filter(|t| {
                !t.trashed
                    && t.status == TaskStatus::Incomplete
                    && t.start == TaskStart::Anytime
                    && !t.is_project()
                    && !t.is_heading()
                    && !t.title.trim().is_empty()
                    && t.entity == "Task6"
                    && (t.start_date.is_none() || t.start_date <= Some(*today))
                    && project_visible(t, self)
            })
            .cloned()
            .collect();
        out.sort_by_key(|t| t.index);
        out
    }

    pub fn someday(&self) -> Vec<Task> {
        let mut out: Vec<Task> = self
            .tasks_by_uuid
            .values()
            .filter(|t| {
                !t.trashed
                    && t.status == TaskStatus::Incomplete
                    && t.start == TaskStart::Someday
                    && !t.is_heading()
                    && !t.title.trim().is_empty()
                    && t.entity == "Task6"
                    && !t.is_recurrence_template()
                    && t.start_date.is_none()
                    && (t.is_project() || self.effective_project_uuid(t).is_none())
            })
            .cloned()
            .collect();
        out.sort_by_key(|t| t.index);
        out
    }

    pub fn logbook(
        &self,
        from_date: Option<DateTime<Local>>,
        to_date: Option<DateTime<Local>>,
    ) -> Vec<Task> {
        let mut out: Vec<Task> = self
            .tasks_by_uuid
            .values()
            .filter(|task| {
                if task.trashed
                    || !(task.status == TaskStatus::Completed
                        || task.status == TaskStatus::Canceled)
                {
                    return false;
                }
                if task.is_heading() || task.entity != "Task6" {
                    return false;
                }
                let Some(stop_date) = task.stop_date else {
                    return false;
                };

                let stop_day = stop_date
                    .with_timezone(&fixed_local_offset())
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .and_then(|d| fixed_local_offset().from_local_datetime(&d).single())
                    .map(|d| d.with_timezone(&Local));

                if let Some(from_day) = from_date
                    && let Some(sd) = stop_day
                    && sd < from_day
                {
                    return false;
                }
                if let Some(to_day) = to_date
                    && let Some(sd) = stop_day
                    && sd > to_day
                {
                    return false;
                }

                true
            })
            .cloned()
            .collect();

        out.sort_by_key(|t| {
            let stop_key = t
                .stop_date
                .map(|d| (d.timestamp(), d.timestamp_subsec_nanos()))
                .unwrap_or((0, 0));
            (
                Reverse(stop_key),
                Reverse(t.index),
                t.uuid.clone(),
            )
        });
        out
    }

    pub fn effective_project_uuid(&self, task: &Task) -> Option<WireId> {
        if let Some(project) = &task.project {
            return Some(project.clone());
        }
        if let Some(action_group) = &task.action_group
            && let Some(heading) = self.tasks_by_uuid.get(action_group)
            && let Some(project) = &heading.project
        {
            return Some(project.clone());
        }
        None
    }

    pub fn effective_area_uuid(&self, task: &Task) -> Option<WireId> {
        if let Some(area) = &task.area {
            return Some(area.clone());
        }

        if let Some(project_uuid) = self.effective_project_uuid(task)
            && let Some(project) = self.tasks_by_uuid.get(&project_uuid)
            && let Some(area) = &project.area
        {
            return Some(area.clone());
        }

        if let Some(action_group) = &task.action_group
            && let Some(heading) = self.tasks_by_uuid.get(action_group)
            && let Some(area) = &heading.area
        {
            return Some(area.clone());
        }

        None
    }

    pub fn projects(&self, status: Option<TaskStatus>) -> Vec<Task> {
        let mut out: Vec<Task> = self
            .tasks_by_uuid
            .values()
            .filter(|t| {
                !t.trashed
                    && t.is_project()
                    && t.entity == "Task6"
                    && status.map(|s| t.status == s).unwrap_or(true)
            })
            .cloned()
            .collect();
        out.sort_by_key(|t| t.index);
        out
    }

    pub fn areas(&self) -> Vec<Area> {
        let mut out: Vec<Area> = self.areas_by_uuid.values().cloned().collect();
        out.sort_by_key(|a| a.index);
        out
    }

    pub fn tags(&self) -> Vec<Tag> {
        let mut out: Vec<Tag> = self
            .tags_by_uuid
            .values()
            .filter(|t| !t.title.trim().is_empty())
            .cloned()
            .collect();
        out.sort_by_key(|t| t.index);
        out
    }

    pub fn get_task(&self, uuid: &str) -> Option<Task> {
        self.tasks_by_uuid.get(uuid).cloned()
    }

    pub fn get_area(&self, uuid: &str) -> Option<Area> {
        self.areas_by_uuid.get(uuid).cloned()
    }

    pub fn get_tag(&self, uuid: &str) -> Option<Tag> {
        self.tags_by_uuid.get(uuid).cloned()
    }

    pub fn resolve_tag_title<T: AsRef<str>>(&self, uuid: T) -> String {
        let uuid = uuid.as_ref();
        self.tags_by_uuid
            .get(uuid)
            .filter(|t| !t.title.trim().is_empty())
            .map(|t| t.title.clone())
            .unwrap_or_else(|| uuid.to_string())
    }

    pub fn resolve_area_title<T: AsRef<str>>(&self, uuid: T) -> String {
        let uuid = uuid.as_ref();
        self.areas_by_uuid
            .get(uuid)
            .map(|a| a.title.clone())
            .unwrap_or_else(|| uuid.to_string())
    }

    pub fn resolve_project_title<T: AsRef<str>>(&self, uuid: T) -> String {
        let uuid = uuid.as_ref();
        if let Some(task) = self.tasks_by_uuid.get(uuid)
            && !task.title.trim().is_empty()
        {
            return task.title.clone();
        }
        if uuid.is_empty() {
            return "(project)".to_string();
        }
        let short: String = uuid.chars().take(8).collect();
        format!("(project {short})")
    }

    pub fn short_id<T: AsRef<str>>(&self, uuid: T) -> String {
        let uuid = uuid.as_ref();
        self.short_ids
            .get(uuid)
            .cloned()
            .unwrap_or_else(|| uuid.to_string())
    }

    pub fn project_progress<T: AsRef<str>>(&self, project_uuid: T) -> ProjectProgress {
        let project_uuid = project_uuid.as_ref();
        self.project_progress_by_uuid
            .get(project_uuid)
            .cloned()
            .unwrap_or_default()
    }

    pub fn unique_prefix_length<T: AsRef<str>>(&self, ids: &[T]) -> usize {
        if ids.is_empty() {
            return 0;
        }
        let mut max_need = 1usize;
        for id in ids {
            if let Some(short) = self.short_ids.get(id.as_ref()) {
                max_need = max_need.max(short.len());
            } else {
                max_need = max_need.max(6);
            }
        }
        max_need
    }

    fn resolve_prefix<T: Clone>(
        &self,
        identifier: &str,
        items: &HashMap<WireId, T>,
        sorted_ids: &[WireId],
        label: &str,
    ) -> (Option<T>, String, Vec<T>) {
        let ident = identifier.trim();
        if ident.is_empty() {
            return (
                None,
                format!("Missing {} identifier.", label.to_lowercase()),
                Vec::new(),
            );
        }

        if let Some(exact) = items.get(ident) {
            return (Some(exact.clone()), String::new(), Vec::new());
        }

        let matches: Vec<&WireId> = sorted_ids
            .iter()
            .filter(|id| id.starts_with(ident))
            .collect();
        if matches.len() == 1
            && let Some(item) = items.get(matches[0].as_str())
        {
            return (Some(item.clone()), String::new(), Vec::new());
        }

        if matches.len() > 1 {
            let mut out = Vec::new();
            for m in matches.iter().take(10) {
                if let Some(item) = items.get(m.as_str()) {
                    out.push(item.clone());
                }
            }
            let remaining = matches.len().saturating_sub(out.len());
            let mut msg = format!("Ambiguous {} id prefix.", label.to_lowercase());
            if remaining > 0 {
                msg.push_str(&format!(
                    " ({} matches, showing first {})",
                    matches.len(),
                    out.len()
                ));
            }
            return (None, msg, out);
        }

        (
            None,
            format!("{} not found: {}", label, identifier),
            Vec::new(),
        )
    }

    pub fn resolve_mark_identifier(&self, identifier: &str) -> (Option<Task>, String, Vec<Task>) {
        let markable: HashMap<WireId, Task> = self
            .markable_ids
            .iter()
            .filter_map(|uid| {
                self.tasks_by_uuid
                    .get(uid)
                    .map(|t| (uid.clone(), t.clone()))
            })
            .collect();
        self.resolve_prefix(identifier, &markable, &self.markable_ids_sorted, "Item")
    }

    pub fn resolve_area_identifier(&self, identifier: &str) -> (Option<Area>, String, Vec<Area>) {
        self.resolve_prefix(
            identifier,
            &self.areas_by_uuid,
            &self.area_ids_sorted,
            "Area",
        )
    }

    pub fn resolve_task_identifier(&self, identifier: &str) -> (Option<Task>, String, Vec<Task>) {
        self.resolve_prefix(
            identifier,
            &self.tasks_by_uuid,
            &self.task_ids_sorted,
            "Task",
        )
    }
}
