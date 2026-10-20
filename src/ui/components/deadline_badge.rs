use chrono::{DateTime, Utc};
use iocraft::prelude::*;

use crate::common::ICONS;

#[derive(Default, Props)]
pub struct DeadlineBadgeProps {
    pub deadline: Option<DateTime<Utc>>,
}

#[component]
pub fn DeadlineBadge<'a>(hooks: Hooks, props: &DeadlineBadgeProps) -> impl Into<AnyElement<'a>> {
    let Some(deadline) = props.deadline else {
        return element!(Fragment).into_any();
    };

    let today = *hooks.use_context::<DateTime<Utc>>();
    let color = if deadline < today {
        Color::Red
    } else {
        Color::Yellow
    };
    let content = format!("due by {}", deadline.format("%Y-%m-%d"));

    element! {
        View(flex_direction: FlexDirection::Row, gap: 1) {
            Text(content: ICONS.deadline, color: color)
            Text(content: content, color: color)
        }
    }
    .into_any()
}
