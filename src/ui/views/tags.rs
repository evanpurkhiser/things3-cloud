use std::collections::BTreeMap;

use iocraft::prelude::*;

use crate::{
    common::ICONS,
    ids::ThingsId,
    store::Tag,
    ui::components::{empty_text::EmptyText, tag_item::TagItem},
};

#[derive(Default, Props)]
pub struct TagsViewProps {
    pub tags_count: usize,
    pub top_level: Vec<Tag>,
    pub children: BTreeMap<ThingsId, Vec<Tag>>,
}

#[component]
pub fn TagsView<'a>(props: &'a TagsViewProps) -> impl Into<AnyElement<'a>> {
    if props.tags_count == 0 {
        return element! { EmptyText(content: "No tags.") }.into_any();
    }

    let trees = props.top_level.iter().map(|tag| {
        let descendants = props.children.get(&tag.uuid).map(|subtags| {
            let subtags = render_subtags(subtags, &props.children);

            element! {
                View(flex_direction: FlexDirection::Column) { #(subtags) }
            }
            .into_any()
        });

        element! {
            View(flex_direction: FlexDirection::Column) {
                TagItem(tag)
                #(descendants)
            }
        }
    });

    element! {
        View(flex_direction: FlexDirection::Column) {
            Text(
                content: format!("{} Tags  ({})", ICONS.tag, props.tags_count),
                weight: Weight::Bold,
                wrap: TextWrap::NoWrap,
            )
            Text(content: "", wrap: TextWrap::NoWrap)
            View(flex_direction: FlexDirection::Column, padding_left: 2) {
                #(trees)
            }
        }
    }
    .into_any()
}

fn render_subtags<'a>(
    subtags: &'a [Tag],
    children: &'a BTreeMap<ThingsId, Vec<Tag>>,
) -> Vec<AnyElement<'a>> {
    subtags
        .iter()
        .enumerate()
        .map(|(index, tag)| {
            let is_last = index == subtags.len() - 1;
            let connector = if is_last { "└╴" } else { "├╴" };
            let descendants = children.get(&tag.uuid).map(|grandchildren| {
                let grandchildren = render_subtags(grandchildren, children);
                let rail = tree_rail(!is_last);

                element! {
                    View(flex_direction: FlexDirection::Row) {
                        #(rail)
                        View(flex_direction: FlexDirection::Column) { #(grandchildren) }
                    }
                }
                .into_any()
            });

            element! {
                View(flex_direction: FlexDirection::Column) {
                    View(flex_direction: FlexDirection::Row) {
                        Text(content: connector, color: Color::DarkGrey, wrap: TextWrap::NoWrap)
                        TagItem(tag)
                    }
                    #(descendants)
                }
            }
            .into_any()
        })
        .collect()
}

fn tree_rail<'a>(continues: bool) -> AnyElement<'a> {
    if !continues {
        return element! { View(width: 2) {} }.into_any();
    }

    let border = BorderCharacters {
        top_left: '│',
        top_right: ' ',
        bottom_left: '│',
        bottom_right: ' ',
        left: '│',
        right: ' ',
        top: ' ',
        bottom: ' ',
    };

    element! {
        View(
            width: 2,
            border_style: BorderStyle::Custom(border),
            border_edges: Some(Edges::Left),
            border_color: Color::DarkGrey,
        ) {}
    }
    .into_any()
}
