//! DOM helpers.

pub mod align;

pub mod focus;

mod number_format;
pub use number_format::{LocaleInfo, format_float, parse_float};

mod dom_size_observer;
pub use dom_size_observer::{DomSizeObserver, IntoSizeCallback, SizeCallback};

mod dom_visibility_observer;
pub use dom_visibility_observer::DomVisibilityObserver;

mod viewport_query;
pub use viewport_query::ViewportQuery;

use web_sys::Node;
use yew::prelude::*;

/// Write `text` to the system clipboard, returning whether the write succeeded.
///
/// Uses the async Clipboard API (`navigator.clipboard.writeText`). For a fire-and-forget copy
/// from a synchronous handler, spawn this and ignore the result.
pub async fn copy_to_clipboard(text: &str) -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let clipboard = window.navigator().clipboard();
    // the Clipboard API is [SecureContext], so the getter yields undefined over plain http, where
    // calling into it would throw out of the wasm frame instead of rejecting the promise
    if clipboard.is_undefined() || clipboard.is_null() {
        return false;
    }
    wasm_bindgen_futures::JsFuture::from(clipboard.write_text(text))
        .await
        .is_ok()
}

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
        Some(self.into())
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
    let el = node.into_html_element()?;

    if let Ok(Some(style)) = gloo_utils::window().get_computed_style(&el) {
        if let Ok(direction) = style.get_property_value("direction") {
            return Some(direction == "rtl");
        }
    }

    None
}

/// Returns if the system prefers dark mode
pub fn get_system_prefer_dark_mode() -> bool {
    if let Ok(Some(list)) = gloo_utils::window().match_media("(prefers-color-scheme: dark)") {
        list.matches()
    } else {
        false
    }
}
