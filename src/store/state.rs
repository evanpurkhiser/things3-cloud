use std::collections::HashMap;

use crate::{
    ids::ThingsId,
    store::entities::{
        AreaStateProps,
        ChecklistItemStateProps,
        StateObject,
        StateProperties,
        TagStateProps,
        TaskStateProps,
    },
    wire::{
        area::AreaPatch,
        checklist::ChecklistItemPatch,
        tags::TagPatch,
        task::TaskPatch,
        wire_object::{OperationType, Properties, WireItem, WireObject},
    },
};

pub type RawState = HashMap<ThingsId, StateObject>;

fn apply_task_patch(task: &mut TaskStateProps, patch: TaskPatch) {
    if let Some(title) = patch.title {
        task.title = title;
    }
    if let Some(notes) = patch.notes {
        task.notes = notes.to_plain_text();
    }
    if let Some(start_location) = patch.start_location {
        task.start_location = start_location;
    }
    if let Some(scheduled_date) = patch.scheduled_date {
        task.scheduled_date = scheduled_date.map(|v| v as f64);
    }
    if let Some(today_index_reference) = patch.today_index_reference {
        task.today_index_reference = today_index_reference;
    }
    if let Some(parent_project_ids) = patch.parent_project_ids {
        task.parent_project_ids = parent_project_ids;
    }
    if let Some(area_ids) = patch.area_ids {
        task.area_ids = area_ids;
    }
    if let Some(action_group_ids) = patch.action_group_ids {
        task.action_group_ids = action_group_ids;
    }
    if let Some(tag_ids) = patch.tag_ids {
        task.tag_ids = tag_ids;
    }
    if let Some(evening_bit) = patch.evening_bit {
        task.evening_bit = evening_bit;
    }
    if let Some(modification_date) = patch.modification_date {
        task.modification_date = Some(modification_date);
    }

    if let Some(item_type) = patch.item_type {
        task.item_type = item_type;
    }
    if let Some(status) = patch.status {
        task.status = status;
    }
    if let Some(stop_date) = patch.stop_date {
        task.stop_date = stop_date;
    }
    if let Some(deadline) = patch.deadline {
        task.deadline = deadline;
    }
    if let Some(sort_index) = patch.sort_index {
        task.sort_index = sort_index;
    }
    if let Some(today_sort_index) = patch.today_sort_index {
        task.today_sort_index = today_sort_index;
    }
    if let Some(recurrence_rule) = patch.recurrence_rule {
        task.recurrence_rule = recurrence_rule;
    }
    if let Some(recurrence_template_ids) = patch.recurrence_template_ids {
        task.recurrence_template_ids = recurrence_template_ids;
    }
    if let Some(instance_creation_paused) = patch.instance_creation_paused {
        task.instance_creation_paused = instance_creation_paused;
    }
    if let Some(leaves_tombstone) = patch.leaves_tombstone {
        task.leaves_tombstone = leaves_tombstone;
    }
    if let Some(trashed) = patch.trashed {
        task.trashed = trashed;
    }
    if let Some(creation_date) = patch.creation_date {
        task.creation_date = creation_date;
    }
}

fn apply_checklist_patch(item: &mut ChecklistItemStateProps, patch: ChecklistItemPatch) {
    if let Some(title) = patch.title {
        item.title = title;
    }
    if let Some(status) = patch.status {
        item.status = status;
    }
    if let Some(task_ids) = patch.task_ids {
        item.task_ids = task_ids;
    }
    if let Some(sort_index) = patch.sort_index {
        item.sort_index = sort_index;
    }
}

fn apply_area_patch(area: &mut AreaStateProps, patch: AreaPatch) {
    if let Some(title) = patch.title {
        area.title = title;
    }
    if let Some(tag_ids) = patch.tag_ids {
        area.tag_ids = tag_ids;
    }
    if let Some(sort_index) = patch.sort_index {
        area.sort_index = sort_index;
    }
}

fn apply_tag_patch(tag: &mut TagStateProps, patch: TagPatch) {
    if let Some(title) = patch.title {
        tag.title = title;
    }
    if let Some(parent_ids) = patch.parent_ids {
        tag.parent_ids = parent_ids;
    }
    if let Some(shortcut) = patch.shortcut {
        tag.shortcut = shortcut;
    }
    if let Some(sort_index) = patch.sort_index {
        tag.sort_index = sort_index;
    }
}

fn wire_object_properties(obj: &WireObject) -> StateProperties {
    match obj.properties() {
        Ok(payload) => payload.into(),
        Err(_) => StateProperties::Other,
    }
}

fn insert_state_object(state: &mut RawState, uuid: ThingsId, obj: WireObject) {
    let properties = wire_object_properties(&obj);
    state.insert(
        uuid,
        StateObject {
            entity_type: obj.entity_type,
            properties,
        },
    );
}

fn apply_update_payload(existing: &mut StateObject, payload: Properties) {
    match (&mut existing.properties, payload) {
        (StateProperties::Task(task), Properties::TaskUpdate(patch)) => {
            apply_task_patch(task, patch);
        }
        (StateProperties::ChecklistItem(item), Properties::ChecklistUpdate(patch)) => {
            apply_checklist_patch(item, patch);
        }
        (StateProperties::Area(area), Properties::AreaUpdate(patch)) => {
            apply_area_patch(area, patch);
        }
        (StateProperties::Tag(tag), Properties::TagUpdate(patch)) => {
            apply_tag_patch(tag, patch);
        }
        (_, payload) => {
            existing.properties = payload.into();
        }
    }
}

pub fn fold_item(item: WireItem, state: &mut RawState) {
    for (uuid, obj) in item {
        let uuid = ThingsId::from(uuid);
        match obj.operation_type {
            OperationType::Create => {
                insert_state_object(state, uuid, obj);
            }
            OperationType::Update => {
                if let Some(existing) = state.get_mut(&uuid) {
                    match obj.properties() {
                        Ok(payload) => apply_update_payload(existing, payload),
                        Err(_) => existing.properties = StateProperties::Other,
                    }
                    if obj.entity_type.is_some() {
                        existing.entity_type = obj.entity_type.clone();
                    }
                } else {
                    insert_state_object(state, uuid, obj);
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
