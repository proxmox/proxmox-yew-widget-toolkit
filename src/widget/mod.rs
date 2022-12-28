//! # Basic widgets

mod action_icon;
pub use action_icon::{ActionIcon, PwtActionIcon};

mod button;
pub use button::{Button, PwtButton};

mod column;
pub use column::Column;

mod container;
pub use container::Container;

mod dropdown;
pub use dropdown::{Dropdown, PwtDropdown, RenderDropdownPickerFn};

mod fa;
pub use fa::{Fa, PwtFa};

pub mod focus;

pub mod form2;

mod input;
pub use input::Input;

mod input_panel;
pub use input_panel::InputPanel;

mod mask;
pub use mask::{Mask, PwtMask};

mod menu;
pub use menu::{dispatch_menu_close_event, Menu, MenuBar, MenuEntry, MenuItem, MenuButton, MenuCheckbox, MenuEvent};

mod resizable;
pub use resizable::Resizable;

pub mod data_table;

mod dialog;
pub use dialog::{Dialog, PwtDialog};

mod panel;
pub use panel::Panel;

mod grid_picker;
pub use grid_picker::GridPicker;

mod row;
pub use row::Row;

mod segmented_button;
pub use segmented_button::{PwtSegmentedButton, SegmentedButton};

mod size_observer;
pub use size_observer::SizeObserver;

mod tab_bar;
pub use tab_bar::{PwtTabBar, TabBar, TabBarItem};

mod tab_panel;
pub use tab_panel::{PwtTabPanel, TabPanel, TabPanelRenderInfo};

mod theme_loader;
pub use theme_loader::ThemeLoader;

mod theme_selector;
pub use theme_selector::ThemeSelector;

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

pub fn get_unique_element_id() -> String {
    let id = UNIQUE_ELEMENT_ID.fetch_add(1, Ordering::SeqCst);
    format!("PwtElementId{}", id)
}
