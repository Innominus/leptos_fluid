use leptos::prelude::request_animation_frame;
use web_sys::{Element, wasm_bindgen::JsCast};

pub(crate) fn get_scroll_pos_of_attr_children(
    parent: &Element,
    attr_name: &str,
) -> Vec<(i32, i32)> {
    let selector = get_selector(attr_name);
    if let Ok(node_list) = parent.query_selector_all(&selector) {
        let mut top_left_scroll_pos = Vec::new();
        for i in 0..node_list.length() {
            if let Some(node) = node_list.get(i) {
                let el = node.unchecked_into::<Element>();
                top_left_scroll_pos.push((el.scroll_top(), el.scroll_left()));
            }
        }
        return top_left_scroll_pos;
    }

    Vec::new()
}

pub(crate) fn set_scroll_pos_to_children_with_attr(
    parent: &Element,
    attr_name: &str,
    top_left_scroll_pos: Vec<(i32, i32)>,
) {
    if top_left_scroll_pos.is_empty() {
        return;
    }

    let selector = get_selector(attr_name);
    if let Ok(node_list) = parent.query_selector_all(&selector) {
        request_animation_frame(move || {
            for i in 0..node_list.length() {
                if let Some(node) = node_list.get(i) {
                    let (top_pos, left_pos) = top_left_scroll_pos[i as usize];
                    let el = node.unchecked_into::<Element>();
                    el.set_scroll_top(top_pos);
                    el.set_scroll_left(left_pos);
                }
            }
        });
    }
}

pub(crate) fn get_selector(attr_name: &str) -> String {
    "[".to_string() + attr_name + "]"
}
