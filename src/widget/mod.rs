//! # Basic widgets

mod action_icon;
pub use action_icon::ActionIcon;

mod alert_dialog;
pub use alert_dialog::AlertDialog;
#[doc(hidden)]
pub use alert_dialog::PwtAlertDialog;

mod message_box;
#[doc(hidden)]
pub use message_box::PwtMessageBox;
pub use message_box::{MessageBox, MessageBoxButtons};

mod button;
#[doc(hidden)]
pub use button::PwtButton;
pub use button::{Button, ButtonType};

pub mod canvas;

mod card;
pub use card::Card;

mod catalog_loader;
pub use catalog_loader::CatalogLoader;
#[doc(hidden)]
pub use catalog_loader::PwtCatalogLoader;

mod column;
pub use column::Column;

mod container;
pub use container::Container;

mod confirm_dialog;
pub use confirm_dialog::ConfirmDialog;

mod desktop_app;
pub use desktop_app::DesktopApp;
#[doc(hidden)]
pub use desktop_app::PwtDesktopApp;

mod trigger;
pub use trigger::Trigger;

mod dropdown;
#[doc(hidden)]
pub use dropdown::PwtDropdown;
pub use dropdown::{Dropdown, DropdownController};

mod fa;
pub use fa::Fa;

mod file_button;
pub use file_button::FileButton;
#[doc(hidden)]
pub use file_button::PwtFileButton;

pub mod form;

mod field_label;
pub use field_label::FieldLabel;
#[doc(hidden)]
pub use field_label::PwtFieldLabel;

mod image;
pub use image::Image;

mod input;
pub use input::Input;

mod input_panel;
pub use input_panel::{FieldPosition, InputPanel, Labelable};

mod language_selector;
pub use language_selector::LanguageSelector;
#[doc(hidden)]
pub use language_selector::PwtLanguageSelector;

// Old code, no longer used.
// mod list_simple;
// pub use list_simple::{List, PwtList};

mod list;
pub use list::{List, ListTile, ListTileObserver};

#[doc(hidden)]
pub use list::{PwtList, PwtListTileObserver};

mod mask;
pub use mask::Mask;
#[doc(hidden)]
pub use mask::PwtMask;

mod mini_scroll;
#[doc(hidden)]
pub use mini_scroll::PwtMiniScroll;
pub use mini_scroll::{MiniScroll, MiniScrollMode};

pub mod nav;

pub mod menu;

mod meter;
pub use meter::Meter;

mod split_pane;
#[doc(hidden)]
pub use split_pane::PwtSplitPane;
pub use split_pane::{Pane, SplitPane};

pub mod data_table;

mod dialog;
pub use dialog::Dialog;
#[doc(hidden)]
pub use dialog::PwtDialog;

mod panel;
pub use panel::Panel;

mod grid_picker;
pub use grid_picker::GridPicker;
#[doc(hidden)]
pub use grid_picker::PwtGridPicker;

mod progress;
pub use progress::Progress;

mod row;
pub use row::Row;

mod rtl_switcher;
pub use rtl_switcher::RtlSwitcher;

mod search_dropdown;
#[doc(hidden)]
pub use search_dropdown::PwtSearchDropdown;
pub use search_dropdown::{FilteredLoadCallback, SearchDropdown, SearchDropdownRenderArgs};

mod selection_view;
#[doc(hidden)]
pub use selection_view::PwtSelectionView;
pub use selection_view::{SelectionView, SelectionViewRenderInfo, VisibilityContext};

mod segmented_button;
#[doc(hidden)]
pub use segmented_button::PwtSegmentedButton;
pub use segmented_button::SegmentedButton;

mod size_observer;
#[doc(hidden)]
pub use size_observer::PwtSizeObserver;
pub use size_observer::SizeObserver;

mod tab;
#[doc(hidden)]
pub use tab::{PwtTabBar, PwtTabPanel};
pub use tab::{TabBar, TabBarItem, TabBarStyle, TabPanel};

mod theme_loader;
#[doc(hidden)]
pub use theme_loader::PwtThemeLoader;
pub use theme_loader::ThemeLoader;

mod theme_density_selector;
#[doc(hidden)]
pub use theme_density_selector::PwtThemeDensitySelector;
pub use theme_density_selector::ThemeDensitySelector;

mod theme_mode_selector;
#[doc(hidden)]
pub use theme_mode_selector::PwtThemeModeSelector;
pub use theme_mode_selector::ThemeModeSelector;

mod theme_name_selector;
#[doc(hidden)]
pub use theme_name_selector::PwtThemeNameSelector;
pub use theme_name_selector::ThemeNameSelector;

mod toolbar;
#[doc(hidden)]
pub use toolbar::PwtToolbar;
pub use toolbar::Toolbar;

mod tooltip;
#[doc(hidden)]
pub use tooltip::PwtTooltip;
pub use tooltip::Tooltip;

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
pub fn error_message(text: &str) -> Row {
    message_box::message(text, "fa-exclamation-triangle")
}
