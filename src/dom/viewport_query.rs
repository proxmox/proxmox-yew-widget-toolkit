use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::MediaQueryList;
use yew::Callback;

use gloo_utils::window;

/// A live subscription to a CSS media query.
///
/// Reports the query's current match state on creation and invokes `on_change` whenever it flips,
/// so a component can re-render when the viewport crosses a breakpoint. The browser listener is
/// removed when the `ViewportQuery` is dropped, so keep it as a component field for the lifetime of
/// the subscription.
///
/// ```
/// # use pwt::dom::ViewportQuery;
/// # use yew::Callback;
/// # fn test(on_change: Callback<bool>) {
/// let (is_wide, _guard) = ViewportQuery::subscribe("(min-width: 768px)", on_change);
/// # }
/// ```
pub struct ViewportQuery {
    mql: MediaQueryList,
    listener: Closure<dyn Fn()>,
}

impl ViewportQuery {
    /// Subscribe to `query`, returning its current match state together with the subscription
    /// guard. The match state is `false` and the guard `None` when the query cannot be evaluated,
    /// for example in a non-browser context or with an invalid query string.
    pub fn subscribe(query: &str, on_change: Callback<bool>) -> (bool, Option<Self>) {
        let mql = match window().match_media(query) {
            Ok(Some(mql)) => mql,
            _ => return (false, None),
        };

        let matches = mql.matches();

        let listener = Closure::wrap(Box::new({
            let mql = mql.clone();
            move || on_change.emit(mql.matches())
        }) as Box<dyn Fn()>);

        let _ = mql.add_event_listener_with_callback("change", listener.as_ref().unchecked_ref());

        (matches, Some(Self { mql, listener }))
    }
}

impl Drop for ViewportQuery {
    fn drop(&mut self) {
        let _ = self
            .mql
            .remove_event_listener_with_callback("change", self.listener.as_ref().unchecked_ref());
    }
}
