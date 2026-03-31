use crate::store::{Task, ThingsStore};
use crate::ui::components::deadline_badge::DeadlineBadge;
use crate::ui::components::details_container::DetailsContainer;
use crate::ui::components::progress_badge::ProgressBadge;
use crate::ui::components::tags_badge::TagsBadge;
use crate::ui::components::tasks::{TaskList, TaskOptions};
use iocraft::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct ProjectHeadingGroup<'a> {
    pub title: String,
    pub items: Vec<&'a Task>,
}

#[derive(Default, Props)]
pub struct ProjectViewProps<'a> {
    pub project: Option<&'a Task>,
    pub ungrouped: Vec<&'a Task>,
    pub heading_groups: Vec<ProjectHeadingGroup<'a>>,
    pub detailed: bool,
    pub no_color: bool,
}

#[component]
pub fn ProjectView<'a>(hooks: Hooks, props: &ProjectViewProps<'a>) -> impl Into<AnyElement<'a>> {
    let store = hooks.use_context::<Arc<ThingsStore>>().clone();
    let Some(project) = props.project else {
        return element! { Text(content: "") }.into_any();
    };
    let _ = props.no_color;

    let mut all_uuids = props
        .ungrouped
        .iter()
        .map(|t| t.uuid.clone())
        .collect::<Vec<_>>();
    for group in &props.heading_groups {
        all_uuids.extend(group.items.iter().map(|t| t.uuid.clone()));
    }
    let id_prefix_len = store.unique_prefix_length(&all_uuids);

    let options = TaskOptions {
        detailed: props.detailed,
        show_project: false,
        show_area: false,
        show_today_markers: true,
        show_staged_today_marker: false,
    };

    let note_lines = project
        .notes
        .as_deref()
        .unwrap_or("")
        .lines()
        .map(|line| {
            element! {
                Text(content: line, wrap: TextWrap::NoWrap, color: Color::DarkGrey)
            }
            .into_any()
        })
        .collect::<Vec<_>>();

    element! {
        View(flex_direction: FlexDirection::Column) {
            View(flex_direction: FlexDirection::Row, gap: 1) {
                ProgressBadge(
                    project: project,
                    title: Some(project.title.clone()),
                    show_count: true,
                    color: Color::Magenta,
                    weight: Weight::Bold,
                )
                DeadlineBadge(deadline: project.deadline)
                TagsBadge(tags: project.tags.clone())
            }

            #(if !note_lines.is_empty() {
                Some(element! {
                    View(flex_direction: FlexDirection::Column, padding_left: 2) {
                        DetailsContainer {
                            #(note_lines)
                        }
                    }
                })
            } else { None })

            #(if props.ungrouped.is_empty() && props.heading_groups.is_empty() {
                Some(element! {
                    Text(content: "  No tasks.", wrap: TextWrap::NoWrap, color: Color::DarkGrey)
                })
            } else { None })

            #(if !props.ungrouped.is_empty() {
                Some(element! {
                    View(flex_direction: FlexDirection::Column) {
                        Text(content: "", wrap: TextWrap::NoWrap)
                        View(flex_direction: FlexDirection::Column, padding_left: 2) {
                            TaskList(items: props.ungrouped.clone(), id_prefix_len, options)
                        }
                    }
                })
            } else { None })

            #(props.heading_groups.iter().map(|group| element! {
                View(flex_direction: FlexDirection::Column) {
                    Text(content: "", wrap: TextWrap::NoWrap)
                    Text(content: format!("  {}", group.title), wrap: TextWrap::NoWrap, weight: Weight::Bold)
                    View(flex_direction: FlexDirection::Column, padding_left: 4) {
                        TaskList(items: group.items.clone(), id_prefix_len, options)
                    }
                }
            }))
        }
    }
    .into_any()
}
