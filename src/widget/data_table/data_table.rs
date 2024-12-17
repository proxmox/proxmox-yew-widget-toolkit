use std::collections::HashSet;
use std::marker::PhantomData;
use std::rc::Rc;

use derivative::Derivative;

use gloo_timers::callback::Timeout;
use wasm_bindgen::JsCast;

use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::dom::focus::{focus_inside_el, focus_inside_input, focus_next_el};
use crate::dom::{DomSizeObserver, IntoHtmlElement};
use crate::prelude::*;
use crate::props::{
    AsClassesMut, AsCssStylesMut, CallbackMut, CssLength, CssStyles, IntoEventCallbackMut,
    SorterFn, WidgetStyleBuilder,
};
use crate::state::{DataStore, Selection, SelectionObserver};
use crate::widget::{get_unique_element_id, Column, Container};

use super::{
    create_indexed_header_list, CellConfiguration, DataTableColumn, DataTableHeader,
    DataTableKeyboardEvent, DataTableMouseEvent, DataTableRow, DataTableRowRenderCallback,
    HeaderWidget, IndexedHeader, IntoOptionalDataTableRowRenderCallback,
};

pub enum HeaderMsg<T: 'static> {
    ToggleSelectAll,
    ColumnWidthChange(Vec<f64>),
    ColumnHiddenChange(Vec<bool>),
    ChangeSort(SorterFn<T>),
}

pub enum Msg<T: 'static> {
    SelectionChange,
    DataChange,
    ScrollTo(i32, i32),
    ViewportResize(f64, f64, f64),
    TableResize(f64, f64),
    KeyDown(KeyboardEvent),
    CursorDown(usize, bool, bool),
    CursorUp(usize, bool, bool),
    CursorLeft,
    CursorRight,
    ItemClick(Key, Option<usize>, MouseEvent, bool),
    ItemDblClick(Key, Option<usize>, MouseEvent),
    FocusChange(bool, bool),
    Header(HeaderMsg<T>),
}

/// Row selction status
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RowSelectionStatus {
    /// Nothing is selected.
    Nothing,
    /// Some rows are selected.
    Some,
    /// All (filtered) rows are selected.
    All,
}

#[derive(Copy, Clone, Debug, PartialEq, Default)]
/// Determines the mode for using multiple selection.
pub enum MultiSelectMode {
    #[default]
    /// Selecting multiple values is only possible with CTRL/SHIFT keyboard keys. (default)
    Default,
    /// Clicking a row toggles it instead of making it the only selection.
    Simple,
}

/// Data Table/Tree with virual scroll.
///
/// # Features
///
/// - Virtual scrolling.
/// - Trees and Lists.
/// - Selection/Cursor management
/// - Nested header definitions.
/// - Header menus (hide, sort, ...).
/// - Resizable headers.
/// - ARIA support: <https://www.w3.org/WAI/ARIA/apg/patterns/grid/>.
///
/// # Keyboard bindings
///
/// * `Right Arrow`: Moves focus one cell to the right. If focus is on the right-most cell in the
///    row, focus does not move.
///
/// * `Left Arrow`: Moves focus one cell to the left. If focus is on the left-most cell in the row,
///    focus does not move.
///
/// * `Down Arrow`: Moves focus one cell down. If focus is on the bottom cell in the column, focus
///    does not move.
///
/// * `Up Arrow`: Moves focus one cell Up. If focus is on the top cell in the column, focus does
///    not move.
///
/// * `Page Down`: Moves focus down one page. If focus is in the last row of the grid, focus does
///    not move.
///
/// * `Page Up`: Moves focus up one page. If focus is in the first row of the grid, focus does not
///    move.
///
/// * `Home`: moves focus to the first cell in the row that contains focus.
///
/// * `End`: moves focus to the last cell in the row that contains focus.
#[derive(Properties, Derivative)]
#[derivative(Clone(bound = ""), PartialEq(bound = ""))]
pub struct DataTable<S: DataStore> {
    #[prop_or_default]
    node_ref: NodeRef,
    /// Yew key property.
    #[prop_or_default]
    pub key: Option<Key>,

    /// CSS class of the container.
    #[prop_or_default]
    pub class: Classes,

    /// CSS styles of the container
    #[prop_or_default]
    pub styles: CssStyles,

    #[derivative(PartialEq(compare_with = "Rc::ptr_eq"))]
    headers: Rc<Vec<DataTableHeader<S::Record>>>,

    // The data collection ([Store] or [TreeStore](crate::state::TreeStore)).
    store: S,

    /// Set the class, style and attributes for Cells
    #[prop_or_default]
    pub cell_configuration: CellConfiguration,

    /// CSS class for header cells (default is "pwt-datatable-header-cell").
    #[prop_or_default]
    pub header_class: Classes,

    /// Show vertical borders.
    #[prop_or_default]
    pub bordered: bool,

    /// Disable horizontal borders.
    #[prop_or_default]
    pub borderless: bool,

    /// Emphase rows when you mouse over them.
    #[prop_or_default]
    pub hover: bool,

    /// Use a striped color scheme for rows.
    #[prop_or(true)]
    pub striped: bool,

    /// Vertical alignment of cells inside the row.
    ///
    /// Possible values are "baseline" (default), "top", "middle" and
    /// "bottom".
    #[prop_or_default]
    pub vertical_align: Option<AttrValue>,

    /// Virtual Scroll
    ///
    /// Virtual scroll is enabled by default for tables with more than 30 rows.
    #[prop_or_default]
    pub virtual_scroll: Option<bool>,

    /// Minimum row height (default 22)
    ///
    /// Sets the minmum height for table rows. This is also used by
    /// the virtual scrolling algorithm to compute the maximal number
    /// of visible rows.
    #[prop_or(22)]
    pub min_row_height: usize,

    /// Selection object.
    #[prop_or_default]
    pub selection: Option<Selection>,

    /// Automatically select the focused row.
    #[prop_or(true)]
    pub autoselect: bool,

    /// Show the header.
    #[prop_or(true)]
    pub show_header: bool,

    /// Allow the header to take focus.
    #[prop_or(true)]
    pub header_focusable: bool,

    /// Row click callback
    #[prop_or_default]
    pub on_row_click: Option<CallbackMut<DataTableMouseEvent>>,

    /// Row double click callback
    #[prop_or_default]
    pub on_row_dblclick: Option<CallbackMut<DataTableMouseEvent>>,

    /// Row keydown callback
    #[prop_or_default]
    pub on_row_keydown: Option<CallbackMut<DataTableKeyboardEvent>>,

    /// Row context click callback
    #[prop_or_default]
    pub on_row_context_click: Option<CallbackMut<DataTableMouseEvent>>,

    #[prop_or_default]
    pub row_render_callback: Option<DataTableRowRenderCallback<S::Record>>,

    #[prop_or_default]
    pub multiselect_mode: MultiSelectMode,
}

impl<S: DataStore> AsClassesMut for DataTable<S> {
    fn as_classes_mut(&mut self) -> &mut yew::Classes {
        &mut self.class
    }
}

