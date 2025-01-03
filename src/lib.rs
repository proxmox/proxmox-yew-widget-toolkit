//! # Proxmox Widget Toolkit (for Yew)
//!
//! This toolkit provides Yew components to build Single Page
//! Apps. It provides everything you need to build complex user
//! interfaces:
//!
//! - Basic layout widgets like [Row](widget::Row) and [Column](widget::Column) (using css flexbox).
//! - Standard widgets like buttons, menus, toolbars, tooltips, tab panels,
//! - Data table with virtual scroll, sorting, nested resizable headers, tree support ...
//! - List widget with virtual scroll support.
//! - Helpers to build complex forms easily ([widget::form])
//! - Modal dialogs
//! - Split pane with a draggable sparator.
//! - Dropdown with arbitrary complex content (i.e. [GridPicker](widget::GridPicker))
//! - Various navigation widgets like [NavigationDrawer](widget::nav::NavigationDrawer),
//!   [NavigationBar](touch::NavigationBar) and [NavigationRail](touch::NavigationRail)
//! - Html SVG Canvas bindings for easy 2D drawings.
//! - Flexible theming using CSS.
//!
//! # History
//!
//! The main goal was to rewrite the existing Proxmox user interfaces.
//! Hence the original style/theme of the widgets was to mimic the current
//! Proxmox UI style which uses a very dense layout.
//!
//! Later, a Material Design-like style was added that uses larger
//! spacing suitable for touch devices.
//!
//! ## Builder Pattern
//!
//! This toolkit uses the builder pattern to create Yew components.
//! Here's an example of creating a simple list:
//!
//! ```
//! use pwt::prelude::*;
//! use pwt::widget::{Button, Column};
//!
//! let html: Html = Column::new()
//!    .padding(2)
//!    .gap(2)
//!    .with_child("This is the first line (simple Text).")
//!    .with_child(Button::new("A Button"))
//!    .with_child(html!{
//!        <h2>{"heading created using the Yew html macro"}</h2>
//!    })
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
//! functions for most Html event. By convention, JavaScript objects that
//! fire events have a corresponding "onevent" properties (named by
//! prefixing "on" to the name of the event). We use the same naming
//! convention for this kind of callbacks. It is possible to bind
//! multiple different callbacks to the same event - all callbacks will
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
//! - [widget::FieldLabel]: Container to add a label to a [Field](widget::form::Field).
//! - [widget::SplitPane]: Container where children are separated by a draggable sparator.
//! - [widget::TabPanel]: A set of layered items where only one item is displayed at a time.
//! - [widget::Toolbar]: Horizontal container for buttons.
//! - [widget::MiniScroll]: Scrolled container usefull for toolbar and tab panels.
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
//! Dialogs are also handy for displaying input forms.

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
//! inside a [Form](widget::form::Form).
//!
//! The following field types are available.
//!
//! - [widget::form::Checkbox]: Checkbox input element, which stores values as boolean by default.
//! - [widget::form::RadioButton]: RadioButton field, which stores values as strings.
//! - [widget::form::Combobox]: Select value from a list of options.
//! - [widget::form::DisplayField]: Just display values (not editable).
//! - [widget::form::Field]: Wrapper around standard Html fields.
//! - [widget::form::Hidden]: Hidden fields.
//! - [widget::form::Number]: Number input element for common Rust number types,
//!   which stores values a number (not string).
//! - [widget::form::Selector]: Select value from a picker widget.
//! - [widget::form::TextArea]: Textarea input element.
//! - [widget::form::TristateBoolean]: Tristate Boolean widget (yes, no, null)
//!
//! There are also special buttons for [reset](widget::form::ResetButton)
//! and [submit](widget::form::SubmitButton).
//!
//! The [ManagedField](widget::form::ManagedField) trait is used as base
//! for input elements, because it simplifies the interaction with the
//! [context](widget::form::FormContext).
//!
//!
//! ### Other input Elements
//!
//! Some elements are html inputs, but do not act as form fields, i.e. they do
//! not interact with the form [context](widget::form::FormContext).
//!
//! The [Dropdown](widget::Dropdown) widget is used as base widget to
//! implement the the more complex [Combobox](widget::form::Combobox) and
//! [Selector](widget::form::Selector) widgets.
//!
//! The [SearchDropdown](widget::SearchDropdown) also uses the [Dropdown](widget::Dropdown)
//! to represent asynchrounous search results to the user.

