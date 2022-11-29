use std::rc::Rc;
use std::marker::PhantomData;

use derivative::Derivative;
use indexmap::IndexMap;

use gloo_timers::callback::Timeout;
use wasm_bindgen::JsCast;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::{IntoEventCallback, IntoPropValue};

use crate::prelude::*;
use crate::props::SorterFn;
use crate::state::{DataStore, DataNode, Selection2, Store};
use crate::widget::{get_unique_element_id, Container, Column, SizeObserver};

use super::{
    create_indexed_header_list,
    DataTableColumn, HeaderWidget, DataTableMouseEvent, DataTableKeyboardEvent,
    DataTableHeader, DataTableCellRenderArgs, IndexedHeader,
    DataTableRowRenderArgs, DataTableRowRenderCallback, IntoOptionalDataTableRowRenderCallback,
};

pub enum Msg<T: 'static> {
    DataChange,
    ChangeSort(SorterFn<T>),
    ColumnWidthChange(Vec<f64>),
    ColumnHiddenChange(Vec<bool>),
    ScrollTo(i32, i32),
    ViewportResize(f64, f64),
    ContainerResize(f64, f64),
    TableResize(f64, f64),
    KeyDown(KeyboardEvent),
    CursorDown(usize, bool, bool),
    CursorUp(usize, bool, bool),
    CursorLeft,
    CursorRight,
    ItemClick(Key, Option<usize>, MouseEvent),
    ItemDblClick(Key, MouseEvent),
    FocusChange(bool),
}

/// DataTable properties
///
/// Features:
///
/// - Virtual scrolling.
/// - Trees and Lists.
/// - Selection/Cursor management
/// - Nested Header definitions.
/// - Header menus (hide, sort, ...)

#[derive(Properties)]
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct DataTable<T: 'static, S: DataStore<T> = Store<T>> {

    #[prop_or_default]
    node_ref: NodeRef,
    pub key: Option<Key>,

    #[prop_or_default]
    pub class: Classes,

    headers: Rc<Vec<DataTableHeader<T>>>,

    // The data collection ([Store] or [TreeStore](crate::state::TreeStore)).
    store: S,

    /// Set class for table cells (default is "pwt-p-2").
    #[prop_or_default]
    pub cell_class: Classes,

    /// Set class for header cells (default is "pwt-p-2").
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
    pub vertical_align: Option<AttrValue>,

    /// Virtual Scroll
    ///
    /// Virtual scroll is enabled by default for tables with more than 30 rows.
    pub virtual_scroll: Option<bool>,

    /// Minimum row height (default 22)
    ///
    /// Sets the minmum height for table rows. This is also used by
    /// the virtual scrolling algorithm to compute the maximal number
    /// of visible rows.
    #[prop_or(22)]
    pub min_row_height: usize,

    /// Selection object.
    pub selection: Option<Selection2>,

    /// Automatically select the focused row.
    #[prop_or(true)]
    pub select_on_focus: bool,

    /// Row click callback
    pub on_row_click: Option<Callback<DataTableMouseEvent>>,

    /// Row double click callback
    pub on_row_dblclick: Option<Callback<DataTableMouseEvent>>,

    /// Row keydown callback
    pub on_row_keydown: Option<Callback<DataTableKeyboardEvent>>,

    pub row_render_callback: Option<DataTableRowRenderCallback<T>>,
}

static VIRTUAL_SCROLL_TRIGGER: usize = 30;

impl <T: 'static, S: DataStore<T> + 'static> DataTable<T, S> {

    /// Create a new instance.
    ///
    /// The store is either a [Store] or [TreeStore](crate::state::TreeStore).
    pub fn new(headers: Rc<Vec<DataTableHeader<T>>>, store: S) -> Self {
        yew::props!(DataTable<T, S> { headers, store })
    }

    /// Builder style method to set the yew `node_ref`.
    pub fn node_ref(mut self, node_ref: ::yew::html::NodeRef) -> Self {
        self.node_ref = node_ref;
        self
    }

    /// Builder style method to set the yew `key` property.
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = Some(key.into());
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

    /// Builder style method to add a html class for table cells.
    pub fn cell_class(mut self, class: impl Into<Classes>) -> Self {
        self.add_cell_class(class);
        self
    }

