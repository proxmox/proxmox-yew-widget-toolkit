// copied from https://raw.githubusercontent.com/jetli/yew-hooks/main/crates/yew-hooks/src/web_sys_ext.rs

//! ResizeObserver/ClipboardEvent in web-sys is unstable and
//! requires `--cfg=web_sys_unstable_apis` to be activated,
//! which is inconvenient, so copy the binding code here for now.
#![allow(unused_imports)]
#![allow(clippy::unused_unit)]
use wasm_bindgen::{self, prelude::*};
use web_sys::{DomRectReadOnly, Element, Event};

#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = ResizeObserver , typescript_type = "ResizeObserver")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type ResizeObserver;

    #[wasm_bindgen(catch, constructor, js_class = "ResizeObserver")]
    pub fn new(callback: &::js_sys::Function) -> Result<ResizeObserver, JsValue>;

    # [wasm_bindgen (method , structural , js_class = "ResizeObserver" , js_name = disconnect)]
    pub fn disconnect(this: &ResizeObserver);

    # [wasm_bindgen (method , structural , js_class = "ResizeObserver" , js_name = observe)]
    pub fn observe(this: &ResizeObserver, target: &Element);

    # [wasm_bindgen (method , structural , js_class = "ResizeObserver" , js_name = observe)]
    pub fn observe_with_options(
        this: &ResizeObserver,
        target: &Element,
        options: &ResizeObserverOptions,
    );
}

#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = ResizeObserverEntry , typescript_type = "ResizeObserverEntry")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type ResizeObserverEntry;

    # [wasm_bindgen (structural , method , getter , js_class = "ResizeObserverEntry" , js_name = target)]
    pub fn target(this: &ResizeObserverEntry) -> Element;

    # [wasm_bindgen (structural , method , getter , js_class = "ResizeObserverEntry" , js_name = contentRect)]
    pub fn content_rect(this: &ResizeObserverEntry) -> DomRectReadOnly;
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeObserverBoxOptions {
    BorderBox = "border-box",
    ContentBox = "content-box",
    DevicePixelContentBox = "device-pixel-content-box",
}

#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = ResizeObserverOptions)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type ResizeObserverOptions;
}
impl ResizeObserverOptions {
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret
    }
    pub fn box_(&mut self, val: ResizeObserverBoxOptions) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("box"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
}

impl Default for ResizeObserverOptions {
    fn default() -> Self {
        Self::new()
    }
}
