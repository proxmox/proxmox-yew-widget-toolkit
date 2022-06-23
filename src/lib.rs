mod erc;
pub use erc::Erc;

pub mod props;
pub mod state;
pub mod widget;
pub mod component;

pub mod web_sys_ext;

// Bindgen java code from js-helper-module.js
use wasm_bindgen::{self, prelude::*};
#[wasm_bindgen(module = "/js-helper-module.js")]
#[cfg(target_arch="wasm32")]
extern "C" {
    fn async_sleep(ms: i32) -> js_sys::Promise;
    fn get_cookie() -> String;
    fn set_auth_cookie(value: &str);
    fn clear_auth_cookie();
    fn test_alert();

    // Popper binding
    fn create_popper(content: web_sys::Node, tip: web_sys::Node, opts: &JsValue) -> JsValue;
    fn update_popper(popper: &JsValue);

    //Dialog bindings
    fn show_modal_dialog(dialog: web_sys::Node);
    fn close_dialog(dialog: web_sys::Node);
}


pub fn session_storage() -> Option<web_sys::Storage> {
    let window = match web_sys::window() {
        None => {
            log::error!("session_storage: no window");
            return None;
        }
        Some(window) => window,
    };

    let store = match window.session_storage() {
        Ok(Some(store)) => store,
        Ok(None) => {
            log::error!("session_storage: no session_storage");
            return None;
        }
        Err(_) => {
            log::error!("session_storage: security error");
            return None;
        }
    };

    Some(store)
}

pub fn local_storage() -> Option<web_sys::Storage> {
    let window = match web_sys::window() {
        None => {
            log::error!("local_storage: no window");
            return None;
        }
        Some(window) => window,
    };

    let store = match window.local_storage() {
        Ok(Some(store)) => store,
        Ok(None) => {
            log::error!("local_storage: no local_storage");
            return None;
        }
        Err(_) => {
            log::error!("local_storage: security error");
            return None;
        }
    };

    Some(store)
}

pub fn store_use_dark_theme(dark: bool) {
    if let Some(store) = local_storage() {
        if let Err(_) = store.set_item("UseDarkTheme", &dark.to_string()) {
            log::error!("store_use_dark_theme: store.set_item() failed");
        }
    }
}

pub fn load_use_dark_theme() -> Option<bool>{
    local_storage()
        .and_then(|store| {
            store
                .get_item("UseDarkTheme")
                .unwrap_or(None)
                .map(|s| s.parse().unwrap_or(false))
        })
}

pub mod prelude {
    pub use crate::props::WidgetBuilder;
    pub use crate::props::ContainerBuilder;
    pub use crate::props::FieldBuilder;
    pub use crate::props::EventSubscriber;
}
