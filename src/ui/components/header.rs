use crate::ids::ThingsId;
use crate::ui::components::id::Id;
use iocraft::prelude::*;

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

    let text = if let Some(icon) = props.icon {
        format!("{} {}", icon, title)
    } else {
        title.to_string()
    };

    element! {
        View(flex_direction: FlexDirection::Row, gap: 1) {
            Id(id: uuid, length: props.id_prefix_len)
            Text(content: text, wrap: TextWrap::NoWrap)
        }
    }
    .into_any()
}
