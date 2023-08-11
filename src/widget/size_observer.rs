use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::Element;

use crate::web_sys_ext::{ResizeObserver, ResizeObserverEntry, ResizeObserverOptions};

use yew::prelude::*;

type ObserverClosure = Closure<dyn Fn(Vec<ResizeObserverEntry>)>;

/// Wrapper around a ResizeObserver browser object, intended to create callbacks for size changes
/// of an element
pub struct SizeObserver {
    observer: ResizeObserver,
    // keep it alive
    _observer_closure: ObserverClosure,
}

/// Custom trait to automatically handle different types of callbacks
pub trait IntoSizeCallback<T> {
    fn into_size_cb(self) -> SizeCallback;
}

/// Callback with width and height parameters
impl<T> IntoSizeCallback<(f64, f64)> for T
where
    T: Into<Callback<(f64, f64)>>,
{
    fn into_size_cb(self) -> SizeCallback {
        SizeCallback::Normal(self.into())
    }
}

/// Callback with width, height, client_width and client_height parameters
impl<T> IntoSizeCallback<(f64, f64, f64, f64)> for T
where
    T: Into<Callback<(f64, f64, f64, f64)>>,
{
    fn into_size_cb(self) -> SizeCallback {
        SizeCallback::ClientRect(self.into())
    }
}

pub enum SizeCallback {
    Normal(Callback<(f64, f64)>),
    ClientRect(Callback<(f64, f64, f64, f64)>),
}

impl SizeObserver {
    fn create_observer(callback: SizeCallback) -> (ResizeObserver, ObserverClosure) {
        let observer_closure = Closure::wrap(Box::new(move |entries: Vec<ResizeObserverEntry>| {
            if entries.len() == 1 {
                let el = entries[0].target();
                let rect = el.get_bounding_client_rect();
                match &callback {
                    SizeCallback::Normal(cb) => cb.emit((rect.width(), rect.height())),
                    SizeCallback::ClientRect(cb) => {
                        let width: f64 = el.client_width().into();
                        let height: f64 = el.client_height().into();
                        cb.emit((rect.width(), rect.height(), width, height))
                    }
                }
            } else {
                unreachable!();
            }
        }) as Box<dyn Fn(Vec<ResizeObserverEntry>)>);

        (
            ResizeObserver::new(observer_closure.as_ref().unchecked_ref()).unwrap_throw(),
            observer_closure,
        )
    }

    /// Create a new SizeObserver for the given element which calls the given callback
    pub fn new<X>(el: &Element, callback: impl IntoSizeCallback<X>) -> Self {
        let (observer, _observer_closure) = Self::create_observer(callback.into_size_cb());
        observer.observe(el);

        Self {
            _observer_closure,
            observer,
        }
    }

    /// Create a new SizeObserver for the given element which calls the given callback
    /// allows to specify ResizeObserverOptions
    pub fn new_with_options<X>(
        el: &Element,
        callback: impl IntoSizeCallback<X>,
        options: ResizeObserverOptions,
    ) -> Self {
        let (observer, _observer_closure) = Self::create_observer(callback.into_size_cb());
        observer.observe_with_options(el, &options);

        Self {
            _observer_closure,
            observer,
        }
    }
}

impl Drop for SizeObserver {
    fn drop(&mut self) {
        self.observer.disconnect();
    }
}