    /// Method to add a html class for table cells.
    pub fn add_cell_class(&mut self, class: impl Into<Classes>) {
        self.cell_class.push(class);
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

    /// Builder style method to set the selection model.
    pub fn selection(mut self, selection: impl IntoPropValue<Option<Selection2>>) -> Self {
        self.selection = selection.into_prop_value();
        self
    }

    /// Builder style method to set the row click callback.
    pub fn on_row_click(mut self, cb: impl IntoEventCallback<DataTableMouseEvent>) -> Self {
        self.on_row_click = cb.into_event_callback();
        self
    }

    /// Builder style method to set the row double click callback.
    pub fn on_row_dblclick(mut self, cb: impl IntoEventCallback<DataTableMouseEvent>) -> Self {
        self.on_row_dblclick = cb.into_event_callback();
        self
    }

    /// Builder style method to set the row keydown callback.
    pub fn on_row_keydown(mut self, cb: impl IntoEventCallback<DataTableKeyboardEvent>) -> Self {
        self.on_row_keydown = cb.into_event_callback();
        self
    }

    /// Builder style method to set the row render callback.
    pub fn row_render_callback(mut self, cb: impl IntoOptionalDataTableRowRenderCallback<T>) -> Self {
        self.row_render_callback = cb.into_optional_row_render_cb();
        self
    }

}

#[derive(Default)]
struct VirtualScrollInfo {
    start: usize,
    end: usize,
    height: usize,
    offset: usize,
}

impl VirtualScrollInfo {
    fn visible_rows(&self) -> usize {
        self.end.saturating_sub(self.start)
    }
}

#[derive(Debug)]
struct Cursor {
    pos: usize,
    record_key: Key,
}

#[doc(hidden)]
pub struct PwtDataTable<T: 'static, S: DataStore<T>> {
    unique_id: String,
    has_focus: bool,
    take_focus: bool, // focus cursor after render
    active_column: usize, // which colums has focus?
    cursor: Option<Cursor>,
    last_select_position: Option<usize>,

    _store_observer: S::Observer,
    _phantom_store: PhantomData<S>,

    headers: Rc<Vec<IndexedHeader<T>>>,

    columns: Vec<DataTableColumn<T>>,
    column_widths: Vec<f64>,
    column_hidden: Vec<bool>,
    scroll_info: VirtualScrollInfo,

    cell_class: Classes,

    header_scroll_ref: NodeRef,
    scroll_ref: NodeRef,
    scroll_top: usize,
    viewport_height: usize,
    table_height: usize,

    viewport_size_observer: Option<SizeObserver>,

    table_ref: NodeRef,
    table_size_observer: Option<SizeObserver>,

    row_height: usize,

    container_ref: NodeRef,
    container_size_observer: Option<SizeObserver>,
    container_width: usize,

    keypress_timeout: Option<Timeout>,
    recover_timeout: Option<Timeout>, // recover cursor after scrollTo
}

fn render_empty_row_with_sizes(widths: &[f64], column_hidden: &[bool], bordered: bool) -> Html {
    let border_width = if bordered { 1.0 } else { 0.0 };
    Container::new()
        .tag("tr")
        .attribute("role", "none")
        .key(Key::from("sizes"))
         // Note: This row should not be visible, so avoid borders
        .attribute("style", "border-top-width: 0px; border-bottom-width: 0px;")
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
                    <td role="none" style={format!("width:{:.3}px;height:0px;", (width - border_width).max(0.0))}></td>
                })
        )
        .into()
}