//! ### Buttons
//!
//! - [widget::Button]: Standard Html Button (Text, Icon + Text, Icon only).
//! - [widget::ActionIcon]: A clickable icon.
//! - [widget::SegmentedButton]: List of Buttons.
//!

//! ### Menus
//!
//! - [widget::menu::Menu]:  A container for [MenuEntry](widget::menu::MenuEntry)s. Can also be
//!   used for Operating system like menu bars via the
//!   [new_menubar()](widget::menu::Menu::new_menubar) constructor
//! - [widget::menu::MenuButton]: A button that opens a [Menu](widget::menu::Menu).
//! - [widget::menu::MenuCheckbox]: Checkbox/RadioGroup widget for [Menu](widget::menu::Menu)s.
//! - [widget::nav::NavigationDrawer]: Navigation menu with routing support.

//! ### DataTable and Trees
//!
//! The [DataTable](widget::data_table) widget is currently
//! the most complex widget. It is able to display tables and trees, and
//! has virtual scroll support.

//! ### Lists
//!
//! The [List](widget::List) widget also supports virtual scrolling, and allows
//! using a html grid column layout.

//! ### Drawing Canvas
//!
//! The [Canvas](widget::canvas) component utilizes the Html `<svg>` element to
//! provide a full features 2D drawing interface.

//! ### Widgets for Touch devices
//!
//! Please note that these widgets are badly accessible with keyboard, so it is best
//! to avoid them for desktop applications (no focus handling).
//!
//! - [touch::ApplicationBar]: Top application bar.
//! - [touch::Fab]: Favorite action button.
//! - [touch::FabMenu]: Favorite actions button Menu.
//! - [touch::GestureDetector]: Gesture detector.
//! - [touch::MaterialApp]: Helper to implement applications that uses material design guidelines.
//! - [touch::NavigationBar]: Navigation bar (bottom)
//! - [touch::NavigationRail]: Navigation rail (left)
//! - [touch::PageView]: A scrollable list that works page by page.
//! - [touch::PageStack]: Stack of Pages using animation when switching pages.
//! - [touch::Scaffold]: Material Design visual page layout.
//! - [touch::SideDialog]: Modal Dialog with slide in/out animations.
//! - [touch::Slidable]: Slidable widget with directional slide actions that can be dismissed.
//! - [touch::SnackBar]: Display lightweight messages with an optional action button.

//! ## Router
//!
//! [Yew](https://yew.rs) provides a framework to implement
//! [routers](https://yew.rs/docs/concepts/router). To simplify that
//! further, some widgets can be used inside a
//! [state::NavigationContainer] which support fully automatic
//! routing. Please note that such navigation container can be nested.
//!
//! - [widget::TabPanel]
//! - [widget::nav::NavigationDrawer]
//!
//! ## Application implementation helpers
//!
//! - [widget::DesktopApp]: Application scaffold for desktop applications.
//! - [touch::MaterialApp]: Helper to implement applications that uses material design guidelines.
//!

//! ## DOM Helpers
//!
//! - [widget::SizeObserver]: Observe size changes.
//! - [widget::VisibilityObserver]: Observe visibility changes.
//!

mod async_abort_guard;
pub use async_abort_guard::AsyncAbortGuard;

mod async_pool;
pub use async_pool::AsyncPool;

pub mod css;
pub mod dom;
pub mod props;
pub mod state;
pub mod touch;