impl<S: DataStore> AsCssStylesMut for DataTable<S> {
    fn as_css_styles_mut(&mut self) -> &mut crate::props::CssStyles {
        &mut self.styles
    }
}

impl<S: DataStore> CssBorderBuilder for DataTable<S> {}
impl<S: DataStore> CssPaddingBuilder for DataTable<S> {}
impl<S: DataStore> CssMarginBuilder for DataTable<S> {}
impl<S: DataStore> WidgetStyleBuilder for DataTable<S> {}

static VIRTUAL_SCROLL_TRIGGER: usize = 30;

impl<S: DataStore> DataTable<S> {
    /// Create a new instance.
    ///
    /// The store is either a [Store](crate::state::Store) or [TreeStore](crate::state::TreeStore).
    pub fn new(headers: Rc<Vec<DataTableHeader<S::Record>>>, store: S) -> Self {
        yew::props!(DataTable<S> { headers, store })
    }

    /// Builder style method to set the yew `node_ref`.
    pub fn node_ref(mut self, node_ref: ::yew::html::NodeRef) -> Self {
        self.node_ref = node_ref;
        self
    }

    /// Builder style method to set the yew `key` property.
    pub fn key(mut self, key: impl IntoOptionalKey) -> Self {
        self.key = key.into_optional_key();
        self
    }

    /// Builder style method to add a html class.
    pub fn class(mut self, class: impl Into<Classes>) -> Self {
        self.add_class(class);
        self
    }

    /// Method to add a html class.
    pub fn add_class(&mut self, class: impl Into<Classes>) {
        self.class.push(class);
    }

    /// Builder style method to set the configuration for cells.
    pub fn cell_configuration(mut self, config: impl Into<CellConfiguration>) -> Self {
        self.set_cell_configuration(config);
        self
    }

    /// Method to set the configuration for cells.
    pub fn set_cell_configuration(&mut self, config: impl Into<CellConfiguration>) {
        self.cell_configuration = config.into();
    }

    /// Builder style method to add a html class for header cells.
    pub fn header_class(mut self, class: impl Into<Classes>) -> Self {
        self.add_header_class(class);
        self
    }

    /// Method to add a html class for header cells.
    pub fn add_header_class(&mut self, class: impl Into<Classes>) {
        self.header_class.push(class);
    }

    /// Builder style method to set striped mode.
    pub fn striped(mut self, striped: bool) -> Self {
        self.set_striped(striped);
        self
    }

    /// Method to set striped mode.
    pub fn set_striped(&mut self, striped: bool) {
        self.striped = striped;
    }

    /// Builder style method to set hover flag.
    pub fn hover(mut self, hover: bool) -> Self {
        self.set_hover(hover);
        self
    }

    /// Method to set hover flag.
    pub fn set_hover(&mut self, hover: bool) {
        self.hover = hover;
    }

    /// Builder style method to enable vertical borders.
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.set_bordered(bordered);
        self
    }

    /// Method to enable vertical borders.
    pub fn set_bordered(&mut self, bordered: bool) {
        self.bordered = bordered;
    }

    /// Builder style method to disable horizontal borders.
    pub fn borderless(mut self, borderless: bool) -> Self {
        self.set_borderless(borderless);
        self
    }

    /// Method to disable horizontal borders.
    pub fn set_borderless(&mut self, borderless: bool) {
        self.borderless = borderless;
    }

    /// Builder style method to set the vertical cell alignment.
    pub fn vertical_align(mut self, align: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_vertical_align(align);
        self
    }

    /// Method to set the vertical cell alignment.
    pub fn set_vertical_align(&mut self, align: impl IntoPropValue<Option<AttrValue>>) {
        self.vertical_align = align.into_prop_value();
    }

    /// Builder style method to set the virtual scroll flag.
    pub fn virtual_scroll(mut self, virtual_scroll: impl IntoPropValue<Option<bool>>) -> Self {
        self.set_virtual_scroll(virtual_scroll);
        self
    }

    /// Method to set the virtual scroll flag.
    pub fn set_virtual_scroll(&mut self, virtual_scroll: impl IntoPropValue<Option<bool>>) {
        self.virtual_scroll = virtual_scroll.into_prop_value();
    }

    /// Builder style method to set the minimum row height.
    pub fn min_row_height(mut self, min_row_height: usize) -> Self {
        self.set_min_row_height(min_row_height);
        self
    }

    /// Method to set the minimum row height.
    pub fn set_min_row_height(&mut self, min_row_height: usize) {
        self.min_row_height = min_row_height;
    }

    /// Builder style method to set the autoselect flag.
    pub fn autoselect(mut self, autoselect: impl IntoPropValue<bool>) -> Self {
        self.set_autoselect(autoselect);
        self
    }

    /// Method to set the autoselect flag.
    pub fn set_autoselect(&mut self, autoselect: impl IntoPropValue<bool>) {
        self.autoselect = autoselect.into_prop_value();
    }

    /// Builder style method to set the show_header flag.
    pub fn show_header(mut self, show_header: impl IntoPropValue<bool>) -> Self {
        self.set_show_header(show_header);
        self
    }

    /// Method to set the show_header flag.
    pub fn set_show_header(&mut self, show_header: impl IntoPropValue<bool>) {
        self.show_header = show_header.into_prop_value();
    }

    /// Builder style method to set the header_focusable flag.
    pub fn header_focusable(mut self, header_focusable: impl IntoPropValue<bool>) -> Self {
        self.set_header_focusable(header_focusable);
        self
    }

    /// Method to set the header_focusable flag.
    pub fn set_header_focusable(&mut self, header_focusable: impl IntoPropValue<bool>) {
        self.header_focusable = header_focusable.into_prop_value();
    }

    /// Builder style method to set the selection model.
    pub fn selection(mut self, selection: impl IntoPropValue<Option<Selection>>) -> Self {
        self.selection = selection.into_prop_value();
        self
    }

    /// Builder style method to set the row click callback.
    pub fn on_row_click(mut self, cb: impl IntoEventCallbackMut<DataTableMouseEvent>) -> Self {
        self.on_row_click = cb.into_event_cb_mut();
        self
    }

    /// Builder style method to set the row context click callback.
    pub fn on_row_context_click(
        mut self,
        cb: impl IntoEventCallbackMut<DataTableMouseEvent>,
    ) -> Self {
        self.on_row_context_click = cb.into_event_cb_mut();
        self
    }

    /// Builder style method to set the row double click callback.
    pub fn on_row_dblclick(mut self, cb: impl IntoEventCallbackMut<DataTableMouseEvent>) -> Self {
        self.on_row_dblclick = cb.into_event_cb_mut();
        self
    }

    /// Builder style method to set the row keydown callback.
    pub fn on_row_keydown(mut self, cb: impl IntoEventCallbackMut<DataTableKeyboardEvent>) -> Self {
        self.on_row_keydown = cb.into_event_cb_mut();
        self
    }

    /// Builder style method to set the row render callback.
    pub fn row_render_callback(
        mut self,
        cb: impl IntoOptionalDataTableRowRenderCallback<S::Record>,
    ) -> Self {
        self.row_render_callback = cb.into_optional_row_render_cb();
        self
    }

    /// Returns the [DataStore].
    pub fn get_store(&self) -> S {
        self.store.clone()
    }

    /// Sets the multiselect mode
    pub fn set_multiselect_mode(&mut self, multiselect_mode: impl IntoPropValue<MultiSelectMode>) {
        self.multiselect_mode = multiselect_mode.into_prop_value();
    }

    /// Builder style method to set the multiselect mode.
    pub fn multiselect_mode(
        mut self,
        multiselect_mode: impl IntoPropValue<MultiSelectMode>,
    ) -> Self {
        self.set_multiselect_mode(multiselect_mode);
        self
    }
}

