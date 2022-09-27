use web_sys::Element;
use wasm_bindgen::JsCast;
use wasm_bindgen::UnwrapThrowExt;
use wasm_bindgen::{prelude::*};

use crate::web_sys_ext::{ResizeObserver, ResizeObserverEntry};

use yew::prelude::*;

pub struct SizeObserver {
    observer: ResizeObserver,
    // keep it alive
    _observer_closure: Closure::<dyn Fn(Vec<ResizeObserverEntry>)>,
}

impl SizeObserver {

    pub fn new(el: &Element, callback: impl Into<Callback<(f64, f64)>>) -> Self {
        let callback = callback.into();
        let observer_closure = Closure::wrap(
            Box::new(
                move |entries: Vec<ResizeObserverEntry>| {
                    if entries.len() == 1 {
                        let el = entries[0].target();
                        let rect = el.get_bounding_client_rect();
                        callback.emit((rect.width(), rect.height()));
                    } else {
                        unreachable!();
                    }
                }
            ) as Box<dyn Fn(Vec<ResizeObserverEntry>)>
        );

        let observer = ResizeObserver::new(observer_closure.as_ref().unchecked_ref()).unwrap_throw();
        observer.observe(&el);

        Self {
            _observer_closure: observer_closure, // keep it alive
            observer,
        }
    }
}

impl Drop for SizeObserver {
    fn drop(&mut self) {
        self.observer.disconnect();
    }
}
