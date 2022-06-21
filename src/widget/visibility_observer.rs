use web_sys::{Element, IntersectionObserver, IntersectionObserverEntry};
use wasm_bindgen::JsCast;
use wasm_bindgen::UnwrapThrowExt;
use wasm_bindgen::{prelude::*};

use yew::prelude::*;

pub struct VisibilityObserver {
    observer: IntersectionObserver,
    // keep it alive
    _observer_closure: Closure::<dyn Fn(Vec<IntersectionObserverEntry>, IntersectionObserver)>,
}

impl VisibilityObserver {

    pub fn new(el: &Element, callback: impl Into<Callback<bool>>) -> Self {
        let callback = callback.into();

        let observer_closure = Closure::wrap(
            Box::new(
                move |entries: Vec<IntersectionObserverEntry>, _observer: IntersectionObserver| {
                    if entries.len() == 1 {
                        let entry = &entries[0];
                        callback.emit(entry.is_intersecting());
                    } else {
                        unreachable!();
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
