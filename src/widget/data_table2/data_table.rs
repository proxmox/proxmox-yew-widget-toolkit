use std::rc::Rc;

use derivative::Derivative;

use gloo_timers::callback::Timeout;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::{IntoEventCallback, IntoPropValue};

use crate::prelude::*;
use crate::props::{Selection2, SorterFn};
use crate::state::{optional_rc_ptr_eq, DataFilter};
use crate::widget::{get_unique_element_id, Container, Column, SizeObserver};

use super::{create_indexed_header_list, DataTableColumn, DataTableHeader, Header, IndexedHeader};

pub enum Msg<T: 'static> {
    ChangeSort(SorterFn<T>),
    ColumnWidthChange(Vec<f64>),
    ColumnHiddenChange(Vec<bool>),
    ScrollTo(i32, i32),
    ViewportResize(f64, f64),
    ContainerResize(f64, f64),
    TableResize(f64, f64),
    KeyDown(u32, bool, bool),
    CursorDown(bool, bool),
    CursorUp(bool, bool),
    ItemClick(usize, bool, bool),
    ItemDblClick(usize),
    FocusChange(bool),
 }

// DataTable properties
#[derive(Properties)]
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct DataTable<T: 'static> {

    #[prop_or_default]
    node_ref: NodeRef,
    pub key: Option<Key>,

    #[prop_or_default]
    pub class: Classes,

    headers: Rc<Vec<Header<T>>>,

    #[derivative(PartialEq(compare_with="optional_rc_ptr_eq::<Vec<T>>"))]
    pub data: Option<Rc<Vec<T>>>,

    /// set class for table cells (default is "pwt-p-2")
    #[prop_or_default]
    pub cell_class: Classes,

    /// set class for header cells (default is "pwt-p-2")
    #[prop_or_default]
    pub header_class: Classes,

    #[prop_or_default]
    pub bordered: bool,

    #[prop_or_default]
    pub borderless: bool,

    #[prop_or_default]
    pub hover: bool,

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

    pub selection: Option<Selection2<T>>,

    /// Row click callback (parameter is the record number)
    pub onrowclick: Option<Callback<usize>>,

    /// Row double click callback (parameter is the record number)
    pub onrowdblclick: Option<Callback<usize>>,
}

static VIRTUAL_SCROLL_TRIGGER: usize = 30;

