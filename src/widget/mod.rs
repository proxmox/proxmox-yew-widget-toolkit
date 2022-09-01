//! # Basic widgets

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

mod grid_picker;
pub use grid_picker::GridPicker;

mod row;
pub use row::Row;

mod size_observer;
pub use size_observer::SizeObserver;

mod tab_bar;
pub use tab_bar::{TabBar, TabBarItem, PwtTabBar};

mod tab_panel;
pub use tab_panel::{TabPanel, TabPanelRenderInfo, PwtTabPanel};

mod theme_loader;
pub use theme_loader::ThemeLoader;

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
