use std::sync::Arc;

use iocraft::prelude::*;

use crate::{
    common::ICONS,
    store::{Task, ThingsStore},
    ui::components::{
        empty_text::EmptyText,
        project_item::ProjectItem,
        task_item::TaskItem,
        tasks::TaskOptions,
    },
};

#[derive(Clone)]
pub struct FindRow<'a> {
    pub task: &'a Task,
    pub force_detailed: bool,
}

#[derive(Default, Props)]
pub struct FindViewProps<'a> {
    pub rows: Vec<FindRow<'a>>,
    pub detailed: bool,
}

#[component]
pub fn FindView<'a>(hooks: Hooks, props: &FindViewProps<'a>) -> impl Into<AnyElement<'a>> {
    let store = hooks.use_context::<Arc<ThingsStore>>().clone();

    if props.rows.is_empty() {
        return element! { EmptyText(content: "No matching tasks.") }.into_any();
    }

    let id_prefix_len = store.unique_prefix_length(
        &props
            .rows
            .iter()
            .map(|row| row.task.uuid.clone())
            .collect::<Vec<_>>(),
    );

    let count = props.rows.len();
    let label = if count == 1 { "task" } else { "tasks" };

    let mut body: Vec<AnyElement<'a>> = Vec::new();
    for row in &props.rows {
        let options = TaskOptions {
            detailed: props.detailed || row.force_detailed,
            show_project: true,
            show_area: false,
            show_today_markers: true,
            show_staged_today_marker: false,
        };
        let line = if row.task.is_project() {
            element! {
                View(flex_direction: FlexDirection::Column, padding_left: 2) {
                    ProjectItem(project: row.task, options, id_prefix_len)
                }
            }
            .into_any()
        } else {
            element! {
                View(flex_direction: FlexDirection::Column, padding_left: 2) {
                    TaskItem(task: row.task, options, id_prefix_len)
                }
            }
            .into_any()
        };
        body.push(line);
    }

    element! {
        View(flex_direction: FlexDirection::Column) {
            Text(
                content: format!("{} Find  ({} {})", ICONS.find, count, label),
                wrap: TextWrap::NoWrap,
                color: Color::Cyan,
                weight: Weight::Bold,
            )
            Text(content: "", wrap: TextWrap::NoWrap)
            #(body)
        }
    }
    .into_any()
}