#[derive(Default)]
struct VirtualScrollInfo {
    start: usize,
    end: usize,
    height: f64,
    offset: f64,
}

impl VirtualScrollInfo {
    fn visible_rows(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    fn is_visible(&self, row: usize) -> bool {
        (self.start..=self.end).contains(&row)
    }
}

#[derive(Debug)]
struct Cursor {
    pos: usize,
    record_key: Key,
}

#[doc(hidden)]
pub struct PwtDataTable<S: DataStore> {
    unique_id: AttrValue,
    has_focus: bool,
    take_focus: bool,     // focus cursor after render
    active_column: usize, // which colums has focus?
    cursor: Option<Cursor>,
    last_select_position: Option<usize>,
    selection_status: RowSelectionStatus,

    _selection_observer: Option<SelectionObserver>,

    _store_observer: S::Observer,
    _phantom_store: PhantomData<S>,

    headers: Rc<Vec<IndexedHeader<S::Record>>>,

    columns: Rc<Vec<DataTableColumn<S::Record>>>,
    column_widths: Vec<f64>,
    column_hidden: Rc<Vec<bool>>,
    scroll_info: VirtualScrollInfo,

    cell_config: Rc<CellConfiguration>,

    header_scroll_ref: NodeRef,
    scroll_ref: NodeRef,
    scroll_top: usize,
    set_scroll_top: Option<usize>,
    viewport_height: f64,
    viewport_width: f64,
    table_height: f64,

    viewport_size_observer: Option<DomSizeObserver>,

    table_ref: NodeRef,
    table_size_observer: Option<DomSizeObserver>,

    row_height: f64,
    scrollbar_size: Option<f64>,

    keypress_timeout: Option<Timeout>,

    focus_table: bool,
}

// Generate first table row using the width from the column definitions.
fn render_empty_row_with_widths<R>(columns: &[DataTableColumn<R>]) -> Html {
    Container::from_tag("tr")
        .attribute("role", "none")
        .key(Key::from("sizes"))
        // Note: This row should not be visible, so avoid borders
        .style("border-top-width", "0px")
        .style("border-bottom-width", "0px")
        .children(columns.iter().filter_map(|column| {
            if column.hidden {
                None
            } else {
                Some(html! {
                    <td role="none" style={format!("width:{};height:0px;", column.width)}></td>
                })
            }
        }))
        .into()
}

// Generate first table row using the observed header sizes.
fn render_empty_row_with_sizes(widths: &[f64], column_hidden: &[bool], bordered: bool) -> Html {
    let border_width = if bordered { 1.0 } else { 0.0 };
    Container::from_tag("tr")
        .attribute("role", "none")
        .key(Key::from("sizes"))
         // Note: This row should not be visible, so avoid borders
        .style("border-top-width", "0px")
        .style("border-bottom-width", "0px")
        .children(
            widths.iter().enumerate()
                .filter(|(column_num, _)| {
                    match column_hidden.get(*column_num) {
                        Some(true) => false,
                        _ => true,
                    }
                })
                .map(|(_, width)| html!{
                    // Note: we substract the border width (1.0) here
                    <td role="none" style={format!("width:{}px;height:0px;", (width - border_width).max(0.0))}></td>
                })
        )
        .into()
}

impl<S: DataStore> PwtDataTable<S> {
    // avoid slow search by lookup up keys nearby cursor first
    fn filtered_record_pos(&self, props: &DataTable<S>, key: &Key) -> Option<usize> {
        if let Some(Cursor { pos, .. }) = &self.cursor {
            let test_pos = *pos;
            if let Some(record_key) = props.store.lookup_filtered_record_key(test_pos) {
                if &record_key == key {
                    return Some(test_pos);
                }
            }

            let test_pos = pos + 1;
            if let Some(record_key) = props.store.lookup_filtered_record_key(test_pos) {
                if &record_key == key {
                    return Some(test_pos);
                }
            }
            if *pos > 0 {
                let test_pos = pos - 1;
                if let Some(record_key) = props.store.lookup_filtered_record_key(test_pos) {
                    if &record_key == key {
                        return Some(test_pos);
                    }
                }
            }
        }

        props.store.filtered_record_pos(&key)
    }

    fn set_cursor(&mut self, props: &DataTable<S>, pos: Option<usize>) {
        if let Some(pos) = pos {
            self.cursor = match props.store.lookup_filtered_record_key(pos) {
                Some(record_key) => Some(Cursor { pos, record_key }),
                None => None,
            }
        } else {
            self.cursor = None;
        }
    }

    fn cursor_down(&mut self, lines: usize, props: &DataTable<S>) {
        let len = props.store.filtered_data_len();
        if len == 0 {
            self.set_cursor(props, None);
            return;
        }
        self.set_cursor(
            props,
            match &self.cursor {
                Some(Cursor { pos, .. }) => {
                    if (pos + lines) < len {
                        Some(pos + lines)
                    } else {
                        Some(len - 1)
                    }
                }
                None => Some(0),
            },
        );
    }

    fn cursor_up(&mut self, lines: usize, props: &DataTable<S>) {
        let len = props.store.filtered_data_len();
        if len == 0 {
            self.set_cursor(props, None);
            return;
        }
        self.set_cursor(
            props,
            match &self.cursor {
                Some(Cursor { pos, .. }) => {
                    if *pos > lines {
                        Some(pos - lines)
                    } else {
                        Some(0)
                    }
                }
                None => Some(len - 1),
            },
        );
    }

    fn select_position(
        &mut self,
        props: &DataTable<S>,
        selection: &Selection,
        cursor: usize,
        _shift: bool,
        ctrl: bool,
    ) {
        self.last_select_position = Some(cursor);

        if let Some(key) = props.store.lookup_filtered_record_key(cursor) {
            if ctrl {
                selection.toggle(key);
            } else {
                selection.select(key);
            }
        }
    }

