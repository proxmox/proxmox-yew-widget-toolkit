//! # Basic widgets

mod action_icon;
pub use action_icon::{ActionIcon, PwtActionIcon};

mod alert_dialog;
pub use alert_dialog::{AlertDialog, PwtAlertDialog};

mod message_box;
pub use message_box::{MessageBox, MessageBoxButtons, PwtMessageBox};

pub mod align;

mod button;
pub use button::{Button, PwtButton};

pub mod canvas;

mod card;
pub use card::Card;

mod catalog_loader;
pub use catalog_loader::{CatalogLoader, PwtCatalogLoader};

mod column;
pub use column::Column;

mod container;
pub use container::Container;

mod desktop_app;
pub use desktop_app::{DesktopApp, PwtDesktopApp};

mod dropdown;
pub use dropdown::{Dropdown, PwtDropdown, RenderDropdownPickerFn};

pub mod dom;

mod fa;
pub use fa::{Fa, PwtFa};

pub mod focus;

pub mod form;

mod input;
pub use input::Input;

mod input_panel;
pub use input_panel::InputPanel;

mod language_selector;
pub use language_selector::{LanguageSelector, ProxmoxLanguageSelector};

mod mask;
pub use mask::{Mask, PwtMask};

mod mini_scroll;
pub use mini_scroll::{MiniScroll, MiniScrollMode, PwtMiniScroll};

pub mod nav;

pub mod menu;

mod meter;
pub use meter::Meter;

mod split_pane;
pub use split_pane::{Pane, PwtSplitPane, SplitPane};

pub mod data_table;

mod dialog;
pub use dialog::{Dialog, PwtDialog};

mod panel;
pub use panel::Panel;

mod grid_picker;
pub use grid_picker::GridPicker;

mod progress;
pub use progress::Progress;

mod row;
pub use row::Row;

mod rtl_switcher;
pub use rtl_switcher::RtlSwitcher;

mod selection_view;
pub use selection_view::{PwtSelectionView, SelectionView, SelectionViewRenderInfo};

mod segmented_button;
pub use segmented_button::{PwtSegmentedButton, SegmentedButton};

mod size_observer;
pub use size_observer::{IntoSizeCallback, SizeObserver};

mod tab;
pub use tab::*;

mod theme_loader;
pub use theme_loader::ThemeLoader;

mod theme_density_selector;
pub use theme_density_selector::{PwtThemeDensitySelector, ThemeDensitySelector};

mod theme_mode_selector;
pub use theme_mode_selector::ThemeModeSelector;

mod theme_name_selector;
pub use theme_name_selector::ThemeNameSelector;

mod toolbar;
pub use toolbar::{PwtToolbar, Toolbar};

mod tooltip;
pub use tooltip::{PwtTooltip, Tooltip};

//mod virtual_scroll;
//pub use virtual_scroll::VirtualScroll;

mod visibility_observer;
pub use visibility_observer::VisibilityObserver;

use std::sync::atomic::{AtomicUsize, Ordering};

static UNIQUE_ELEMENT_ID: AtomicUsize = AtomicUsize::new(0);

/// Creates an unique element ID (`PwtElementId{unique_num}`)
pub fn get_unique_element_id() -> String {
    let id = UNIQUE_ELEMENT_ID.fetch_add(1, Ordering::SeqCst);
    format!("PwtElementId{}", id)
}

/// Creates a nicely formatted error message.
pub fn error_message(text: &str, class: &str) -> yew::Html {
    message_box::message(text, class, "fa-exclamation-triangle")
}
