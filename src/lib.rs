//! # Proxmox Widget Toolkit (for Yew)
//!
//! This toolkit provides Yew components to build Single Page
//! Apps. The main goal is rewriting the existing Proxmox UIs, so the
//! style/theme of the widgets mimics the current Proxmox UI style.
//!
//! ## Builder Pattern
//!
//! This toolkit uses the builder pattern to create Yew components. It
//! is currently not possible to create the components using the Yew
//! 'html' macro.
//!
//! Here's an example of creating a simple list:
//! ```
//! Column::new()
//!    .padding(2)
//!    .gap(2)
//!    .with_child("This is the first line (simple Text).")
//!    .with_child(Button::new("A Button"))
//!    .with_child(html!{
//!        <h2>{"heading created using the Yew html macro"}</h2>
//!    })
//! ```
//! All builder implements `Into<Html>`.
//!
//! ## Widget Overview
//!
//! ### Layout widgets
//!
//! - [widget::Container]: Basically a wrapper for `<div>`.
//! - [widget::Row]: Horizontal container with flex layout
//! - [widget::Column]: Vertical container with flex layout
//! - [widget::Panel]: Container with title.
//! - [widget::InputPanel]: Container to create simple forms.
//! - [widget::TabPanel]: A set of layered items where only one item is displayed at a time.
//! - [widget::Toolbar]: Horizontal container for buttons.
//! - [widget::VirtualScroll]: Container with virtual scrolling support.
//!
//! ### Modal Dialogs
//!
//! ### Forms and Fields
//!
//! ### Buttons
//!
//! ## Components Overview
//!
//! Components are more complex widget???

pub mod props;
pub mod state;
pub mod theme;
pub mod widget;
pub mod component;

pub mod web_sys_ext;

// Bindgen java code from js-helper-module.js
use wasm_bindgen::{self, prelude::*};
#[wasm_bindgen(module = "/js-helper-module.js")]
#[cfg(target_arch="wasm32")]
extern "C" {
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

pub mod prelude {
    pub use crate::props::WidgetBuilder;
    pub use crate::props::ContainerBuilder;
    pub use crate::props::FieldBuilder;
    pub use crate::props::EventSubscriber;
}
