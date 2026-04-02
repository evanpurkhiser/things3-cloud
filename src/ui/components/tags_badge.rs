use std::sync::Arc;

use iocraft::prelude::*;

use crate::{ids::ThingsId, store::ThingsStore};

#[derive(Default, Props)]
pub struct TagsBadgeProps {
    pub tags: Vec<ThingsId>,
}

#[component]
pub fn TagsBadge<'a>(hooks: Hooks, props: &TagsBadgeProps) -> impl Into<AnyElement<'a>> {
    if props.tags.is_empty() {
        return element!(Fragment).into_any();
    }

    let store = hooks.use_context::<Arc<ThingsStore>>().clone();
    let names = props
        .tags
        .iter()
        .map(|tag| store.resolve_tag_title(tag))
        .collect::<Vec<_>>()
        .join(", ");

    element! {
        Text(content: format!("[{}]", names), color: Color::DarkGrey, wrap: TextWrap::NoWrap)
    }
    .into_any()
}
