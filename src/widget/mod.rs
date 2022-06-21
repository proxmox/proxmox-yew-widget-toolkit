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
//! - [Container]: Basically a wrapper for `<div>`.
//! - [Row]: Horizontal container with flex layout
//! - [Column]: Vertical container with flex layout
//! - [Panel]: Container with title.
//! - [InputPanel]: Container to create simple forms.
//! - [Toolbar]: Horizontal container for buttons.
//! - [VirtualScroll]: Container with virtual scrolling support.
//!
//! ### Forms and Fields
//!
//! ### Buttons

mod button;
pub use button::{Button, PwtButton};

mod column;
pub use column::Column;

mod container;
pub use container::Container;

mod dropdown;
pub use dropdown::{Dropdown, PwtDropdown};

mod fa;
pub use fa::{Fa, PwtFa};

pub mod focus;

pub mod form;

mod input_panel;
pub use input_panel::InputPanel;

mod mask;
pub use mask::{Mask, PwtMask};

mod resizable;
pub use resizable::Resizable;

mod data_table;
pub use data_table::{PwtDataTable, DataTable, DataTableColumn};

mod dialog;
pub use dialog::{Dialog, PwtDialog};

mod panel;
pub use panel::Panel;

mod picker;
pub use picker::GridPicker;

mod row;
pub use row::Row;

mod size_observer;
pub use size_observer::SizeObserver;

mod tab_bar;
pub use tab_bar::{TabBar, TabBarItem, PwtTabBar};

mod tab_panel;
pub use tab_panel::*;

mod theme_selector;
pub use theme_selector::ThemeSelector;

mod toolbar;
pub use toolbar::{Toolbar, PwtToolbar};

mod tooltip;
pub use tooltip::{Tooltip, PwtTooltip};

mod virtual_scroll;
pub use virtual_scroll::VirtualScroll;

mod visibility_observer;
pub use visibility_observer::VisibilityObserver;


use std::sync::atomic::{AtomicUsize, Ordering};

static UNIQUE_ELEMENT_ID: AtomicUsize = AtomicUsize::new(0);

pub fn get_unique_element_id() -> String {
    let id = UNIQUE_ELEMENT_ID.fetch_add(1, Ordering::SeqCst);
    format!("PwtElementId{}", id)
}

pub mod prelude {
    pub use crate::props::WidgetBuilder;
    pub use crate::props::ContainerBuilder;
    pub use crate::props::FieldBuilder;
    pub use crate::props::EventSubscriber;
}
