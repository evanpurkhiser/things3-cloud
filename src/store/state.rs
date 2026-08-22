use std::collections::HashMap;

use crate::{
    ids::ThingsId,
    store::entities::{
        AreaStateProps, ChecklistItemStateProps, StateObject, StateProperties, TagStateProps,
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
    if let Some(repeater) = patch.repeater {
        task.repeater = repeater;
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
            apply_task_patch(task, *patch);
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
        (_, Properties::Ignored(_) | Properties::Unknown(_)) => {}
        (_, payload) => {
            existing.properties = payload.into();
        }
    }
}

pub fn fold_item(item: WireItem, state: &mut RawState) {
    for (uuid, obj) in item {
        let Ok(uuid) = uuid.parse::<ThingsId>() else {
            continue;
        };
        match obj.operation_type {
            OperationType::Create => {
                insert_state_object(state, uuid, obj);
            }
            OperationType::Update => {
                if let Some(existing) = state.get_mut(&uuid) {
                    if let Ok(payload) = obj.properties() {
                        apply_update_payload(existing, payload);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        store::ThingsStore,
        wire::{
            task::{TaskStatus, TaskType},
            wire_object::EntityType,
        },
    };

    const TASK_ID: &str = "A7h5eCi24RvAWKC3Hv3muf";

    fn wire_item(json: &str) -> WireItem {
        serde_json::from_str(json).expect("test wire item should deserialize")
    }

    fn task6_create() -> WireItem {
        wire_item(&format!(
            r#"{{"{TASK_ID}":{{"t":0,"e":"Task6","p":{{"tt":"Send tracking number","tp":0,"ss":0,"st":1,"cd":1.0,"md":1.0}}}}}}"#
        ))
    }

    #[test]
    fn task7_update_preserves_task_state_and_promotes_entity() {
        let update = wire_item(&format!(
            r#"{{"{TASK_ID}":{{"t":1,"e":"Task7","p":{{"ss":0,"sp":null,"md":2.0}}}}}}"#
        ));
        let state = fold_items([task6_create(), update]);
        let task_id = TASK_ID.parse::<ThingsId>().expect("valid task id");
        let object = state.get(&task_id).expect("task state should remain");

        assert_eq!(object.entity_type, Some(EntityType::Task7));
        let StateProperties::Task(properties) = &object.properties else {
            panic!("Task7 update replaced typed task state");
        };
        assert_eq!(properties.title, "Send tracking number");
        assert_eq!(properties.status, TaskStatus::Incomplete);
        assert_eq!(properties.modification_date, Some(2.0));

        let store = ThingsStore::from_raw_state(&state);
        let task = store
            .get_task(TASK_ID)
            .expect("Task7 task should be visible");
        assert_eq!(task.item_type, TaskType::Todo);
        assert_eq!(task.entity, EntityType::Task7);
    }

    #[test]
    fn unknown_future_task_update_does_not_destroy_known_state() {
        let update = wire_item(&format!(
            r#"{{"{TASK_ID}":{{"t":1,"e":"Task8","p":{{"future":true}}}}}}"#
        ));
        let state = fold_items([task6_create(), update]);
        let task_id = TASK_ID.parse::<ThingsId>().expect("valid task id");
        let object = state.get(&task_id).expect("task state should remain");

        assert_eq!(
            object.entity_type,
            Some(EntityType::Unknown("Task8".to_string()))
        );
        assert!(matches!(object.properties, StateProperties::Task(_)));
        assert!(
            ThingsStore::from_raw_state(&state)
                .get_task(TASK_ID)
                .is_some()
        );
    }

    #[test]
    fn malformed_task7_patch_is_preserved_as_unknown_and_ignored() {
        let update = wire_item(&format!(
            r#"{{"{TASK_ID}":{{"t":1,"e":"Task7","p":{{"ss":"future"}}}}}}"#
        ));
        let object = update.get(TASK_ID).expect("Task7 update");
        assert!(matches!(object.payload, Properties::Unknown(_)));

        let state = fold_items([task6_create(), update]);
        let task_id = TASK_ID.parse::<ThingsId>().expect("valid task id");
        let StateProperties::Task(properties) = &state[&task_id].properties else {
            panic!("malformed Task7 patch replaced typed task state");
        };
        assert_eq!(properties.title, "Send tracking number");
        assert_eq!(properties.status, TaskStatus::Incomplete);
    }
}