    fn select_range(
        &mut self,
        props: &DataTable<S>,
        selection: &Selection,
        last_cursor: Option<usize>,
        new_cursor: Option<usize>,
        shift: bool,
        ctrl: bool,
    ) {
        let new_cursor = match new_cursor {
            Some(new_cursor) => new_cursor,
            None => return,
        };

        if shift || ctrl {
            if let Some(last_cursor) = last_cursor {
                let (start, end) = if last_cursor <= new_cursor {
                    (last_cursor, new_cursor)
                } else {
                    (new_cursor, last_cursor)
                };

                // use write lock to avoid multiple notification
                let mut guard = selection.write();
                for pos in start..=end {
                    if let Some(key) = props.store.lookup_filtered_record_key(pos) {
                        if ctrl {
                            guard.toggle(key);
                        } else {
                            guard.select(key);
                        }
                    }
                }
                self.last_select_position = Some(end);
            } else {
                self.select_position(props, selection, new_cursor, shift, ctrl);
            }
        }
    }

    fn select_all(&mut self, props: &DataTable<S>) {
        let selection = match &props.selection {
            Some(selection) => selection,
            None => {
                self.selection_status = RowSelectionStatus::Nothing;
                return;
            }
        };
        let record_count = props.store.filtered_data_len();
        // use write lock to avoid multiple notification
        let mut selection = selection.write();
        if !selection.is_multiselect() {
            return;
        }

        let mut keys: HashSet<Key> = HashSet::new();
        for pos in 0..record_count {
            if let Some(key) = props.store.lookup_filtered_record_key(pos) {
                keys.insert(key);
            }
        }
        selection.bulk_select(keys);
        self.selection_status = RowSelectionStatus::All;
    }

    fn clear_selection(&mut self, props: &DataTable<S>) {
        if let Some(selection) = &props.selection {
            selection.clear();
        }
    }

    fn update_selection_status(&mut self, props: &DataTable<S>) {
        let selection = match &props.selection {
            Some(selection) => selection,
            None => {
                self.selection_status = RowSelectionStatus::Nothing;
                return;
            }
        };

        let record_count = props.store.filtered_data_len();
        if record_count == 0 {
            self.selection_status = RowSelectionStatus::Nothing;
            return;
        }

        let selection_len = selection.len();
        if record_count == selection_len {
            self.selection_status = RowSelectionStatus::All;
        } else if selection_len > 0 {
            self.selection_status = RowSelectionStatus::Some;
        } else {
            self.selection_status = RowSelectionStatus::Nothing;
        }
    }

    // remove stale keys from selection
    fn cleanup_selection(&mut self, props: &DataTable<S>) {
        if let Some(selection) = &props.selection {
            let mut selection = selection.write();
            if selection.is_multiselect() {
                let mut keys: HashSet<Key> = HashSet::new();
                for (_pos, node) in props.store.filtered_data() {
                    let key = props.store.extract_key(&node.record());
                    if selection.contains(&key) {
                        keys.insert(key);
                    }
                }
                selection.bulk_select(keys);
            } else {
                if let Some(key) = selection.selected_key() {
                    if props.store.filtered_record_pos(&key).is_none() {
                        selection.clear();
                    }
                }
            }
        }
    }

    fn select_cursor(&mut self, props: &DataTable<S>, shift: bool, ctrl: bool) -> bool {
        let selection = match &props.selection {
            Some(selection) => selection,
            None => return false,
        };

        let (cursor, record_key) = match &self.cursor {
            Some(Cursor { pos, record_key }) => (*pos, record_key),
            None => return false, // nothing to do
        };

        self.last_select_position = Some(cursor);

        if !(shift || ctrl) && props.multiselect_mode != MultiSelectMode::Simple {
            selection.clear();
        }

        selection.toggle(record_key.clone());
        true
    }

    fn focus_cursor(&mut self) {
        match &self.cursor {
            Some(Cursor { record_key, .. }) => self.focus_cell(&record_key.clone()),
            None => return, // nothing to do
        };
    }

    fn get_row_el(&self, key: &Key) -> Option<web_sys::Element> {
        let id = self.get_unique_item_id(key);
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        document.get_element_by_id(&id)
    }

    fn focus_cell(&mut self, key: &Key) {
        let row_el = match self.get_row_el(key) {
            Some(el) => el,
            None => {
                // row not rendered, delay after render
                self.take_focus = true;
                return;
            }
        };
        if let Some(cell) = dom_find_cell(&row_el, self.active_column) {
            let _ = cell.focus();
        }
    }

    fn focus_inside_cell(&self, key: &Key) -> bool {
        let row_el = match self.get_row_el(key) {
            Some(el) => el,
            None => return false,
        };
        if let Some(cell) = dom_find_cell(&row_el, self.active_column) {
            return focus_inside_el(cell);
        }
        false
    }

    fn cell_focus_next(&mut self, key: &Key, backwards: bool) {
        let row_el = match self.get_row_el(key) {
            Some(el) => el,
            None => return,
        };
        if let Some(cell) = dom_find_cell(&row_el, self.active_column) {
            focus_next_el(cell, backwards);
        }
    }

    fn find_focused_cell(&self) -> Option<(Key, Option<usize>)> {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let active_el = match document.active_element() {
            Some(el) => el,
            None => return None,
        };
        dom_find_focus_pos(active_el, &self.unique_id)
    }

    fn get_unique_item_id(&self, key: &Key) -> String {
        format!("{}-item-{}", self.unique_id, key)
    }

    fn scroll_cursor_into_view(&mut self) {
        let (cursor, _record_key) = match &self.cursor {
            Some(Cursor { pos, record_key }) => (*pos, record_key),
            None => return, // nothing to do
        };

        if !(self.scroll_info.start..self.scroll_info.end).contains(&cursor) {
            let height = (self.row_height * cursor as f64) - (self.viewport_height / 2.0);
            self.set_scroll_top = Some(height.max(0.0).round() as usize);
        }
    }

    fn render_table(&self, props: &DataTable<S>, offset: f64, start: usize, end: usize) -> Html {
        let virtual_scroll = props.virtual_scroll.unwrap_or(true);
        let fixed_mode = props.show_header || virtual_scroll;

        let first_row = if fixed_mode && !self.column_widths.is_empty() {
            render_empty_row_with_sizes(&self.column_widths, &self.column_hidden, props.bordered)
        } else {
            render_empty_row_with_widths(&self.columns)
        };

        let mut table = Container::new()
            // do not use table tag here to avoid role="table", instead set display type in style"
            .attribute("role", "none")
            .class("pwt-datatable-content")
            .class(props.hover.then(|| "table-hover"))
            .class(props.striped.then(|| "table-striped"))
            .class(props.bordered.then(|| "table-bordered"))
            .class(props.borderless.then(|| "table-borderless"))
            .node_ref(self.table_ref.clone())
            .style("display", "table")
            .style("table-layout", fixed_mode.then(|| "fixed"))
            .style("width", fixed_mode.then(|| "1px")) // required by table-layout fixed
            .style("position", "relative")
            .style("top", format!("{offset}px"))
            .with_child(first_row);

        let mut cursor = self.cursor.as_ref().map(|c| c.pos);

        if let Some(c) = cursor {
            if c < start || c >= end {
                // Cursor row is outside visible region.
                cursor = None;
            }
        }

        for (filtered_pos, item) in props.store.filtered_data_range(start..end) {
            let record_key = props.store.extract_key(&*item.record());

            let mut selected = false;
            if let Some(selection) = &props.selection {
                selected = selection.contains(&record_key);
            }

            let active = cursor
                .map(|cursor| cursor == filtered_pos)
                // if no cursor, mark first row active
                .unwrap_or(filtered_pos == start);

            let row = DataTableRow {
                selection: props.selection.clone(),
                unique_table_id: self.unique_id.clone(),
                record: item.record().clone(),
                record_key,
                row_num: filtered_pos,
                columns: self.columns.clone(),
                column_hidden: self.column_hidden.clone(),
                min_row_height: props.min_row_height,
                vertical_align: props.vertical_align.clone(),
                cell_config: self.cell_config.clone(),
                row_render_callback: props.row_render_callback.clone(),
                selected,
                active_cell: active.then(|| self.active_column),
                has_focus: active && self.has_focus,
                is_expanded: item.expanded(),
                is_leaf: item.is_leaf(),
                level: item.level(),
            };

            table.add_child(row);
        }

        table.into()
    }