impl<T: 'static, S: DataStore<T>> PwtDataTable<T, S> {

    // avoid slow search by lookup up keys nearby cursor first
    fn filtered_record_pos(
        &self,
        props: &DataTable<T, S>,
        key: &Key,
    ) -> Option<usize> {
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

    fn set_cursor(
        &mut self,
        props: &DataTable<T, S>,
        pos: Option<usize>,
    ) {
        if let Some(pos) = pos {
            self.cursor = match props.store.lookup_filtered_record_key(pos) {
                Some(record_key) => Some(Cursor { pos, record_key }),
                None => None,
            }
        } else {
            self.cursor = None;
        }
    }

    fn cursor_down(
        &mut self,
        lines: usize,
        props: &DataTable<T, S>,
    ) {
        let len = props.store.filtered_data_len();
        if len == 0 {
            self.set_cursor(props, None);
            return;
        }
        self.set_cursor(props, match &self.cursor {
            Some(Cursor { pos, ..}) => if (pos + lines) < len { Some(pos + lines) }  else { Some(len - 1) },
            None => Some(0),
        });
    }

    fn cursor_up(
        &mut self,
        lines: usize,
        props: &DataTable<T, S>,
    ) {
        let len = props.store.filtered_data_len();
        if len == 0 {
            self.set_cursor(props, None);
            return;
        }
        self.set_cursor(props, match &self.cursor {
            Some(Cursor { pos, ..}) => if *pos > lines { Some(pos - lines) } else { Some(0) },
            None => Some(len - 1),
        });
    }

    fn select_position(
        &mut self,
        props: &DataTable<T, S>,
        selection: &Selection2,
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
        props: &DataTable<T, S>,
        selection: &Selection2,
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
                for pos in start..=end {
                    self.select_position(props, selection, pos, shift, ctrl);
                }
            } else {
                self.select_position(props, selection, new_cursor, shift, ctrl);
            }
        }
    }

    fn select_cursor(&mut self, props: &DataTable<T, S>, shift: bool, ctrl: bool) -> bool {
        let selection = match &props.selection {
            Some(selection) => selection,
            None => return false,
        };

        let (cursor, record_key) = match &self.cursor {
            Some(Cursor { pos, record_key}) => (*pos, record_key),
            None => return false, // nothing to do
        };

        self.last_select_position = Some(cursor);

        if !(shift || ctrl) { selection.clear(); }

        if ctrl {
            selection.toggle(record_key.clone());
        } else {
            selection.select(record_key.clone());
        }
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
            },
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
            return crate::widget::focus::focus_inside_el(cell);
        }
        false
    }

    fn cell_focus_next(&mut self, key: &Key, backwards: bool) {
        let row_el = match self.get_row_el(key) {
            Some(el) => el,
            None => return,
        };
        if let Some(cell) = dom_find_cell(&row_el, self.active_column) {
            crate::widget::focus::focus_next_tabable_el(cell, backwards, false);
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

    fn scroll_cursor_into_view(&self) {
        let (cursor, _record_key) = match &self.cursor {
            Some(Cursor { pos, record_key}) => (*pos, record_key),
            None => return, // nothing to do
        };

         if !(self.scroll_info.start..self.scroll_info.end).contains(&cursor) {
            let height =  (self.row_height * cursor).saturating_sub(self.viewport_height/2);
            if let Some(el) = self.scroll_ref.cast::<web_sys::Element>() {
                el.set_scroll_top(height as i32);
            }
        }
    }

    fn render_row(&self, props: &DataTable<T, S>, item: &dyn DataNode<T>, record_key: Key, row_num: usize, selected: bool, active: bool) -> Html {

        let item_id = self.get_unique_item_id(&record_key);

        // Make sure our rows have a minimum height
        // Note: setting min-height on <tr> or <td> does not work
        let minheight_cell_style = AttrValue::Rc(
            format!("vertical-align:top;height: {}px;", props.min_row_height).into()
        );

        let aria_expanded = if item.is_leaf() {
            None
        } else {
            if item.expanded() { Some("true") } else { Some("false") }
        };

        let mut row = Container::new()
            .tag("tr")
            .key(record_key)
            .attribute("role", "row")
            .attribute("aria-rowindex", (row_num + 1).to_string()) // does not work, no firefox support?
            .attribute("aria-expanded", aria_expanded)
            .attribute("id", item_id)
            .class((active && self.has_focus).then(|| "row-cursor"))
            .class(selected.then(|| "selected"));

        if let Some(row_render_callback) = &props.row_render_callback {
            let mut args = DataTableRowRenderArgs {
                node: item,
                row_index: row_num,
                selected,
                class: Classes::new(),
                attributes: IndexMap::new(),
            };

            row_render_callback.apply(&mut args);

            if !args.class.is_empty() {
                row.add_class(args.class);
            }

            for (attr_name, attr_value) in args.attributes.into_iter() {
                row.set_attribute(attr_name, attr_value);
            }
        }

        let mut col_index = 0;
        let mut column_num = 0;

        loop {
            let column = match self.columns.get(column_num) {
                Some(column) => column,
                None => break,
            };

            if let Some(true) = self.column_hidden.get(column_num) {
                column_num += 1;
                continue;
            }

            let mut item_style = format!(
                "vertical-align: {}; text-align: {};",
                props.vertical_align.as_deref().unwrap_or("baseline"),
                column.justify,
            );
            let cell_active = active && self.active_column == column_num;

            let mut args = DataTableCellRenderArgs {
                node: item,
                row_index: row_num,
                column_index: col_index,
                selected,
                class: self.cell_class.clone(),
                attributes: IndexMap::new(),
            };

            let cell = column.render_cell.apply(&mut args);

            if let Some(style) = args.attributes.remove("style") {
                item_style.push_str(&style);
            }

            let mut td = Container::new()
                .tag("td")
                .class(args.class)
                .class((cell_active && self.has_focus).then(|| "cell-cursor"))
                .attribute("style", item_style)
                .attribute("role", "gridcell")
                .attribute("data-column-num", column_num.to_string())
                .attribute("tabindex", if cell_active { "0" } else { "-1" })
                .with_child(html!{<div role="none">{cell}</div>});

            let mut colspan = 1;

            if let Some(colspan_str) = args.attributes.get("colspan") {
                if let Ok(n) = (*colspan_str).parse::<usize>() {
                    if n > 0 { colspan = n }
                }
            }

            for (attr_name, attr_value) in args.attributes.into_iter() {
                td.set_attribute(attr_name, attr_value);
            }


            col_index += colspan;
            column_num += colspan;

            row.add_child(td);
        }

        row.add_child(html!{<td aria-hidden="true" style={minheight_cell_style.clone()}/>});
        row.into()
    }

    fn render_table(&self, props: &DataTable<T, S>, offset: usize, start: usize, end: usize) -> Html {

        let mut table = Container::new()
        // do not use table tag here to avoid role="table", instead set "pwt-d-table"
            .attribute("role", "none")
            .class("pwt-d-table")
            .class("pwt-datatable2-content")
            .class(props.hover.then(|| "table-hover"))
            .class(props.striped.then(|| "table-striped"))
            .class(props.bordered.then(|| "table-bordered"))
            .class(props.borderless.then(|| "table-borderless"))
            .node_ref(self.table_ref.clone())
            .attribute("style", format!("table-layout: fixed;width:1px; position:relative;top:{}px;", offset))
            .with_child(render_empty_row_with_sizes(&self.column_widths, &self.column_hidden, props.bordered));

        let mut cursor = self.cursor.as_ref().map(|c| c.pos);

        if let Some(c) = cursor {
            if c < start || c >= end {
                // Cursor row is outside visible region.
                cursor = None;
            }
        }
        if !self.column_widths.is_empty() {
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

                let row = self.render_row(props, item.as_ref(), record_key, filtered_pos, selected, active);
                table.add_child(row);
            }
        }

        table.into()
    }

    fn render_scroll_content(
        &self,
        props: &DataTable<T, S>,
    ) -> Html {

        let table = self.render_table(props, self.scroll_info.offset, self.scroll_info.start, self.scroll_info.end);

        let height = self.scroll_info.height;

        // firefox scrollbar ignores height, so we need ad some
        // content at the end.
        let end_marker = Container::new()
            .tag("img")
            .attribute("role", "none")
            .attribute("style", format!(
                "height: 0px; width: 0px; overflow: hidden; position:relative;top:{}px;",
                height
            ));

        let height = height + 15; // add some space at the end
        Container::new()
            .attribute("style", format!("height:{}px", height))
            .attribute("role", "none")
            .with_child(table)
            .with_child(end_marker)
            .into()
    }

    fn update_scroll_info(
        &mut self,
        props: &DataTable<T, S>,
    ) {
        let row_count = props.store.filtered_data_len();

        let virtual_scroll = props.virtual_scroll.unwrap_or(row_count >= VIRTUAL_SCROLL_TRIGGER);

        let mut start = if virtual_scroll {
            self.scroll_top / self.row_height
        } else {
            0
        };

        if start > 0 { start -= 1; }
        if (start & 1) == 1 { start -= 1; } // make it work with striped rows

        let max_visible_rows = (self.viewport_height / props.min_row_height) + 5;
        let end = if virtual_scroll {
            (start + max_visible_rows).min(row_count)
        } else {
            row_count
        };

        let offset = start * self.row_height;

        let height = offset + self.table_height + row_count.saturating_sub(end) * self.row_height;

        self.scroll_info = VirtualScrollInfo { start, end, offset, height };
    }
}

