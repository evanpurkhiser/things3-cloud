use iocraft::prelude::*;

use crate::{
    common::ICONS,
    store::Area,
    ui::components::{id::Id, tags_badge::TagsBadge},
};

#[derive(Default, Props)]
pub struct AreaItemProps<'a> {
    pub area: Option<&'a Area>,
    pub id_prefix_len: usize,
}

#[component]
pub fn AreaItem<'a>(props: &AreaItemProps<'a>) -> impl Into<AnyElement<'a>> {
    let Some(area) = props.area else {
        return element!(Fragment).into_any();
    };

    element! {
        View(flex_direction: FlexDirection::Row, gap: 2) {
            View(flex_direction: FlexDirection::Row, gap: 1) {
                Id(id: &area.uuid, length: props.id_prefix_len)
                Text(content: ICONS.area, color: Color::DarkGrey)
                Text(content: area.title.clone(), wrap: TextWrap::NoWrap)
            }
            TagsBadge(tags: area.tags.clone())
        }
    }
    .into_any()
}