    fn render_scroll_content(&self, props: &DataTable<S>) -> Html {
        let table = self.render_table(
            props,
            self.scroll_info.offset,
            self.scroll_info.start,
            self.scroll_info.end,
        );

        let height = self.scroll_info.height;

        Container::new()
            .height(height)
            .attribute("role", "none")
            .with_child(table)
            .into()
    }

    fn rows_per_page(&self, props: &DataTable<S>) -> usize {
        let row_count = props.store.filtered_data_len();
        let virtual_scroll = props
            .virtual_scroll
            .unwrap_or(row_count >= VIRTUAL_SCROLL_TRIGGER);
        if virtual_scroll {
            return self.scroll_info.visible_rows();
        }
        (self.viewport_height / self.row_height).floor() as usize
    }

    fn update_scroll_info(&mut self, props: &DataTable<S>) {
        let row_count = props.store.filtered_data_len();

        let virtual_scroll = props
            .virtual_scroll
            .unwrap_or(row_count >= VIRTUAL_SCROLL_TRIGGER);

        let mut start = if virtual_scroll {
            (self.scroll_top as f64 / self.row_height).floor() as usize
        } else {
            0
        };

        if start > 0 {
            start -= 1;
        }
        if (start & 1) == 1 {
            start -= 1;
        } // make it work with striped rows

        let max_visible_rows =
            (self.viewport_height / props.min_row_height as f64).ceil() as usize + 5;
        let end = if virtual_scroll {
            (start + max_visible_rows).min(row_count)
        } else {
            row_count
        };

        if start > end {
            start = end.saturating_sub(max_visible_rows);
        }

        let offset = (start as f64) * self.row_height;

        let height =
            offset + self.table_height + row_count.saturating_sub(end) as f64 * self.row_height;

        self.scroll_info = VirtualScrollInfo {
            start,
            end,
            offset,
            height,
        };
    }

    fn init_headers(&mut self, props: &DataTable<S>) {
        let headers = create_indexed_header_list(&props.headers);

        // fixme: try to put column info into indexed headers so we have only one place for that
        let mut columns = Vec::new();
        for header in props.headers.iter() {
            header.extract_column_list(&mut columns);
        }
        let mut column_hidden = Vec::new();
        for column in columns.iter() {
            column_hidden.push(column.hidden);
        }
        self.headers = Rc::new(headers);
        self.columns = Rc::new(columns);
        self.column_hidden = Rc::new(column_hidden);
    }
}

impl<S: DataStore + 'static> Component for PwtDataTable<S> {
    type Message = Msg<S::Record>;
    type Properties = DataTable<S>;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let cell_config = if props.cell_configuration.class.is_empty() {
            CellConfiguration::new().class("pwt-datatable-cell")
        } else {
            props.cell_configuration.clone()
        };

        let _store_observer = props
            .store
            .add_listener(ctx.link().callback(|_| Msg::DataChange));

        let _selection_observer = match &props.selection {
            Some(selection) => {
                Some(selection.add_listener(ctx.link().callback(|_| Msg::SelectionChange)))
            }
            None => None,
        };

        let mut me = Self {
            _phantom_store: PhantomData::<S>,
            _store_observer,
            headers: Rc::new(Vec::new()),
            unique_id: AttrValue::from(get_unique_element_id()),
            has_focus: false,
            take_focus: false,
            cursor: None,
            last_select_position: None,
            selection_status: RowSelectionStatus::Nothing,
            _selection_observer,

            active_column: 0,
            columns: Rc::new(Vec::new()),
            column_widths: Vec::new(),
            column_hidden: Rc::new(Vec::new()),
            scroll_info: VirtualScrollInfo::default(),
            cell_config: Rc::new(cell_config),
            scroll_top: 0,
            set_scroll_top: None,
            viewport_height: 0.0,
            viewport_width: 0.0,
            viewport_size_observer: None,
            header_scroll_ref: NodeRef::default(),
            scroll_ref: NodeRef::default(),

            table_ref: NodeRef::default(),
            table_size_observer: None,
            table_height: 0.0,

            row_height: props.min_row_height as f64,
            scrollbar_size: None,
            keypress_timeout: None,

            focus_table: false,
        };
        me.init_headers(props);