impl <T: 'static, S: DataStore<T> + 'static> Component for PwtDataTable<T, S> {

    type Message = Msg<T>;
    type Properties = DataTable<T, S>;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let headers = create_indexed_header_list(&props.headers);

        // fixme: remove
        let mut columns = Vec::new();
        for header in props.headers.iter() {
            header.extract_column_list(&mut columns);
        }

        let cell_class = if props.cell_class.is_empty() {
            Classes::from("pwt-p-2")
        } else {
            props.cell_class.clone()
        };

        let _store_observer = props.store.add_listener(ctx.link().callback(|_| Msg::DataChange));

        let mut me = Self {
            _phantom_store: PhantomData::<S>,
            _store_observer,
            headers: Rc::new(headers),
            unique_id: get_unique_element_id(),
            has_focus: false,
            take_focus: false,
            cursor: None,
            last_select_position: None,

            active_column: 0,
            columns,
            column_widths: Vec::new(),
            column_hidden: Vec::new(),
            scroll_info: VirtualScrollInfo::default(),
            cell_class,
            scroll_top: 0,
            viewport_height: 0,
            viewport_size_observer: None,
            header_scroll_ref: NodeRef::default(),
            scroll_ref: NodeRef::default(),

            table_ref: NodeRef::default(),
            table_size_observer: None,
            table_height: 0,

            container_ref: NodeRef::default(),
            container_size_observer: None,
            container_width: 0,

            row_height: props.min_row_height,
            keypress_timeout: None,
            recover_timeout: None,
        };

