//! DOM helpers.

mod number_format;
pub use number_format::{format_float, parse_float, LocaleInfo};

mod dom_size_observer;
pub use dom_size_observer::{DomSizeObserver, IntoSizeCallback, SizeCallback};

mod dom_visibility_observer;
pub use dom_visibility_observer::DomVisibilityObserver;

use web_sys::Node;
use yew::prelude::*;

/// A Trait to convert structs into HtmlElement when possible
pub trait IntoHtmlElement {
    fn into_html_element(self) -> Option<web_sys::HtmlElement>;
}

impl IntoHtmlElement for &NodeRef {
    fn into_html_element(self) -> Option<web_sys::HtmlElement> {
        self.cast::<web_sys::HtmlElement>()
    }
}

impl IntoHtmlElement for NodeRef {
    fn into_html_element(self) -> Option<web_sys::HtmlElement> {
        (&self).into_html_element()
    }
}

impl IntoHtmlElement for wasm_bindgen::JsValue {
    fn into_html_element(self) -> Option<web_sys::HtmlElement> {
        match self.try_into() {
            Ok(el) => Some(el),
            Err(_) => None,
        }
    }
}

impl IntoHtmlElement for Node {
    fn into_html_element(self) -> Option<web_sys::HtmlElement> {
        std::convert::Into::<wasm_bindgen::JsValue>::into(self).into_html_element()
    }
}

impl IntoHtmlElement for web_sys::HtmlElement {
    fn into_html_element(self) -> Option<web_sys::HtmlElement> {
        Some(self)
    }
}

/// Detect if CSS `direction` property is set to `rtl` (left-to-right mode).
///
/// Uses `getComputedStyle()` to get the inherited CSS value. Simply returns
/// [None] on error.
pub fn element_direction_rtl<T: IntoHtmlElement>(node: T) -> Option<bool> {
    let el = match node.into_html_element() {
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

/// Returns if the system prefers dark mode
pub fn get_system_prefer_dark_mode() -> bool {
    let window = web_sys::window().unwrap();
    if let Ok(Some(list)) = window.match_media("(prefers-color-scheme: dark)") {
        list.matches()
    } else {
        false
    }
}
