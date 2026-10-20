use iocraft::prelude::*;

use crate::{common::ICONS, store::Tag, ui::components::id::Id};

#[derive(Default, Props)]
pub struct TagItemProps<'a> {
    pub tag: Option<&'a Tag>,
    pub id_prefix_len: usize,
}

#[component]
pub fn TagItem<'a>(props: &TagItemProps<'a>) -> impl Into<AnyElement<'a>> {
    let Some(tag) = props.tag else {
        return element!(Fragment).into_any();
    };

    let id = (props.id_prefix_len > 0).then(|| {
        element! {
            Id(id: &tag.uuid, length: props.id_prefix_len)
        }
        .into_any()
    });
    let shortcut = tag.shortcut.as_ref().map(|shortcut| {
        element! {
            View(margin_left: 1) {
                Text(content: format!("[{shortcut}]"), color: Color::DarkGrey, wrap: TextWrap::NoWrap)
            }
        }
        .into_any()
    });

    element! {
        View(flex_direction: FlexDirection::Row, gap: 1) {
            #(id)
            Text(content: ICONS.tag, color: Color::DarkGrey, wrap: TextWrap::NoWrap)
            Text(content: tag.title.clone(), wrap: TextWrap::NoWrap)
            #(shortcut)
        }
    }
    .into_any()
}
