use wasm_bindgen::JsCast;

use yew::prelude::*;

const FOCUSABLE_SELECTOR: &str = "a:not([disabled]), button:not([disabled]), input[type=text]:not([disabled]), [tabindex]:not([disabled])";
const FOCUSABLE_SELECTOR_ALL: &str = "a, button, input, [tabindex]";

pub fn focus_next_tabable(node_ref: &NodeRef, backwards: bool, roving: bool) {
    if let Some(el) = node_ref.cast::<web_sys::Element>() {
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
}

pub fn init_roving_tabindex(node_ref: &NodeRef) {
    if let Some(el) = node_ref.cast::<web_sys::Element>() {
        if let Ok(list) = el.query_selector_all(FOCUSABLE_SELECTOR_ALL) {
            if list.length() == 0 { return; }

            //log::info!("init_roving_tabindex: got {} focusable elements", list.length());

            let mut default_index = 0;
            for i in 0..list.length() {
                let node = list.item(i).unwrap();
                if let Ok(el) = node.dyn_into::<web_sys::HtmlElement>() {
                    if el.tab_index() == 0 {
                        default_index = i;
                        break;
                    }
                }
            }

            for i in 0..list.length() {
                let node = list.item(i).unwrap();
                if let Ok(el) = node.dyn_into::<web_sys::HtmlElement>() {
                    el.set_tab_index(if i == default_index { 0 } else { -1 });
                }
            }
        }
    }

}
