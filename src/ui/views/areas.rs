use iocraft::prelude::*;

use crate::{
    common::ICONS,
    store::Area,
    ui::components::{area_item::AreaItem, empty_text::EmptyText},
};

#[derive(Default, Props)]
pub struct AreasViewProps {
    pub areas: Vec<Area>,
    pub id_prefix_len: usize,
}

#[component]
pub fn AreasView<'a>(props: &'a AreasViewProps) -> impl Into<AnyElement<'a>> {
    if props.areas.is_empty() {
        return element!(EmptyText(content: "No areas.")).into_any();
    }

    element! {
        View(flex_direction: FlexDirection::Column, gap: 1) {
            Text(
                content: format!("{} Areas  ({})", ICONS.area, props.areas.len()),
                color: Color::Magenta,
                weight: Weight::Bold,
                wrap: TextWrap::NoWrap,
            )
            View(flex_direction: FlexDirection::Column, padding_left: 2) {
                #(props.areas.iter().map(|area| element! {
                    AreaItem(area, id_prefix_len: props.id_prefix_len)
                }))
            }
        }
    }
    .into_any()
}
