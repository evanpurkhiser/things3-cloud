use iocraft::prelude::*;

use crate::{common::ICONS, store::ChecklistItem, ui::components::id::Id};

/// A single checklist-item row.
///
/// When `id` is `Some`, it is rendered in a fixed-width left column so
/// connectors follow the ID prefix.
///
/// ```text
/// M ├╴○ Confirm changelog
/// J └╴● Tag release commit   (is_last)
/// ```
///
/// When `id` is `None` (no IDs), the connector starts at column 0:
/// ```text
/// ├╴○ title
/// └╴● title
/// ```

#[derive(Default, Props)]
pub struct CheckListRowProps<'a> {
    pub item: Option<&'a ChecklistItem>,
    pub id_prefix_len: usize,
    pub is_last: bool,
}

#[component]
pub fn CheckListRow<'a>(props: &CheckListRowProps<'a>) -> impl Into<AnyElement<'a>> {
    let Some(item) = props.item else {
        return element!(Fragment).into_any();
    };

    let connector = if props.is_last { "└╴" } else { "├╴" };

    let id = if props.id_prefix_len > 0 {
        element!(Id(id: &item.uuid, length: props.id_prefix_len)).into_any()
    } else {
        element!(Fragment).into_any()
    };

    element!(View {
        View(flex_direction: FlexDirection::Row, gap: 1) {
            #(id)
            Text(content: connector, color: Color::DarkGrey)
        }
        View(flex_direction: FlexDirection::Row, gap: 1) {
            Text(content: checklist_icon(item), color: Color::DarkGrey)
            Text(content: item.title.clone())
        }
    })
    .into_any()
}

fn checklist_icon(item: &ChecklistItem) -> &'static str {
    if item.is_completed() {
        ICONS.checklist_done
    } else if item.is_canceled() {
        ICONS.checklist_canceled
    } else {
        ICONS.checklist_open
    }
}