        me.update_scroll_info(props);
        // fixme: remove unknown keys from selection
        me.update_selection_status(props);
        me
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::SelectionChange => {
                self.update_selection_status(props);
                true
            }
            Msg::DataChange => {
                // try to keep cursor on the same record
                if let Some(Cursor { record_key, .. }) = &self.cursor {
                    self.cursor = self
                        .filtered_record_pos(props, record_key)
                        .map(|pos| Cursor {
                            pos,
                            record_key: record_key.clone(),
                        });
                }
                self.update_scroll_info(props);

                if self.selection_status == RowSelectionStatus::All {
                    self.select_all(props);
                } else {
                    self.cleanup_selection(props);
                }
                self.update_selection_status(props);

                true
            }
            Msg::ScrollTo(x, y) => {
                self.scroll_top = y.max(0) as usize;
                if let Some(el) = self.header_scroll_ref.cast::<web_sys::Element>() {
                    el.set_scroll_left(x as i32);
                }
                self.update_scroll_info(props);
                props.virtual_scroll.unwrap_or(true)
            }
            Msg::ViewportResize(width, height, scrollbar_size) => {
                self.viewport_height = height.max(0.0);
                self.viewport_width = width.max(0.0);

                if scrollbar_size.abs() < 1.0 {
                    // on certain zoom levels, the scrollbar size calculation is not perfect...
                    self.scrollbar_size = None;
                } else {
                    self.scrollbar_size = Some(scrollbar_size);
                }

                self.update_scroll_info(props);

                true
            }
            Msg::TableResize(_width, height) => {
                let height = height.max(0.0);
                if self.table_height == height {
                    return false;
                };
                self.table_height = height;
                let visible_rows = self.scroll_info.visible_rows();
                if (height > 0.0) && (visible_rows > 0) {
                    let row_height = height / visible_rows as f64;
                    if row_height > self.row_height {
                        self.row_height = row_height;
                    }
                }
                self.update_scroll_info(props);

                if self.cursor.is_none() {
                    if let Some(selection) = &props.selection {
                        if let Some(record_key) = selection.selected_key() {
                            self.cursor =
                                self.filtered_record_pos(props, &record_key)
                                    .map(|pos| Cursor {
                                        pos,
                                        record_key: record_key.clone(),
                                    });
                        }
                    }
                    self.scroll_cursor_into_view();
                }

                true
            }
            // Cursor handling
            Msg::KeyDown(event) => {
                let key: &str = &event.key();
                let shift = event.shift_key();
                let ctrl = event.ctrl_key();

                if let Some(Cursor { record_key, .. }) = &self.cursor {
                    let record_key = record_key.clone();
                    if let Some(callback) = &props.on_row_keydown {
                        let mut event = DataTableKeyboardEvent {
                            record_key: record_key.clone(),
                            inner: event.clone(),
                            selection: props.selection.clone(),
                            stop_propagation: false,
                        };
                        callback.emit(&mut event);
                        if event.stop_propagation {
                            return false;
                        }
                    }

                    if self.focus_inside_cell(&record_key) {
                        let inside_input = focus_inside_input();
                        match key {
                            "F2" | "Escape" => {
                                event.prevent_default();
                                self.focus_cell(&record_key);
                            }
                            "ArrowRight" | "ArrowDown" if !inside_input => {
                                event.prevent_default();
                                self.cell_focus_next(&record_key, false);
                            }
                            "ArrowLeft" | "ArrowUp" if !inside_input => {
                                event.prevent_default();
                                self.cell_focus_next(&record_key, true);
                            }
                            "Tab" if inside_input => {
                                event.prevent_default();
                                self.cell_focus_next(&record_key, !shift);
                            }
                            " " if !inside_input => {
                                // avoid scrollbar default action
                                event.prevent_default();
                            }
                            _ => {}
                        }
                        return false;
                    }

                    if let Some(column) = self.columns.get(self.active_column) {
                        if let Some(on_cell_keydown) = &column.on_cell_keydown {
                            let mut event = DataTableKeyboardEvent {
                                record_key: record_key.clone(),
                                inner: event.clone(),
                                selection: props.selection.clone(),
                                stop_propagation: false,
                            };
                            on_cell_keydown.emit(&mut event);
                            if event.stop_propagation {
                                return false;
                            }
                        }
                    }
                }

                let msg = match key {
                    "PageDown" => {
                        event.prevent_default();
                        let rows = self.rows_per_page(props);
                        Msg::CursorDown(rows, shift, ctrl)
                    }
                    "PageUp" => {
                        event.prevent_default();
                        let rows = self.rows_per_page(props);
                        Msg::CursorUp(rows, shift, ctrl)
                    }
                    "ArrowDown" => {
                        event.prevent_default();
                        Msg::CursorDown(1, shift, ctrl)
                    }
                    "ArrowUp" => {
                        event.prevent_default();
                        Msg::CursorUp(1, shift, ctrl)
                    }
                    "ArrowLeft" => {
                        event.prevent_default();
                        Msg::CursorLeft
                    }
                    "ArrowRight" => {
                        event.prevent_default();
                        Msg::CursorRight
                    }
                    " " => {
                        event.prevent_default();

                        if shift {
                            let cursor = self.cursor.as_ref().map(|c| c.pos);
                            if let Some(selection) = &props.selection {
                                self.select_range(
                                    props,
                                    selection,
                                    self.last_select_position,
                                    cursor,
                                    shift,
                                    false,
                                );
                            }
                        } else {
                            self.select_cursor(props, false, ctrl);
                        }

                        return true;
                    }
                    "End" => {
                        event.prevent_default();
                        self.set_cursor(props, None);
                        self.cursor_up(1, props);
                        self.scroll_cursor_into_view();
                        self.focus_cursor();
                        return true;
                    }
                    "Home" => {
                        // also known as "Pos 1"
                        event.prevent_default();
                        self.set_cursor(props, None);
                        self.cursor_down(1, props);
                        self.scroll_cursor_into_view();
                        self.focus_cursor();
                        return true;
                    }
                    "Enter" => {
                        // also known as "Return"
                        // Return - same behavior as rowdblclick

                        event.prevent_default();

                        self.select_cursor(props, false, false);

                        return false;
                    }
                    "F2" => {
                        event.prevent_default();

                        let record_key = match &self.cursor {
                            Some(Cursor { record_key, .. }) => record_key.clone(),
                            None => return false, // nothing to do
                        };

                        self.cell_focus_next(&record_key, false);

                        return false;
                    }
                    _ => return false,
                };
                let link = ctx.link().clone();
                // delay message to give time to render changes
                self.keypress_timeout = Some(Timeout::new(1, move || {
                    link.send_message(msg);
                }));
                false
            }
            Msg::CursorDown(lines, shift, ctrl) => {
                //if shift { self.select_cursor(props, shift, false); }
                self.cursor_down(lines, props);
                self.scroll_cursor_into_view();
                self.focus_cursor();
                if shift {
                    if let Some(selection) = &props.selection {
                        let cursor = self.cursor.as_ref().map(|c| c.pos);
                        self.select_range(
                            props,
                            selection,
                            self.last_select_position,
                            cursor,
                            shift,
                            false,
                        );
                    }
                }

                if !(shift || ctrl) && props.autoselect {
                    self.select_cursor(props, false, false);
                }

                true
            }
            Msg::CursorUp(lines, shift, ctrl) => {
                self.cursor_up(lines, props);
                self.scroll_cursor_into_view();
                self.focus_cursor();
                if shift {
                    if let Some(selection) = &props.selection {
                        let cursor = self.cursor.as_ref().map(|c| c.pos);
                        self.select_range(
                            props,
                            selection,
                            self.last_select_position,
                            cursor,
                            shift,
                            false,
                        );
                    }
                }

                if !(shift || ctrl) && props.autoselect {
                    self.select_cursor(props, false, false);
                }

                true
            }
            Msg::CursorLeft => {
                let record_key = match &self.cursor {
                    Some(Cursor { record_key, .. }) => record_key.clone(),
                    None => return false,
                };
                let row_el = match self.get_row_el(&record_key) {
                    Some(el) => el,
                    None => return false,
                };

                for i in (0..self.active_column).rev() {
                    if dom_find_cell(&row_el, i).is_some() {
                        self.active_column = i;
                        self.focus_cursor();
                        break;
                    }
                }
                true
            }
            Msg::CursorRight => {
                let record_key = match &self.cursor {
                    Some(Cursor { record_key, .. }) => record_key.clone(),
                    None => return false,
                };
                let row_el = match self.get_row_el(&record_key) {
                    Some(el) => el,
                    None => return false,
                };

                let next = self.active_column + 1;
                for i in next..self.columns.len() {
                    if dom_find_cell(&row_el, i).is_some() {
                        self.active_column = i;
                        self.focus_cursor();
                        break;
                    }
                }
                true
            }
            Msg::ItemClick(record_key, opt_col_num, event, context) => {
                let new_cursor = self.filtered_record_pos(props, &record_key);

                let shift = event.shift_key();
                let ctrl = event.ctrl_key();

                self.set_cursor(props, new_cursor);

                if let Some(col_num) = opt_col_num {
                    if let Some(column) = self.columns.get(col_num) {
                        match (
                            context,
                            &column.on_cell_click,
                            &column.on_cell_context_click,
                        ) {
                            (false, Some(cb), _) | (true, _, Some(cb)) => {
                                let mut event = DataTableMouseEvent {
                                    record_key: record_key.clone(),
                                    inner: event.clone(),
                                    selection: props.selection.clone(),
                                    stop_propagation: false,
                                };
                                cb.emit(&mut event);
                                if event.stop_propagation {
                                    return true;
                                }
                            }
                            _ => {}
                        }
                    }
                }

                match (context, &props.on_row_click, &props.on_row_context_click) {
                    (false, Some(callback), _) | (true, _, Some(callback)) => {
                        let mut event = DataTableMouseEvent {
                            record_key: record_key.clone(),
                            inner: event,
                            selection: props.selection.clone(),
                            stop_propagation: false,
                        };
                        callback.emit(&mut event);
                        if event.stop_propagation {
                            return false;
                        }
                    }
                    _ => {}
                }

                if shift {
                    if let Some(selection) = &props.selection {
                        self.select_range(
                            props,
                            selection,
                            self.last_select_position,
                            new_cursor,
                            shift,
                            false,
                        );
                    }
                } else {
                    self.select_cursor(props, false, ctrl);
                }

                true
            }
            Msg::ItemDblClick(record_key, opt_col_num, event) => {
                if let Some(col_num) = opt_col_num {
                    if let Some(column) = self.columns.get(col_num) {
                        if let Some(on_cell_dblclick) = &column.on_cell_dblclick {
                            let mut event = DataTableMouseEvent {
                                record_key: record_key.clone(),
                                inner: event.clone(),
                                selection: props.selection.clone(),
                                stop_propagation: false,
                            };
                            on_cell_dblclick.emit(&mut event);
                            if event.stop_propagation {
                                return true;
                            }
                        }
                    }
                }

                if let Some(callback) = &props.on_row_dblclick {
                    let mut event = DataTableMouseEvent {
                        record_key: record_key.clone(),
                        inner: event,
                        selection: props.selection.clone(),
                        stop_propagation: false,
                    };
                    callback.emit(&mut event);
                    if event.stop_propagation {
                        return false;
                    }
                }

                let cursor = self.filtered_record_pos(props, &record_key);
                self.set_cursor(props, cursor);
                self.select_cursor(props, false, false);

                true
            }
            Msg::FocusChange(has_focus, has_related_target) => {
                if has_focus {
                    if let Some((row, column)) = self.find_focused_cell() {
                        let cursor = self.filtered_record_pos(props, &row);
                        self.set_cursor(props, cursor);
                        if let Some(selection) = &props.selection {
                            if selection.is_empty() && props.autoselect {
                                self.select_cursor(props, false, false);
                            }
                        }
                        if let Some(column) = column {
                            self.active_column = column;
                        }
                    }
                } else if !has_related_target {
                    // focus table again if the focused/selected cell vanished
                    if let Some(Cursor { ref record_key, .. }) = &self.cursor {
                        match self.filtered_record_pos(props, record_key) {
                            None => self.focus_table = true,
                            Some(pos) if !self.scroll_info.is_visible(pos) => {
                                self.focus_table = true
                            }
                            _ => {}
                        }
                    }
                }
                self.has_focus = has_focus;
                true
            }
            Msg::Header(HeaderMsg::ColumnWidthChange(column_widths)) => {
                self.column_widths = column_widths;
                true
            }
            Msg::Header(HeaderMsg::ChangeSort(sorter_fn)) => {
                // Note: this triggers a Msg::DataChange
                props.store.set_sorter(sorter_fn);
                false
            }
            Msg::Header(HeaderMsg::ColumnHiddenChange(column_hidden)) => {
                self.column_hidden = Rc::new(column_hidden);
                true
            }
            Msg::Header(HeaderMsg::ToggleSelectAll) => {
                if self.selection_status == RowSelectionStatus::All {
                    self.clear_selection(props);
                } else {
                    self.select_all(props);
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let row_count = props.store.filtered_data_len();

        let mut active_descendant = None;
        if let Some(Cursor { record_key, .. }) = &self.cursor {
            active_descendant = Some(self.get_unique_item_id(&record_key));
        }

        let column_widths =
            self.column_widths.iter().sum::<f64>() + self.scrollbar_size.unwrap_or_default();

        let viewport = Container::new()
            .node_ref(self.scroll_ref.clone())
            .key(Key::from("table-viewport"))
            .class("pwt-flex-fill")
            .style(
                "overflow",
                if self.table_height < 1.0 {
                    // if the content cannot be visible, omit the scrollbars
                    "hidden"
                } else if column_widths > self.viewport_width {
                    "auto"
                } else {
                    "hidden auto"
                },
            )
            .style("outline", "0")
            // avoid https://bugzilla.mozilla.org/show_bug.cgi?id=1069739
            .attribute("tabindex", "-1")
            .attribute("role", "rowgroup")
            .attribute("aria-label", "table body")
            .with_child(self.render_scroll_content(props))
            .onfocusin(ctx.link().callback(|_| Msg::FocusChange(true, false)))
            .onfocusout(ctx.link().callback(|event: FocusEvent| {
                Msg::FocusChange(false, event.related_target().is_some())
            }))
            .onscroll(ctx.link().batch_callback(move |event: Event| {
                let target: Option<web_sys::HtmlElement> = event.target_dyn_into();
                target.map(|el| Msg::ScrollTo(el.scroll_left(), el.scroll_top()))
            }))
            .onkeydown({
                let link = ctx.link().clone();
                move |event: KeyboardEvent| {
                    link.send_message(Msg::KeyDown(event));
                }
            })
            .onclick({
                let link = ctx.link().clone();
                let unique_id = self.unique_id.clone();
                move |event: MouseEvent| {
                    if let Some((row_num, col_num)) = dom_find_record_num(&event, &unique_id) {
                        link.send_message(Msg::ItemClick(row_num, col_num, event, false));
                    }
                }
            })
            .ondblclick({
                let link = ctx.link().clone();
                let unique_id = self.unique_id.clone();
                move |event: MouseEvent| {
                    if let Some((row_num, col_num)) = dom_find_record_num(&event, &unique_id) {
                        link.send_message(Msg::ItemDblClick(row_num, col_num, event));
                    }
                }
            })
            .oncontextmenu({
                let link = ctx.link().clone();
                let unique_id = self.unique_id.clone();
                move |event: MouseEvent| {
                    if let Some((row_num, col_num)) = dom_find_record_num(&event, &unique_id) {
                        link.send_message(Msg::ItemClick(row_num, col_num, event, true));
                    }
                }
            });

        let multiselect = props
            .selection
            .as_ref()
            .map(|s| s.is_multiselect())
            .unwrap_or(false);

        let mut header_class = props.header_class.clone();
        if header_class.is_empty() {
            header_class.push("pwt-datatable-header-cell");
        }

        Column::new()
            .class("pwt-datatable")
            .class(props.class.clone())
            .styles(props.styles.clone())
            .attribute("role", "grid")
            .attribute("aria-activedescendant", active_descendant)
            .attribute("aria-rowcount", row_count.to_string())
            .attribute("aria-colcount", (self.columns.len()).to_string())
            .attribute(
                "aria-multiselectable",
                if multiselect { "true" } else { "false" },
            )
            .with_child(
                Container::new() // scollable for header
                    .key(Key::from("table-header"))
                    .node_ref(self.header_scroll_ref.clone())
                    .attribute("role", "rowgroup")
                    .attribute("aria-label", "table header")
                    .style("flex", "0 0 auto")
                    .height(if props.show_header {
                        CssLength::None
                    } else {
                        CssLength::Px(0.0)
                    })
                    .class("pwt-overflow-hidden")
                    .class("pwt-datatable-header")
                    .class((!props.show_header).then_some("pwt-datatable-header-hidden"))
                    .with_child(
                        HeaderWidget::new(self.headers.clone(), ctx.link().callback(Msg::Header))
                            .focusable(props.header_focusable && props.show_header)
                            .selection_status(self.selection_status)
                            .header_class(header_class)
                            .reserve_scroll_space(self.scrollbar_size.unwrap_or_default()),
                    ),
            )
            .with_child(viewport)
            .into()
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();

        if props.store != old_props.store {
            // store changed
            self.update_scroll_info(props);
        }

        if props.headers != old_props.headers {
            self.init_headers(props);
        }

        true
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            if let Some(el) = self.scroll_ref.cast::<web_sys::Element>() {
                let link = ctx.link().clone();
                let size_observer =
                    DomSizeObserver::new(&el, move |(width, height, client_width, _)| {
                        link.send_message(Msg::ViewportResize(width, height, width - client_width));
                    });
                self.viewport_size_observer = Some(size_observer);
            }

            if let Some(el) = self.table_ref.cast::<web_sys::HtmlElement>() {
                let link = ctx.link().clone();
                let size_observer = DomSizeObserver::new(&el, move |(width, height)| {
                    link.send_message(Msg::TableResize(width, height));
                });
                self.table_size_observer = Some(size_observer);
            }
        }
        if let Some(top) = self.set_scroll_top.take() {
            // Note: we delay setting ScrollTop until we rendered the
            // viewport with correct height. Else, set_scroll_top can
            // fail because the viewport is smaller.
            if let Some(el) = self.scroll_ref.cast::<web_sys::Element>() {
                el.set_scroll_top(top as i32);
            }
        }

        if self.take_focus {
            // required when we do big jumps (to end, to start),
            // because previous cursor is not rendered (virtual
            // scroll) and looses focus.
            self.take_focus = false;
            self.focus_cursor();
        }

        if self.focus_table {
            self.focus_table = false;
            if let Some(el) = self.scroll_ref.clone().into_html_element() {
                let _ = el.focus();
            }
        }
    }
}