        me.update_scroll_info(props);
        me
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::DataChange => {
                // try to keep cursor on the same record
                if let Some(Cursor { record_key, .. }) = &self.cursor {
                    self.cursor = self.filtered_record_pos(props, record_key)
                        .map(|pos| Cursor { pos, record_key: record_key.clone() });

                }
                self.update_scroll_info(props);

                self.scroll_cursor_into_view();

                true
            }
            Msg::ColumnWidthChange(column_widths) => {
                self.column_widths = column_widths;
                true
            }
            Msg::ScrollTo(x, y) => {
                self.scroll_top = y.max(0) as usize;
                if let Some(el) = self.header_scroll_ref.cast::<web_sys::Element>() {
                    el.set_scroll_left(x as i32);
                }
                self.update_scroll_info(props);
                true
            }
            Msg::ViewportResize(_width, height) => {
                self.viewport_height = height.max(0.0) as usize;
                self.update_scroll_info(props);
                true
            }
            Msg::ContainerResize(width, _height) => {
                self.container_width = width.max(0.0) as usize;
                true
            }
            Msg::TableResize(_width, height) => {
                let height = height.max(0.0) as usize;
                if self.table_height == height { return false; };
                self.table_height = height;
                let visible_rows = self.scroll_info.visible_rows();
                if (height > 0) && (visible_rows > 0) {
                    let row_height = (height as usize) / visible_rows;
                    if row_height > self.row_height {
                        self.row_height = row_height;
                    }
                }
                self.update_scroll_info(props);
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
                        let event = DataTableKeyboardEvent::new(record_key.clone(), event.clone());
                        callback.emit(event);
                    }

