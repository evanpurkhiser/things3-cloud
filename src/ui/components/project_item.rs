use iocraft::prelude::*;

use crate::{
    store::Task,
    ui::components::{
        details_container::DetailsContainer,
        id::Id,
        progress_badge::ProgressBadge,
        task_line::TaskLine,
        tasks::TaskOptions,
    },
};

#[derive(Default, Props)]
pub struct ProjectItemProps<'a> {
    pub project: Option<&'a Task>,
    pub options: TaskOptions,
    pub id_prefix_len: usize,
}

#[component]
pub fn ProjectItem<'a>(props: &ProjectItemProps<'a>) -> impl Into<AnyElement<'a>> {
    let Some(project) = props.project else {
        return element!(Fragment).into_any();
    };

    let details = if props.options.detailed {
        element!(ProjectDetails(project: project)).into_any()
    } else {
        element!(Fragment).into_any()
    };

    element! {
        View(flex_direction: FlexDirection::Row, gap: 1) {
            Id(id: &project.uuid, length: props.id_prefix_len)
            View(flex_direction: FlexDirection::Column) {
                ProjectText(project: project, options: props.options)
                #(details)
            }
        }
    }
    .into_any()
}

#[derive(Default, Props)]
struct ProjectTextProps<'a> {
    pub project: Option<&'a Task>,
    pub options: TaskOptions,
}

#[component]
fn ProjectText<'a>(props: &ProjectTextProps<'a>) -> impl Into<AnyElement<'a>> {
    let Some(project) = props.project else {
        return element!(Fragment).into_any();
    };

    element! {
        View(flex_direction: FlexDirection::Row, gap: 1) {
            ProgressBadge(project)
            TaskLine(
                task: project,
                show_today_markers: props.options.show_today_markers,
                show_staged_today_marker: props.options.show_staged_today_marker,
                show_tags: true,
                show_project: false,
                show_area: props.options.show_area,
            )
        }
    }
    .into_any()
}

#[derive(Default, Props)]
struct ProjectDetailsProps<'a> {
    pub project: Option<&'a Task>,
}

#[component]
fn ProjectDetails<'a>(props: &ProjectDetailsProps<'a>) -> impl Into<AnyElement<'a>> {
    let Some(project) = props.project else {
        return element!(Fragment).into_any();
    };

    let note_text = project.notes.as_deref().unwrap_or("").trim();
    if note_text.is_empty() {
        return element!(Fragment).into_any();
    }

    element! {
        DetailsContainer {
            Text(content: note_text, wrap: TextWrap::NoWrap, color: Color::DarkGrey)
        }
    }
    .into_any()
}
