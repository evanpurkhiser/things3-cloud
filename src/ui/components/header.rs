use iocraft::prelude::*;

use crate::{ids::ThingsId, ui::components::id::Id};

#[derive(Default, Props)]
pub struct HeaderProps<'a> {
    pub uuid: Option<&'a ThingsId>,
    pub title: Option<&'a str>,
    pub id_prefix_len: usize,
    pub icon: Option<&'a str>,
}

#[component]
pub fn Header<'a>(props: &HeaderProps<'a>) -> impl Into<AnyElement<'a>> {
    let (Some(uuid), Some(title)) = (props.uuid, props.title) else {
        return element!(Fragment).into_any();
    };

    let icon = props.icon.map(|icon| {
        element! {
            Text(content: icon, wrap: TextWrap::NoWrap, weight: Weight::Bold)
        }
        .into_any()
    });

    element! {
        View(flex_direction: FlexDirection::Row, gap: 1) {
            Id(id: uuid, length: props.id_prefix_len)
            View(flex_direction: FlexDirection::Row, gap: 1) {
                #(icon)
                Text(content: title, wrap: TextWrap::NoWrap, weight: Weight::Bold)
            }
        }
    }
    .into_any()
}
