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

//! ## Callbacks
//!
//! Widgets with corresponding HTML element implements
//! [props::EventSubscriber], which provides builder functions most
//! HTML event. By convention, JavaScript objects that fire events
//! have a corresponding "onevent" properties (named by prefixing "on"
//! to the name of the event). We use the same naming convention for
//! this kind of callbacks.
//!
//! Some components compute there own custom events. The naming
//! convention for those callbacks is "on_event" (please note the
//! underscore after "on") to distinguish custom events from HTML
//! element events.

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
//! All form fields support a set of common
//! [properties](props::FieldStdProps), which can be set using the
//! associated [builder](props::FieldBuilder) functions. All fields
//! can store there state inside a shared
//! [context](widget::form::FormContext), which allow the
//! implementation of complex field interactions.
//!
//! The form context is automatically provided when putting the fields
//! inside a [widget::form::Form] or [component::EditWindow]. Custom
//! components can provide a form context using
//! [widget::form::form_context_provider].
//!
//! The following field types are available.
//!
//! - [widget::form::Checkbox]: Checkbox field
//! - [widget::form::Combobox]: Select value from a list of options.
//! - [widget::form::Field]: Wrapper around standard HTML fields.
//! - [widget::form::Selector]: Select value from a picker widget.
//!
//! There are also special buttons for [reset](widget::form::Reset)
//! and [submit](widget::form::Submit).


//! ### Buttons
//!

//! ## Components Overview
//!
//! Components are more complex objects composed from several basic
//! widgets, and usually include advanced state handling.
//!
//! -
//!

//! ## Router
//!
//! [Yew](https://yew.rs) provides a framework to implement
//! [routers](https://yew.rs/docs/concepts/router). To simplify that
//! further, some widgets can be turned into an
//! [state::NavigationContainer] which support fully automatic
//! routing. Please note that navigation container can be nested.
//!
//! - [widget::TabPanel::navigation_container]
//! - [component::NavigationMenu::navigation_container]

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

/// # Prelude, which include all builder traits.
///
/// Many builder function are implemented using trait
/// implementations. To acces those functions, the trait objects need
/// to be visiable. The easiest way to do that is:
///
/// ```
/// use pwt::prelude::*;
/// ```

pub mod prelude {
    pub use crate::props::WidgetBuilder;
    pub use crate::props::ContainerBuilder;
    pub use crate::props::FieldBuilder;
    pub use crate::props::EventSubscriber;
}
