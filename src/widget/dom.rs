//! DOM helpers.

use yew::prelude::*;

/// Detect if CSS `direction` property is set to `rtl` (left-to-right mode).
///
/// Uses `getComputedStyle()` to get the inherited CSS value. Simply returns
/// [None] on error.
pub fn element_direction_rtl(node_ref: &NodeRef) -> Option<bool> {

    let el = match node_ref.cast::<web_sys::HtmlElement>() {
        Some(el) => el,
        None => return None,
    };

    let window = web_sys::window().unwrap();
    if let Ok(Some(style)) = window.get_computed_style(&el) {
        if let Ok(direction) = style.get_property_value("direction") {
            return Some(direction == "rtl");
        }
    }

    None
}