impl <T: 'static> DataTable<T> {

    /// Create a new instance.
    pub fn new(headers: Rc<Vec<Header<T>>>) -> Self {
        yew::props!(DataTable<T> { headers })
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

    pub fn data(mut self, data: impl IntoPropValue<Option<Rc<Vec<T>>>>) -> Self {
        self.set_data(data);
        self
    }

    pub fn set_data(&mut self, data: impl IntoPropValue<Option<Rc<Vec<T>>>>) {
        self.data = data.into_prop_value();
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
    pub fn selection(mut self, selection: impl IntoPropValue<Option<Selection2<T>>>) -> Self {
        self.selection = selection.into_prop_value();
        self
    }

    /// Builder style method to set the row click callback.
    pub fn onrowclick(mut self, cb: impl IntoEventCallback<usize>) -> Self {
        self.onrowclick = cb.into_event_callback();
        self
    }

    /// Builder style method to set the row double click callback.
    pub fn onrowdblclick(mut self, cb: impl IntoEventCallback<usize>) -> Self {
        self.onrowdblclick = cb.into_event_callback();
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

#[doc(hidden)]
pub struct PwtDataTable<T: 'static> {
    unique_id: String,
    has_focus: bool,

    store: DataFilter<T>,

    headers: Rc<Vec<IndexedHeader<T>>>,

    columns: Vec<DataTableColumn<T>>,
    column_widths: Vec<f64>,
    column_hidden: Vec<bool>,
    virtual_scroll: bool,
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

impl<T: 'static> PwtDataTable<T> {

    fn select_position(
        &mut self,
        selection: &Selection2<T>,
        cursor: usize,
        _shift: bool,
        ctrl: bool,
    ) {
        if let Some((_, item)) = self.store.lookup_filtered_record(cursor) {
            if ctrl {
                selection.toggle(item);
            } else {
                selection.select(item);
            }
        }
    }

    fn select_range(
        &mut self,
        selection: &Selection2<T>,
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
                    self.select_position(selection, pos, shift, ctrl);
                }
            } else {
                self.select_position(selection, new_cursor, shift, ctrl);
            }
        }
    }

    fn select_cursor(&mut self, props: &DataTable<T>, shift: bool, ctrl: bool) -> bool {
        let selection = match &props.selection {
            Some(selection) => selection,
            None => return false,
        };

        let cursor = match self.store.get_cursor() {
            Some(c) => c,
            None => return false, // nothing to do
        };

        if !(shift || ctrl) { selection.clear(); }

        if let Some((_, item)) = self.store.lookup_filtered_record(cursor) {
            if ctrl {
                selection.toggle(item);
            } else {
                selection.select(item);
            }
            return true;
        }
        false
    }

    fn get_unique_item_id(&self, n: usize) -> String {
        format!("{}-item-{}", self.unique_id, n)
    }

    fn cursor_row_is_rendered(&self, cursor: usize) -> bool {
        (self.scroll_info.start..self.scroll_info.end).contains(&cursor)
    }

    fn scroll_cursor_into_view(&self, pos: web_sys::ScrollLogicalPosition) {
        let cursor = match self.store.get_cursor() {
            Some(cursor) => cursor,
            None => return,
        };

        if !self.cursor_row_is_rendered(cursor) {
            self.scroll_to_cursor(cursor);
            return;
        }

        if let Some(n) = self.store.unfiltered_pos(cursor) {
            self.scroll_item_into_view(n, pos);
        }
    }

    fn scroll_to_cursor(&self, cursor: usize) {
        let height =  (self.row_height * cursor).saturating_sub(self.viewport_height/2);
        if let Some(el) = self.scroll_ref.cast::<web_sys::Element>() {
            el.set_scroll_top(height as i32);
        }
    }

    fn scroll_item_into_view(&self, n: usize, pos: web_sys::ScrollLogicalPosition) {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let id = self.get_unique_item_id(n);

        let el = match document.get_element_by_id(&id) {
            Some(el) => el,
            None => return,
        };

        let mut options = web_sys::ScrollIntoViewOptions::new();
        options.block(pos);
        el.scroll_into_view_with_scroll_into_view_options(&options);
    }

    fn  render_row(&self, props: &DataTable<T>, item: &T, row_num: usize, record_num: usize, selected: bool, active: bool) -> Html {

        let key = Key::from(record_num); // fixme: use extract key

        // Make sure our rows have a minimum height
        // Note: setting min-height on <tr> or <td> does not work
        let minheight_cell_style = AttrValue::Rc(format!("height: {}px;", props.min_row_height).into());

        Container::new()
            .tag("tr")
            .key(key)
            .attribute("role", "row")
            .attribute("aria-rowindex", row_num.to_string())
            .attribute("id", self.get_unique_item_id(record_num))
            .class((active && self.has_focus).then(|| "row-cursor"))
            .class(selected.then(|| "selected"))
            .children(
                self.columns.iter().enumerate()
                    .filter(|(column_num, _)| {
                        match self.column_hidden.get(*column_num) {
                            Some(true) => false,
                            _ => true,
                        }
                    })
                    .map(|(_column_num, column)| {
                        let item_style = format!(
                            "vertical-align: {}; text-align: {};",
                            props.vertical_align.as_deref().unwrap_or("baseline"),
                            column.justify,
                        );
                        Container::new()
                            .tag("td")
                            .class(self.cell_class.clone())
                            .attribute("style", item_style)
                            .with_child(html!{
                                <div>{
                                    column.render.apply(item)
                                }</div>
                            })
                            .into()
                    })
            )
            .with_child(html!{<th style={minheight_cell_style.clone()}/>})
            .into()
    }

    fn render_table(&self, props: &DataTable<T>, offset: usize, start: usize, end: usize) -> Html {

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

        if !self.column_widths.is_empty() {
            for (filtered_pos, record_num, item) in self.store.filtered_data_range(start..end) {

                let mut selected = false;
                if let Some(selection) = &props.selection {
                    selected = selection.contains(item);
                }

                let active = self.store
                    .get_cursor().map(|cursor| cursor == filtered_pos)
                    .unwrap_or(false);

                let row = self.render_row(props, item, filtered_pos, record_num, selected, active);
                table.add_child(row);
            }
        }

        table.into()
    }

    fn render_scroll_content(
        &self,
        props: &DataTable<T>,
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
        props: &DataTable<T>,
    ) {
        let row_count = self.store.filtered_data_len();

        let mut start = if self.virtual_scroll {
            self.scroll_top / self.row_height
        } else {
            0
        };

        if start > 0 { start -= 1; }
        if (start & 1) == 1 { start -= 1; } // make it work with striped rows

        let max_visible_rows = (self.viewport_height / props.min_row_height) + 5;
        let end = if self.virtual_scroll {
            (start + max_visible_rows).min(row_count)
        } else {
            row_count
        };

        let offset = start * self.row_height;

        let height = offset + self.table_height + row_count.saturating_sub(end) * self.row_height;

        self.scroll_info = VirtualScrollInfo { start, end, offset, height };
    }
}

