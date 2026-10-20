use iocraft::prelude::*;

use crate::common::ICONS;

#[derive(Default, Props)]
pub struct DeletedItemProps {
    pub id: String,
    pub title: String,
}

#[component]
pub fn DeletedItem(props: &DeletedItemProps) -> impl Into<AnyElement<'_>> {
    let id = if props.id.is_empty() {
        None
    } else {
        Some(
            element! {
                Text(content: props.id.clone(), color: Color::DarkGrey, wrap: TextWrap::NoWrap)
            }
            .into_any(),
        )
    };

    element! {
        View(flex_direction: FlexDirection::Row, gap: 1) {
            #(id)
            Text(content: ICONS.deleted, color: Color::DarkGrey, wrap: TextWrap::NoWrap)
            Text(content: props.title.clone(), wrap: TextWrap::NoWrap)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::render_element_to_string;

    #[test]
    fn renders_resolved_id_and_title() {
        let mut element = element! {
            DeletedItem(id: "A", title: "Archived task")
        };

        assert_eq!(
            render_element_to_string(&mut element, true),
            "A × Archived task"
        );
    }
}