mod web_sys_abort_guard;
pub use web_sys_abort_guard::WebSysAbortGuard;

pub mod widget;

#[doc(hidden)]
pub mod gettext_runtime_format;

mod gettext_wrapper;
pub use gettext_wrapper::{
    gettext, gettext_noop, init_i18n, init_i18n_from_blob, init_i18n_from_url, ngettext, npgettext,
    pgettext,
};

mod tr;

#[doc(hidden)]
pub mod web_sys_ext;

use anyhow::{format_err, Error};

/// Convert JS errors to [Error]
pub fn convert_js_error(js_err: ::wasm_bindgen::JsValue) -> Error {
    if let Ok(error) = ::wasm_bindgen::JsCast::dyn_into::<js_sys::Error>(js_err) {
        format_err!("{}", error.message())
    } else {
        format_err!("unknown js error: error is no ERROR object")
    }
}

// Bindgen javascript code from js-helper-module.js
use wasm_bindgen::{self, prelude::*};
#[wasm_bindgen(module = "/js-helper-module.js")]
#[cfg(target_arch = "wasm32")]
extern "C" {
    fn test_alert();

    //Dialog bindings
    pub fn show_modal_dialog(dialog: web_sys::Node);
    pub fn show_dialog(dialog: web_sys::Node);
    pub fn close_dialog(dialog: web_sys::Node);
    pub fn show_popover(popover: web_sys::Node);
    pub fn hide_popover(popover: web_sys::Node);
    pub fn toggle_popover(popover: web_sys::Node);
    pub fn toggle_event_is_closed(event: web_sys::Event) -> bool;
}

// Create wrapper which panics if called from target_arch!=wasm32
// This allows us to run tests with "cargo test".
#[cfg(not(target_arch = "wasm32"))]
pub use panic_wrapper::*;
#[cfg(not(target_arch = "wasm32"))]
mod panic_wrapper {
    pub fn show_modal_dialog(_dialog: web_sys::Node) {
        unreachable!()
    }
    pub fn show_dialog(_dialog: web_sys::Node) {
        unreachable!()
    }
    pub fn close_dialog(_dialog: web_sys::Node) {
        unreachable!()
    }
    pub fn show_popover(_popover: web_sys::Node) {
        unreachable!()
    }
    pub fn hide_popover(_popover: web_sys::Node) {
        unreachable!()
    }
    pub fn toggle_popover(_popover: web_sys::Node) {
        unreachable!()
    }
}

// some helpers

use serde::Serialize;
/// Serialize data into a [JsValue] using `serde_wasm_bindgen`.
pub fn to_js_value<T: Serialize + ?Sized>(value: &T) -> Result<JsValue, serde_wasm_bindgen::Error> {
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
///
/// This also re-exports the yew prelude: `use yew::prelude::*;`
///
/// # Note
///
/// You may want to use [crate::touch::prelude] instead if you develop a mobile app.
pub mod prelude {
    #[doc(hidden)]
    pub use yew::prelude::*;

    pub use crate::props::CallbackMutScopeExt;
    pub use crate::props::ContainerBuilder;
    pub use crate::props::CssBorderBuilder;
    pub use crate::props::CssMarginBuilder;
    pub use crate::props::CssPaddingBuilder;
    pub use crate::props::EventSubscriber;
    pub use crate::props::FieldBuilder;
    pub use crate::props::IntoOptionalKey;
    pub use crate::props::WidgetBuilder;
    pub use crate::props::WidgetStyleBuilder;

    pub use crate::tr;
    pub use crate::{gettext, gettext_noop, ngettext, npgettext, pgettext};
}

#[macro_export]
macro_rules! impl_to_html {
    ($struct_name:ident) => {
        impl ToHtml for $struct_name {
            fn to_html(&self) -> Html {
                self.clone().into()
            }

            fn into_html(self) -> Html {
                self.into()
            }
        }
    };
}