impl <T: 'static> Component for PwtDataTable<T> {

    type Message = Msg<T>;
    type Properties = DataTable<T>;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let headers = create_indexed_header_list(&props.headers);

        let mut store = DataFilter::new()
            .data(props.data.clone());
        // fixme: set cursor to first selected item
        //.cursor(props.selection)

        store.set_cursor(Some(0));

        // fixme: remove
        let mut columns = Vec::new();
        for header in props.headers.iter() {
            header.extract_column_list(&mut columns);
        }

        //store.set_sorter(create_combined_sorter_fn(sorters.sorters(), &columns));

        let cell_class = if props.cell_class.is_empty() {
            Classes::from("pwt-text-truncate pwt-p-2")
        } else {
            props.cell_class.clone()
        };

        let row_count = props.data.as_ref().map(|data| data.len()).unwrap_or(0);
        let virtual_scroll = props.virtual_scroll.unwrap_or(row_count >= VIRTUAL_SCROLL_TRIGGER);

        let mut me = Self {
            headers: Rc::new(headers),
            unique_id: get_unique_element_id(),
            has_focus: false,
            store,
            columns,
            column_widths: Vec::new(),
            column_hidden: Vec::new(),
            virtual_scroll,
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
        };

        me.update_scroll_info(props);
        me
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
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
            Msg::KeyDown(key_code, shift, ctrl) => {
                let msg = match key_code {
                    40 => Msg::CursorDown(shift, ctrl),
                    38 => Msg::CursorUp(shift, ctrl),
                    32 => {
                        self.select_cursor(props, shift, ctrl);
                        return true;
                    }
                    35 => {
                        // end
                        self.store.set_cursor(None);
                        self.store.cursor_up();
                        self.scroll_cursor_into_view(web_sys::ScrollLogicalPosition::Nearest);
                        return true;
                    }
                    36 => {
                        // pos1
                        self.store.set_cursor(None);
                        self.store.cursor_down();
                        self.scroll_cursor_into_view(web_sys::ScrollLogicalPosition::Nearest);
                        return true;
                    }
                    13 => {
                        // Return - same behavior as rowdblclick
                        let cursor = match self.store.get_cursor() {
                            Some(cursor) => cursor,
                            None => return false,
                        };
                        let record_num = match self.store.unfiltered_pos(cursor) {
                            Some(record_num) => record_num,
                            None => return false,
                        };

                        self.select_cursor(props, false, false);

                        if let Some(callback) = &props.onrowdblclick {
                            callback.emit(record_num);
                        }

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
            Msg::CursorDown(shift, _ctrl) => {
                if shift { self.select_cursor(props, shift, false); }
                self.store.cursor_down();
                if shift { self.select_cursor(props, shift, false); }
                self.scroll_cursor_into_view(web_sys::ScrollLogicalPosition::Nearest);
                true
            }
            Msg::CursorUp(shift, _ctrl) => {
                if shift { self.select_cursor(props, shift, false); }
                self.store.cursor_up();
                if shift { self.select_cursor(props, shift, false); }
                self.scroll_cursor_into_view(web_sys::ScrollLogicalPosition::Nearest);
                true
            }
            Msg::ItemClick(record_num, shift, ctrl) => {
                let last_cursor = self.store.get_cursor();
                let new_cursor = self.store.filtered_pos(record_num);

                self.store.set_cursor(new_cursor);

                if shift || ctrl {
                    if let Some(selection) = &props.selection {
                        self.select_range(selection, last_cursor, new_cursor, shift, ctrl);
                    }
                }

                if let Some(callback) = &props.onrowclick {
                    callback.emit(record_num);
                }

                true
            }
            Msg::ItemDblClick(record_num) => {
                self.store.set_cursor(self.store.filtered_pos(record_num));
                self.select_cursor(props, false, false);

                if let Some(callback) = &props.onrowdblclick {
                    callback.emit(record_num);
                }

                true
            }
            Msg::FocusChange(has_focus) => {
                if self.has_focus == has_focus { return false; }

                self.has_focus = has_focus;

                true
            }
            Msg::ChangeSort(sorter_fn) => {
                self.store.set_sorter(sorter_fn);
                true
            }
            Msg::ColumnHiddenChange(column_hidden) => {
                self.column_hidden = column_hidden;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let row_count = self.store.filtered_data_len();

        let active_descendant = self.store
            .get_cursor()
            .map(|cursor|  self.get_unique_item_id(cursor));

       let viewport = Container::new()
            .node_ref(self.scroll_ref.clone())
            .class("pwt-flex-fill")
            .attribute("style", "overflow: auto; outline: 0")
            .attribute("tabindex", "0")
            .attribute("role", "grid")
            .attribute("aria-label", "table body")
            .attribute("aria-activedescendant", active_descendant)
            .attribute("aria-rowcount", row_count.to_string())
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
                    match event.key_code() {
                        40 | 38 | 13 | 32 | 35 | 36 => { /* ok */}
                        _ => return,
                    };
                    link.send_message(Msg::KeyDown(
                        event.key_code(),
                        event.shift_key(),
                        event.ctrl_key(),
                    ));
                    event.prevent_default();
                }
            })
            .onclick({
                let link = ctx.link().clone();
                let unique_id = self.unique_id.clone();
                move |event: MouseEvent| {
                    if let Some(n) = dom_find_record_num(&event, &unique_id) {
                        link.send_message(Msg::ItemClick(
                            n,
                            event.shift_key(),
                            event.ctrl_key(),
                          ));
                    }
                }
            })
            .ondblclick({
                let link = ctx.link().clone();
                let unique_id = self.unique_id.clone();
                move |event: MouseEvent| {
                    if let Some(n) = dom_find_record_num(&event, &unique_id) {
                        link.send_message(Msg::ItemDblClick(n));
                    }
                }
            });

        Column::new()
            .class(props.class.clone())
            .node_ref(self.container_ref.clone())
            .with_child(
                Container::new() // scollable for header
                    .node_ref(self.header_scroll_ref.clone())
                    .attribute("style", "flex: 0 0 auto")
                    .class("pwt-overflow-hidden")
                    .class("pwt-datatable2-header")
                    .with_child(
                        DataTableHeader::new(self.headers.clone())
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

        if !optional_rc_ptr_eq(&props.data, &old_props.data) { // data changed
            self.store.set_data(props.data.clone());
            let row_count = props.data.as_ref().map(|data| data.len()).unwrap_or(0);
            self.virtual_scroll = props.virtual_scroll.unwrap_or(row_count >= VIRTUAL_SCROLL_TRIGGER);
            if let Some(selection) = &props.selection {
                selection.filter_nonexistent(self.store.filtered_data().map(|t| t.2));
            }
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
    }
}

impl<T: 'static> Into<VNode> for DataTable<T> {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtDataTable<T>>(Rc::new(self), key);
        VNode::from(comp)
    }
}


fn dom_find_record_num(event: &MouseEvent, unique_id: &str) -> Option<usize> {

    let unique_row_prefix = format!("{}-item-", unique_id);

    let mut cur_el: Option<web_sys::Element> = event.target_dyn_into();
    loop {
        match cur_el {
            Some(el) => {
                if el.tag_name() == "TR" {
                    if let Some(n_str) = el.id().strip_prefix(&unique_row_prefix) {
                        if let Ok(n) = n_str.parse() {
                            return Some(n);
                        }
                        break; // stop on errors
                    }
                }
                cur_el = el.parent_element();
            }
            None => break,
        }
    }
    None
}