                    if self.focus_inside_cell(&record_key) {
                        match key {
                            "F2" | "Escape" => {
                                event.prevent_default();
                                self.focus_cell(&record_key);
                            }
                            "ArrowRight" | "ArrowDown" => {
                                event.prevent_default();
                                self.cell_focus_next(&record_key, false);
                            }
                            "ArrowLeft" | "ArrowUp" => {
                                event.prevent_default();
                                self.cell_focus_next(&record_key, true);
                            }
                            _ => {}
                        }

                        return false;
                    }
                }

                let msg = match key {
                    "PageDown" => {
                        event.prevent_default();
                        let visible_rows = self.scroll_info.visible_rows();
                        Msg::CursorDown(visible_rows, shift, ctrl)
                    }
                    "PageUp" => {
                        event.prevent_default();
                        let visible_rows = self.scroll_info.visible_rows();
                        Msg::CursorUp(visible_rows, shift, ctrl)
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
                                self.select_range(props, selection, self.last_select_position, cursor, shift, false);
                            }
                        } else {
                            if props.select_on_focus {
                                self.select_cursor(props, false, ctrl);
                            }
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
                    "Home" => { // also known as "Pos 1"
                        event.prevent_default();
                        self.set_cursor(props, None);
                        self.cursor_down(1, props);
                        self.scroll_cursor_into_view();
                        self.focus_cursor();
                        return true;
                    }
                    "Enter" => { // also known as "Return"
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
                if shift { self.select_cursor(props, shift, false); }
                self.cursor_down(lines, props);
                self.scroll_cursor_into_view();
                self.focus_cursor();
                if shift { self.select_cursor(props, shift, false); }

                if !(shift || ctrl) && props.select_on_focus {
                    self.select_cursor(props, false, false);
                }

                true
            }
            Msg::CursorUp(lines, shift, ctrl) => {
                if shift { self.select_cursor(props, shift, false); }
                self.cursor_up(lines, props);
                self.scroll_cursor_into_view();
                self.focus_cursor();
                if shift { self.select_cursor(props, shift, false); }

                if !(shift ||ctrl) && props.select_on_focus {
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
            Msg::ItemClick(record_key, opt_col_num, event) => {
                let new_cursor = self.filtered_record_pos(props, &record_key);

                let shift = event.shift_key();
                let ctrl = event.ctrl_key();

                self.set_cursor(props, new_cursor);

                if shift {
                    if let Some(selection) = &props.selection {
                        self.select_range(props, selection, self.last_select_position, new_cursor, shift, false);
                    }
                } else {
                    if props.select_on_focus {
                        self.select_cursor(props, false, ctrl);
                    }
                }

                if let Some(col_num) = opt_col_num {
                    if let Some(column) = self.columns.get(col_num)  {
                        if let Some(on_cell_click) = &column.on_cell_click {
                            let event = DataTableMouseEvent::new(record_key.clone(), event.clone());
                            on_cell_click.emit(event);
                        }
                    }
                }

                if let Some(callback) = &props.on_row_click {
                    let event = DataTableMouseEvent::new(record_key.clone(), event);
                    callback.emit(event);
                }

                true
            }
            Msg::ItemDblClick(record_key, event) => {
                let cursor = self.filtered_record_pos(props, &record_key);
                self.set_cursor(props, cursor);
                self.select_cursor(props, false, false);

                if let Some(callback) = &props.on_row_dblclick {
                    let event = DataTableMouseEvent::new(record_key.clone(), event);
                    callback.emit(event);
                }

                true
            }
            Msg::FocusChange(has_focus) => {
                if has_focus {
                    if let Some((row, column)) = self.find_focused_cell() {
                        let cursor = self.filtered_record_pos(props, &row);
                        self.set_cursor(props, cursor);
                        if let Some(selection) = &props.selection {
                            if selection.is_empty() {
                                self.select_cursor(props, false, false);
                            }
                        }
                        if let Some(column) = column {
                            self.active_column = column;
                        }
                    }

                }
                self.has_focus = has_focus;
                true
            }
            Msg::ChangeSort(sorter_fn) => {
                // Note: this triggers a Msg::DataChange
                props.store.set_sorter(sorter_fn);
                false
            }
            Msg::ColumnHiddenChange(column_hidden) => {
                self.column_hidden = column_hidden;
                true
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

        let viewport = Container::new()
            .node_ref(self.scroll_ref.clone())
            .class("pwt-flex-fill")
            .attribute("style", "overflow: auto; outline: 0")
            // avoid https://bugzilla.mozilla.org/show_bug.cgi?id=1069739
            .attribute("tabindex", "-1")
            .attribute("role", "rowgroup")
            .attribute("aria-label", "table body")
            .with_child(self.render_scroll_content(props))
            .onfocusin(ctx.link().callback(|_| Msg::FocusChange(true)))
            .onfocusout(ctx.link().callback(|_| Msg::FocusChange(false)))
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
                        link.send_message(Msg::ItemClick(
                            row_num,
                            col_num,
                            event,
                          ));
                    }
                }
            })
            .ondblclick({
                let link = ctx.link().clone();
                let unique_id = self.unique_id.clone();
                move |event: MouseEvent| {
                    if let Some((row_num, _col_num)) = dom_find_record_num(&event, &unique_id) {
                        link.send_message(Msg::ItemDblClick(row_num, event));
                    }
                }
            });

        Column::new()
            .class(props.class.clone())
            .node_ref(self.container_ref.clone())
            .attribute("role", "grid")
            .attribute("aria-activedescendant", active_descendant)
            .attribute("aria-rowcount", row_count.to_string())
            .attribute("aria-colcount", (self.columns.len()).to_string())
            .with_child(
                Container::new() // scollable for header
                    .node_ref(self.header_scroll_ref.clone())
                    .attribute("role", "rowgroup")
                    .attribute("aria-label", "table header")
                    .attribute("style", "flex: 0 0 auto")
                    .class("pwt-overflow-hidden")
                    .class("pwt-datatable2-header")
                    .with_child(
                        HeaderWidget::new(self.headers.clone())
                            .header_class(props.header_class.clone())
                            .on_size_change(ctx.link().callback(Msg::ColumnWidthChange))
                            .on_hidden_change(ctx.link().callback(Msg::ColumnHiddenChange))
                            .on_sort_change(ctx.link().callback(Msg::ChangeSort))
                    )
            )
            .with_child(viewport)
            .into()
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();

        if props.store != old_props.store { // store changed
            self.update_scroll_info(props);
        }

        true
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            if let Some(el) = self.scroll_ref.cast::<web_sys::Element>() {
                let link = ctx.link().clone();
                let size_observer = SizeObserver::new(&el, move |(width, height)| {
                    link.send_message(Msg::ViewportResize(width, height));
                });
                self.viewport_size_observer = Some(size_observer);
            }

            if let Some(el) = self.container_ref.cast::<web_sys::Element>() {
                let link = ctx.link().clone();
                let size_observer = SizeObserver::new(&el, move |(width, height)| {
                    link.send_message(Msg::ContainerResize(width, height));
                });
                self.container_size_observer = Some(size_observer);
            }

            if let Some(el) = self.table_ref.cast::<web_sys::HtmlElement>() {
                let link = ctx.link().clone();
                let size_observer = SizeObserver::new(&el, move |(width, height)| {
                    link.send_message(Msg::TableResize(width, height));
                });
                self.table_size_observer = Some(size_observer);
            }
        }

        if self.take_focus {
            // required when we do big jumps (to end, to start),
            // because previous cursor is not rendered (virtual
            // scroll) and looses focus.
            self.take_focus = false;
            self.focus_cursor();
        }
    }
}

impl<T: 'static, S: DataStore<T> + 'static> Into<VNode> for DataTable<T, S> {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtDataTable<T, S>>(Rc::new(self), key);
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
                        if key_str.len() == 0 { break; } // stop on errors
                        // try to find out the column_num
                        let children = el.children();
                        for i in 0..children.length() {
                            let child: web_sys::HtmlElement = children.item(i).unwrap().dyn_into().unwrap();

                            if child.contains(Some(&focused_el)) {
                                if let Some(column_num_str) = child.get_attribute("data-column-num") {
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
                            let child: web_sys::HtmlElement = children.item(i).unwrap().dyn_into().unwrap();
                            let rect = child.get_bounding_client_rect();

                            if rect.x() < click_x && click_x < (rect.x() + rect.width()) {
                                if let Some(column_num_str) = child.get_attribute("data-column-num") {
                                    if let Ok(n) = column_num_str.parse() {
                                        column_num = Some(n);
                                    }
                                }
                            }
                        }

                        if n_str.len() > 0 {
                            return Some((Key::from(n_str), column_num));
                        }
                        break; // stop on errors
                    }
                }
                cur_el = el.parent_element().map(|el| el.dyn_into().unwrap());
            }
            None => break,
        }
    }
    None
}
