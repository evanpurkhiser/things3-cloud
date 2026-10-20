use iocraft::prelude::*;

use crate::common::ICONS;

#[derive(Default, Props)]
pub struct MutationResultProps<'a> {
    pub action: String,
    pub detail: Option<String>,
    pub children: Vec<AnyElement<'a>>,
}

#[component]
pub fn MutationResult<'a>(props: &mut MutationResultProps<'a>) -> impl Into<AnyElement<'a>> {
    let detail = props.detail.as_ref().map(|detail| {
        element! {
            Text(content: format!("({detail})"), color: Color::DarkGrey, wrap: TextWrap::NoWrap)
        }
        .into_any()
    });
    let children = std::mem::take(&mut props.children);

    element! {
        View(flex_direction: FlexDirection::Column) {
            View(flex_direction: FlexDirection::Row, gap: 2) {
                View(flex_direction: FlexDirection::Row, gap: 1) {
                    Text(content: ICONS.done, color: Color::Green, wrap: TextWrap::NoWrap)
                    Text(content: props.action.clone(), wrap: TextWrap::NoWrap)
                }
                #(detail)
            }
            View(flex_direction: FlexDirection::Column) {
                #(children)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::render_element_to_string;

    #[test]
    fn renders_action_detail_and_body() {
        let mut element = element! {
            MutationResult(action: "Edited", detail: Some("title, tags".to_string())) {
                Text(content: "A ▢ Updated task")
            }
        };

        assert_eq!(
            render_element_to_string(&mut element, true),
            "✓ Edited  (title, tags)\nA ▢ Updated task"
        );
    }
}
