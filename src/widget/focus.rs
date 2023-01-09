//! DOM focus management helpers.

use std::ops::Deref;

use wasm_bindgen::JsCast;

use yew::prelude::*;

const FOCUSABLE_SELECTOR: &str = "a:not([disabled]), button:not([disabled]), input[type=text]:not([disabled]), [tabindex]:not([disabled])";
const FOCUSABLE_SELECTOR_ALL: &str = "a, button, input, [tabindex]";

/// Test is an element can take focus.
///
/// Returns true if the element has an `tabindex` attribute, or if the
/// element is of type `<a>`, `<button>`, or `<input>`.
pub fn element_is_focusable(el: &web_sys::HtmlElement) -> bool {
    if el.has_attribute("tabindex") {
        return true;
    }

    match el.tag_name().as_str() {
        "A" | "BUTTON" | "INPUT" => true,
        _ => false,
    }
}

/// Returns the first focusable child element.
///
/// This function skips disabled elements.
pub fn get_first_focusable(item_el: web_sys::Element) -> Option<web_sys::HtmlElement> {
    const FOCUSABLE_SELECTOR: &str = concat!(
        "a:not([disabled]),",
        "button:not([disabled]),",
        "input:not([disabled]),",
        "[tabindex]:not([disabled])",
    );

    let focus_el = match item_el.query_selector(FOCUSABLE_SELECTOR) {
        Ok(Some(focus_el)) => focus_el,
        _ => return None,
    };

    match focus_el.dyn_into::<web_sys::HtmlElement>() {
        Ok(el) => Some(el),
        _ => None,
    }
}

pub fn focus_next_tabable(node_ref: &NodeRef, backwards: bool, roving: bool) {
    if let Some(el) = node_ref.cast::<web_sys::HtmlElement>() {
        focus_next_tabable_el(el, backwards, roving);
    }
}

pub fn focus_next_tabable_el(el: web_sys::HtmlElement, backwards: bool, roving: bool) {
    if let Ok(list) = el.query_selector_all(FOCUSABLE_SELECTOR) {
        let list = js_sys::Array::from(&list);
        if list.length() == 0 { return; }

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();

        let index = match document.active_element() {
            Some(active_element) => list.index_of(&active_element, 0),
            None => -1,
        };

        if roving && list.length() > 1 && index >= 0 {
            let node = list.get(index as u32);
            if let Ok(el) = node.dyn_into::<web_sys::HtmlElement>() {
                el.set_tab_index(-1);
            }
        }

        //log::info!("focus_next: got {} focusable elements, index {}", list.length(), index);

        let next = if index < 0 {
            if backwards { list.length() as i32 - 1 } else { 0 }
        } else {
            if backwards {
	        if index == 0 {
		    list.length() as i32 - 1
	        } else {
		    index - 1
	        }
	    } else {
	        if (index + 1) >= list.length() as i32 {
		    0
	        } else {
                    index as i32 + 1
                }
	    }
        };

        if let Ok(next_element) = list.get(next as u32).dyn_into::<web_sys::HtmlElement>() {
            let _ = next_element.focus();
            if roving && list.length() > 1 {
                next_element.set_tab_index(0);
            }
        }
    }
}

/// Test if a child has focus.
///
/// # Note
///
/// This returns false if the element itself has focus.
pub fn focus_inside_el(el: web_sys::HtmlElement) -> bool {
    if let Ok(list) = el.query_selector_all(FOCUSABLE_SELECTOR) {
        let list = js_sys::Array::from(&list);
        if list.length() == 0 { return false; }

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();

        let index = match document.active_element() {
            Some(active_element) => list.index_of(&active_element, 0),
            None => -1,
        };
        index >= 0
    } else {
        false
    }
}

pub fn update_roving_tabindex(node_ref: &NodeRef) {
    if let Some(el) = node_ref.cast::<web_sys::HtmlElement>() {
        update_roving_tabindex_el(el);
    }
}

/// Update roving tabindex after focus change
pub fn update_roving_tabindex_el(el: web_sys::HtmlElement) {

    if let Ok(child_list) = el.query_selector_all(":scope > *") {
        let child_list = js_sys::Array::from(&child_list);
        if child_list.length() == 0 { return; }

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();

        let active_el = document.active_element();
        let active_node: Option<&web_sys::Node> = active_el.as_ref().map(|el| el.deref());

        let mut index = 0;
        for i in 0..child_list.length() {
            let node = child_list.get(i).dyn_into::<web_sys::HtmlElement>().unwrap();
            if node.contains(active_node) {
                index = i;
            }
        }

        //log::info!("update_roving_tabindex: got {} focusable elements, index {}", child_list.length(), index);

        for i in 0..child_list.length() {
            let node = child_list.get(i).dyn_into::<web_sys::HtmlElement>().unwrap();

            if element_is_focusable(&node) {
                if i == index {
                    node.set_tab_index(0);
                } else {
                    node.set_tab_index(-1);
                }
            } else if let Ok(Some(child)) = node.query_selector(FOCUSABLE_SELECTOR_ALL) {
                let child = child.dyn_into::<web_sys::HtmlElement>().unwrap();
                if i == index {
                    child.set_tab_index(0);
                } else {
                    child.set_tab_index(-1);
                }
            }
        }
    }
}

/// Return all child elements participating in the roving tabindex algorithm.
///
/// The list contains all children which are [focusable](element_is_focusable),
/// or the first focusable descendant for children not directly focusable.
///
/// The list includes disabled element.
pub fn roving_tabindex_members(el: web_sys::HtmlElement) -> Vec<web_sys::HtmlElement> {
    let mut members: Vec<web_sys::HtmlElement> = Vec::new();

    if let Ok(child_list) = el.query_selector_all(":scope > *") {
        let child_list = js_sys::Array::from(&child_list);
        for i in 0..child_list.length() {
            let node = child_list.get(i).dyn_into::<web_sys::HtmlElement>().unwrap();
            if element_is_focusable(&node) {
                members.push(node);
            } else if let Ok(Some(child)) = node.query_selector(FOCUSABLE_SELECTOR_ALL) {
                let first_focusable_child = child.dyn_into::<web_sys::HtmlElement>().unwrap();
                members.push(first_focusable_child);
            }
        }
    }

    members
}

pub fn init_roving_tabindex(node_ref: &NodeRef) {
    if let Some(el) = node_ref.cast::<web_sys::HtmlElement>() {
        init_roving_tabindex_el(el, false);
    }
}

pub fn init_roving_tabindex_el(el: web_sys::HtmlElement, take_focus: bool) {
    let list = roving_tabindex_members(el);

    if list.len() == 0 { return; }

    //log::info!("init_roving_tabindex: got {} focusable elements", list.length());

    let mut default_index = 0;
    for i in 0..list.len() {
        let item = &list[i];
        if item.tab_index() == 0 {
            default_index = i;
            break;
        }
    }

    for i in 0..list.len() {
        let item = &list[i];
        if i == default_index {
            item.set_tab_index(0);
            if take_focus {
                let _ = item.focus();
            }
        } else {
            item.set_tab_index(-1);
        }
    }
}