impl<S: DataStore + 'static> Into<VNode> for DataTable<S> {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtDataTable<S>>(Rc::new(self), key);
        VNode::from(comp)
    }
}

fn dom_find_cell(row_el: &web_sys::Element, column_num: usize) -> Option<web_sys::HtmlElement> {
    let children = row_el.children();
    for i in 0..children.length() {
        let child: web_sys::HtmlElement = children.item(i).unwrap().dyn_into().unwrap();
        if let Some(column_num_str) = child.get_attribute("data-column-num") {
            if let Ok(n) = column_num_str.parse::<usize>() {
                if n == column_num {
                    return Some(child);
                }
            }
        }
    }
    None
}

fn dom_find_focus_pos(el: web_sys::Element, unique_id: &str) -> Option<(Key, Option<usize>)> {
    let unique_row_prefix = format!("{}-item-", unique_id);
    let mut column_num: Option<usize> = None;

    let focused_el: web_sys::Node = el.clone().dyn_into().unwrap();
    let mut cur_el: Option<web_sys::Element> = Some(el);

    loop {
        match cur_el {
            Some(el) => {
                if el.tag_name() == "TR" {
                    if let Some(key_str) = el.id().strip_prefix(&unique_row_prefix) {
                        if key_str.len() == 0 {
                            break;
                        } // stop on errors
                          // try to find out the column_num
                        let children = el.children();
                        for i in 0..children.length() {
                            let child: web_sys::HtmlElement =
                                children.item(i).unwrap().dyn_into().unwrap();

                            if child.contains(Some(&focused_el)) {
                                if let Some(column_num_str) = child.get_attribute("data-column-num")
                                {
                                    if let Ok(n) = column_num_str.parse() {
                                        column_num = Some(n);
                                    }
                                }
                            }
                        }
                        return Some((Key::from(key_str), column_num));
                    }
                }
                cur_el = el.parent_element().map(|el| el.dyn_into().unwrap());
            }
            None => break,
        }
    }
    None
}

