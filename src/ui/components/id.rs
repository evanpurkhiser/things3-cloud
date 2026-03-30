use crate::ids::ThingsId;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct IdProps<'a> {
    pub id: Option<&'a ThingsId>,
    pub length: usize,
}

#[component]
pub fn Id<'a>(props: &IdProps<'a>) -> impl Into<AnyElement<'a>> {
    let content = props.id.map_or_else(String::new, |id| {
        id.to_string().chars().take(props.length).collect()
    });

    element! {
        Text(content, color: Some(Color::DarkGrey), wrap: TextWrap::NoWrap)
    }
    .into_any()
}
