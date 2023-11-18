use web_sys::{Element, IntersectionObserver, IntersectionObserverEntry};
use wasm_bindgen::JsCast;
use wasm_bindgen::UnwrapThrowExt;
use wasm_bindgen::{prelude::*};

use yew::prelude::*;

/// Track component visibility.
///
/// This struct uses the [IntersectionObserver] API to track if an element is visible
/// in the viewport or not.
///
/// See <https://developer.mozilla.org/en-US/docs/Web/API/Intersection_Observer_API>.
pub struct VisibilityObserver {
    observer: IntersectionObserver,
    // keep it alive
    _observer_closure: Closure::<dyn Fn(Vec<IntersectionObserverEntry>, IntersectionObserver)>,
}

impl VisibilityObserver {

    /// Creates a new instance.
    ///
    /// The callback is called whenever the element visibility changes.
    pub fn new(el: &Element, callback: impl Into<Callback<bool>>) -> Self {
        let callback = callback.into();

        let observer_closure = Closure::wrap(
            Box::new(
                move |entries: Vec<IntersectionObserverEntry>, _observer: IntersectionObserver| {
                    // Note: Chrome seems to queue events for a single target (sometimes), so we
                    // check the last entry.
                    if let Some(last) = entries.last() {
                        callback.emit(last.is_intersecting());
                    }
                }
            ) as Box<dyn Fn(Vec<IntersectionObserverEntry>, IntersectionObserver)>
        );

        let observer = IntersectionObserver::new(observer_closure.as_ref().unchecked_ref()).unwrap_throw();
        observer.observe(&el);

        Self {
            _observer_closure: observer_closure, // keep it alive
            observer,
        }
    }
}

impl Drop for VisibilityObserver {
    fn drop(&mut self) {
        self.observer.disconnect();
    }
}
