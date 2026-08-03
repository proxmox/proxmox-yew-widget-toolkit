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

use wasm_bindgen::JsCast;
use web_sys::{Event, KeyboardEvent, Node};
use yew::prelude::*;

/// Whether `event` can be read as a keyboard event without panicking.
///
/// Yew hands a listener its event through `unchecked_into`, so anything dispatched under a
/// keyboard event type arrives typed as one. Reading a non-nullable member of it - `key`, `code` -
/// then finds the property missing, and decoding `undefined` as a string panics, taking the whole
/// wasm instance down with it. The page is dead until it is reloaded. Chrome's form autofill
/// dispatches exactly such an event, and it was a login mask that found this.
///
/// Either it really is a keyboard event, or it at least carries a string `key` - which the
/// synthetic events password managers and autofill dispatch usually do. An `instanceof` alone
/// would reject those, and would also reject a genuine event constructed in another realm (an
/// iframe, an extension content script), so the property is worth asking about rather than
/// trusting the class alone.
pub fn is_keyboard_event(event: &Event) -> bool {
    event.is_instance_of::<KeyboardEvent>()
        || js_sys::Reflect::get(event, &wasm_bindgen::JsValue::from_str("key"))
            .ok()
            .and_then(|value| value.as_string())
            .is_some()
}

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
