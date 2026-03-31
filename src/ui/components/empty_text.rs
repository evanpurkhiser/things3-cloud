use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct EmptyTextProps<'a> {
    pub content: &'a str,
}

#[component]
pub fn EmptyText<'a>(props: &EmptyTextProps<'a>) -> impl Into<AnyElement<'a>> {
    element! {
        Text(content: props.content, color: Color::DarkGrey, wrap: TextWrap::NoWrap)
    }
    .into_any()
}
