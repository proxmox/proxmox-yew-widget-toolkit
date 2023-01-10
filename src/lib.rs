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
//! use pwt::prelude::*;
//! use pwt::widget::{Button, Column};
//!
//! let html = Column::new()
//!    .padding(2)
//!    .gap(2)
//!    .with_child("This is the first line (simple Text).")
//!    .with_child(Button::new("A Button"))
//!    //.with_child(html!{
//!    //    <h2>{"heading created using the Yew html macro"}</h2>
//!    //})
//!    .into();
//! ```
//!
//! The builders creates yew component properties, which can then be
//! transformed into Html. All component properties implements
//! `Into<Html>`.
//!

//! ## Callbacks
//!
//! Simple Widgets which corresponds to Html elements implements
//! [EventSubscriber](props::EventSubscriber). This trait provides builder
//! functions most Html event. By convention, JavaScript objects that
//! fire events have a corresponding "onevent" properties (named by
//! prefixing "on" to the name of the event). We use the same naming
//! convention for this kind of callbacks. It is possible to bind
//! multiple different callbacks to the same event - all callback will
//! be called when the event occur.
//!
//! Some components compute there own custom events. The naming
//! convention for those callbacks is "on_event" (please note the
//! underscore after "on") to distinguish custom events from Html
//! element events. It is **not** possible to bind multiple different
//! callbacks to the same event (only the last callback is called).

//! ## Widget Overview
//!
//! ### Layout widgets
//!
//! - [widget::Container]: Basically a wrapper for `<div>`.
//! - [widget::Row]: Horizontal container with flex layout.
//! - [widget::Column]: Vertical container with flex layout.
//! - [widget::Panel]: Container with title.
//! - [widget::InputPanel]: Container to create simple forms.
//! - [widget::TabPanel]: A set of layered items where only one item is displayed at a time.
//! - [widget::Toolbar]: Horizontal container for buttons.
//!

//! ### Modal Dialogs
//!
//! The [Dialog](widget::Dialog) widget implements a modal dialog
//! with a title. The widget is implemented using the relatively new
//! Html `<dialog>` tag in order to get correct focus handling.
//!
//! The [AlertDialog](widget::AlertDialog) is a convenient way to
//! display error messages.
//!
//! Dialogs are also handy for displaying input forms. The specialized
//! [EditWindow](component::EditWindow) makes it easy to implement such dialogs.


//! ### Forms and Fields
//!
//! All form fields support a set of common
//! [properties](props::FieldStdProps), which can be set using the
//! associated [builder](props::FieldBuilder) functions. All fields
//! can store there state inside a shared
//! [context](widget::form2::FormContext), which allow the
//! implementation of complex field interactions.
//!
//! The form context is automatically provided when putting the fields
//! inside a [Form](widget::form2::Form) or
//! [EditWindow](component::EditWindow).
//!
//! The following field types are available.
//!
//! - [widget::form2::Checkbox]: Checkbox or Radiobox field
//! - [widget::form2::Combobox]: Select value from a list of options.
//! - [widget::form2::Field]: Wrapper around standard Html fields.
//! - [widget::form2::Selector]: Select value from a picker widget.
//!
//! There are also special buttons for [reset](widget::form2::ResetButton)
//! and [submit](widget::form2::SubmitButton).


//! ### Buttons
//!
//! - [widget::Button]: Standard Html Button (Text, Icon + Text, Icon only).
//! - [widget::ActionIcon]: A clickable icon.
//! - [widget::SegmentedButton]: List of Buttons.
//!

//! ### Menus
//!
//! - [widget::Menu]:  A container for [MenuEntry](widget::MenuEntry)s.
//! - [widget::MenuBar]: Operating system like menu bar.
//! - [widget::MenuButton]: A button that opens a [Menu](widget::Menu).
//! - [widget::MenuCheckbox]: Checkbox/RadioGroup widget for [Menu](widget::Menu)s.

//! ### DataTable and Trees 
//!
//! The [DataTable](widget::data_table) widget is currently
//! the most complex widget. It is able to display tables and trees, and
//! has virtual scroll support.

//! ## Components Overview
//!
//! Components are more complex objects composed from several basic
//! widgets, and usually include advanced state handling (i.e. loading
//! data from an URL).
//!
//! - [widget::AlertDialog]: Display error messages.
//! - [component::EditWindow]: Input form inside a modal dialog.
//! - [component::KVGrid]: Grid with two columns (key and value).
//! - [component::NavigationMenu]: Navigation menu with routing support.
//! - [component::ObjectGrid]: Extends [KVGrid](component::KVGrid) with load/edit functionality.




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

#[doc(hidden)]
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

// Create wrapper which panics if called from target_arch!=wasm32
// This allows us to run tests with "cargo test".
#[cfg(not(target_arch="wasm32"))]
pub(crate) fn create_popper(_content: web_sys::Node, _tip: web_sys::Node, _opts: &JsValue) -> JsValue { unreachable!() }
#[cfg(not(target_arch="wasm32"))]
pub(crate) fn update_popper(_popper: &JsValue) { unreachable!() }
#[cfg(not(target_arch="wasm32"))]
pub(crate) fn show_modal_dialog(_dialog: web_sys::Node) { unreachable!() }
#[cfg(not(target_arch="wasm32"))]
pub(crate) fn close_dialog(_dialog: web_sys::Node) { unreachable!() }

// some helpers

use serde::Serialize;
pub fn to_js_value<T:  Serialize + ?Sized>(value: &T) -> Result<JsValue, serde_wasm_bindgen::Error> {
    value.serialize(&serde_wasm_bindgen::Serializer::json_compatible())
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
    pub use crate::props::CallbackMutScopeExt;
}
