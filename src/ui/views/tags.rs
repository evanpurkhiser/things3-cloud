use crate::common::ICONS;
use crate::ids::ThingsId;
use crate::store::Tag;
use iocraft::prelude::*;
use std::collections::BTreeMap;

#[derive(Default, Props)]
pub struct TagsViewProps {
    pub tags_count: usize,
    pub top_level: Vec<Tag>,
    pub children: BTreeMap<ThingsId, Vec<Tag>>,
}

#[component]
pub fn TagsView<'a>(props: &'a TagsViewProps) -> impl Into<AnyElement<'a>> {
    if props.tags_count == 0 {
        return element! {
            Text(content: "No tags.", color: Color::DarkGrey, wrap: TextWrap::NoWrap)
        }
        .into_any();
    }

    let mut lines: Vec<AnyElement<'a>> = Vec::new();
    for tag in &props.top_level {
        lines.push(
            element! {
                View(flex_direction: FlexDirection::Row, gap: 0, padding_left: 2) {
                    Text(content: ICONS.tag, color: Color::DarkGrey, wrap: TextWrap::NoWrap)
                    Text(content: " ", wrap: TextWrap::NoWrap)
                    Text(content: tag.title.clone(), wrap: TextWrap::NoWrap)
                    #(shortcut_element(tag))
                }
            }
            .into_any(),
        );
        if let Some(subtags) = props.children.get(&tag.uuid) {
            lines.extend(render_subtags(subtags, "", &props.children));
        }
    }

    element! {
        View(flex_direction: FlexDirection::Column) {
            Text(
                content: format!("{} Tags  ({})", ICONS.tag, props.tags_count),
                weight: Weight::Bold,
                wrap: TextWrap::NoWrap,
            )
            Text(content: "", wrap: TextWrap::NoWrap)
            #(lines)
        }
    }
    .into_any()
}

fn shortcut_element<'a>(tag: &Tag) -> Option<AnyElement<'a>> {
    tag.shortcut.as_ref().map(|shortcut| {
        element! {
            Text(content: format!("  [{shortcut}]"), color: Color::DarkGrey, wrap: TextWrap::NoWrap)
        }
        .into_any()
    })
}

fn render_subtags<'a>(
    subtags: &[Tag],
    indent: &str,
    children: &BTreeMap<ThingsId, Vec<Tag>>,
) -> Vec<AnyElement<'a>> {
    let mut lines = Vec::new();

    for (i, tag) in subtags.iter().enumerate() {
        let is_last = i == subtags.len() - 1;
        let connector = if is_last { "└╴" } else { "├╴" };

        lines.push(
            element! {
                View(flex_direction: FlexDirection::Row, gap: 0, padding_left: 2) {
                    Text(content: indent.to_string(), color: Color::DarkGrey, wrap: TextWrap::NoWrap)
                    Text(content: connector, color: Color::DarkGrey, wrap: TextWrap::NoWrap)
                    Text(content: ICONS.tag, color: Color::DarkGrey, wrap: TextWrap::NoWrap)
                    Text(content: " ", wrap: TextWrap::NoWrap)
                    Text(content: tag.title.clone(), wrap: TextWrap::NoWrap)
                    #(shortcut_element(tag))
                }
            }
            .into_any(),
        );

        if let Some(grandchildren) = children.get(&tag.uuid) {
            let child_indent = if is_last {
                format!("{}  ", indent)
            } else {
                format!("{}│ ", indent)
            };
            lines.extend(render_subtags(grandchildren, &child_indent, children));
        }
    }

    lines
}