// Find the [DataTable] record associated with a [MouseEvent].
fn dom_find_record_num(event: &MouseEvent, unique_id: &str) -> Option<(Key, Option<usize>)> {
    let unique_row_prefix = format!("{}-item-", unique_id);
    let mut column_num: Option<usize> = None;

    let mut cur_el: Option<web_sys::HtmlElement> = event.target_dyn_into();

    let click_x = event.client_x() as f64;

    loop {
        match cur_el {
            Some(el) => {
                if el.tag_name() == "TR" {
                    if let Some(n_str) = el.id().strip_prefix(&unique_row_prefix) {
                        // try to find out the column_num
                        let children = el.children();
                        for i in 0..children.length() {
                            let child: web_sys::HtmlElement =
                                children.item(i).unwrap().dyn_into().unwrap();
                            let rect = child.get_bounding_client_rect();

                            if rect.x() < click_x && click_x < (rect.x() + rect.width()) {
                                if let Some(column_num_str) = child.get_attribute("data-column-num")
                                {
                                    if let Ok(n) = column_num_str.parse() {
                                        column_num = Some(n);
                                    }
                                }
                            }
                        }
                        return Some((Key::from(n_str), column_num));
                    }
                }
                cur_el = el.parent_element().map(|el| el.dyn_into().unwrap());
            }
            None => break,
        }
    }
    None
}
