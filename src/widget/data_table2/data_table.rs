use std::rc::Rc;

use derivative::Derivative;

use gloo_timers::callback::Timeout;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::IntoPropValue;

use crate::prelude::*;
use crate::state::{optional_rc_ptr_eq, DataFilter};
use crate::widget::{get_unique_element_id, Container, Column, SizeObserver};

use super::{create_combined_sorter_fn, ColumnSorterState, DataTableColumn, DataTableHeader, Header};

pub enum Msg {
    ChangeSort(usize, bool),
    ColumnWidthChange(Vec<usize>),
    ScrollTo(i32, i32),
    ViewportResize(i32, i32),
    ContainerResize(i32, i32),
    TableResize(i32, i32),
    KeyDown(u32),
    CursorDown,
    CursorUp,
    CursorSelect,
    ItemClick(usize),
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

    #[derivative(PartialEq(compare_with="optional_rc_ptr_eq::<T>"))]
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

    /// Initial sort order for columns.
    #[prop_or_default]
    pub sorters: Vec<(usize, bool)>,

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

    /// Builder style method to set the minimum row height
    pub fn min_row_height(mut self, min_row_height: usize) -> Self {
        self.set_min_row_height(min_row_height);
        self
    }

    /// Method to set the minimum row height
    pub fn set_min_row_height(&mut self, min_row_height: usize) {
        self.min_row_height = min_row_height;
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
    sorters: ColumnSorterState,

    store: DataFilter<T>,
    columns: Vec<DataTableColumn<T>>,
    column_widths: Vec<usize>,
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

fn render_empty_row_with_sizes(widths: &[usize]) -> Html {
    Container::new()
        .tag("tr")
        .key(Key::from("sizes"))
         // Note: This row should not be visible, so avoid borders
        .attribute("style", "border-top-width: 0px; border-bottom-width: 0px;")
        .children(
            widths.iter().map(|w| html!{
                <td style={format!("width:{w}px;height:0px;")}></td>
            })
        )
        .into()
}

impl<T: 'static> PwtDataTable<T> {

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

    fn  render_row(&self, props: &DataTable<T>, item: &T, record_num: usize, selected: bool, active: bool) -> Html {

        let key = Key::from(record_num); // fixme: use extract key

        // Make sure our rows have a minimum height
        // Note: setting min-height on <tr> or <td> does not work
        let minheight_cell_style = AttrValue::Rc(format!("height: {}px;", props.min_row_height).into());

        Container::new()
            .tag("tr")
            .key(key)
            .attribute("id", self.get_unique_item_id(record_num))
            .class((active && self.has_focus).then(|| "row-cursor"))
            .children(
                self.columns.iter().enumerate().map(|(_column_num, column)| {
                    let item_style = format!(
                        "vertical-align: {}; text-align: {};",
                        props.vertical_align.as_deref().unwrap_or("baseline"),
                        column.justify,
                    );
                    Container::new()
                        .tag("td")
                        .class(self.cell_class.clone())
                        .class(selected.then(|| "selected"))
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

        let mut active_descendant = None;

        let mut table = Container::new()
            .tag("table")
            .class("pwt-datatable2-content")
            .class(props.hover.then(|| "table-hover"))
            .class(props.striped.then(|| "table-striped"))
            .class(props.bordered.then(|| "table-bordered"))
            .class(props.borderless.then(|| "table-borderless"))
            .node_ref(self.table_ref.clone())
            .attribute("style", format!("table-layout: fixed;width:1px; position:relative;top:{}px;", offset))
            .with_child(render_empty_row_with_sizes(&self.column_widths));

        if !self.column_widths.is_empty() {
            for (filtered_pos, record_num, item) in self.store.filtered_data_range(start..end) {
                let selected = false;

                 let active = self.store
                    .get_cursor().map(|cursor| cursor == filtered_pos)
                    .unwrap_or(false);

                if active {
                    active_descendant = Some(record_num);
                }

                let row = self.render_row(props, item, record_num, selected, active);
                table.add_child(row);
            }
        }

        if let Some(active_descendant) = active_descendant {
            table.set_attribute("aria-activedescendant", self.get_unique_item_id(active_descendant));
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
            .attribute("style", format!(
                "height: 0px; width: 0px; overflow: hidden; position:relative;top:{}px;",
                height
            ))
            .with_child("End Marker for Firefox");

        let height = height + 15; // add some space at the end
        Container::new()
            .attribute("style", format!("height:{}px", height))
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

    type Message = Msg;
    type Properties = DataTable<T>;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let sorters = ColumnSorterState::new(&props.sorters);

        let mut store = DataFilter::new()
            .data(props.data.clone());
        // fixme: set cursor to first selected item
        //.cursor(props.selection)



        let mut columns = Vec::new();
        for header in props.headers.iter() {
            header.extract_column_list(&mut columns);
        }

        store.set_sorter(create_combined_sorter_fn(sorters.sorters(), &columns));

        let cell_class = if props.cell_class.is_empty() {
            Classes::from("pwt-text-truncate pwt-p-2")
        } else {
            props.cell_class.clone()
        };

        let row_count = props.data.as_ref().map(|data| data.len()).unwrap_or(0);
        let virtual_scroll = props.virtual_scroll.unwrap_or(row_count >= VIRTUAL_SCROLL_TRIGGER);

        let mut me = Self {
            unique_id: get_unique_element_id(),
            has_focus: false,
            sorters,
            store,
            columns,
            column_widths: Vec::new(),
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
                    el.scroll_to_with_x_and_y(x as f64, 0.0);
                }
                self.update_scroll_info(props);
                true
            }
            Msg::ViewportResize(_width, height) => {
                self.viewport_height = height.max(0) as usize;
                self.update_scroll_info(props);
                true
            }
            Msg::ContainerResize(width, _height) => {
                self.container_width = width.max(0) as usize;
                true
            }
            Msg::TableResize(_width, height) => {
                let height = height.max(0) as usize;
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
            Msg::KeyDown(key_code) => {
                let msg = match key_code {
                    40 => Msg::CursorDown,
                    38 => Msg::CursorUp,
                    13 => Msg::CursorSelect,
                    _ => return false,
                };
                let link = ctx.link().clone();
                // delay message to give time to render changes
                self.keypress_timeout = Some(Timeout::new(1, move || {
                    link.send_message(msg);
                }));
                false
            }
            Msg::CursorSelect => { /* TODO */ false }
            Msg::CursorDown => {
                self.store.cursor_down();
                self.scroll_cursor_into_view(web_sys::ScrollLogicalPosition::Nearest);
                true
            }
            Msg::CursorUp => {
                self.store.cursor_up();
                self.scroll_cursor_into_view(web_sys::ScrollLogicalPosition::Nearest);
                true
            }
            Msg::ItemClick(record_num) => {
                let _item = match self.store.lookup_record(record_num) {
                    Some(item) => item,
                    None => return false, // should not happen
                };

                self.store.set_cursor(self.store.filtered_pos(record_num));

                // fixme: handle selection

                true
            }
            Msg::FocusChange(has_focus) => {
                self.has_focus = has_focus;
                true
            }
            // Sorting
            Msg::ChangeSort(col_idx, ctrl_key) => {
                if self.columns[col_idx].sorter.is_none() {
                    return false;
                }
                if ctrl_key { // add sorter or reverse direction if exists
                    self.sorters.add_column_sorter(col_idx);
                } else {
                    self.sorters.set_column_sorter(col_idx);
                }
                self.store.set_sorter(create_combined_sorter_fn(self.sorters.sorters(), &self.columns));
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let viewport = Container::new()
            .node_ref(self.scroll_ref.clone())
            .class("pwt-flex-fill")
            .attribute("style", "overflow: auto; outline: 0")
            .attribute("tabindex", "0")
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
                        40 | 38 | 13 => { /* ok */}
                        _ => return,
                    };
                    link.send_message(Msg::KeyDown(event.key_code()));
                    event.prevent_default();
                }
            })
            .onclick({
                let link = ctx.link().clone();
                let unique_row_prefix = format!("{}-item-", self.unique_id);
                move |event: MouseEvent| {
                    let mut cur_el: Option<web_sys::Element> = event.target_dyn_into();
                    loop {
                        match cur_el {
                            Some(el) => {
                                if el.tag_name() == "TR" {
                                    if let Some(n_str) = el.id().strip_prefix(&unique_row_prefix) {
                                        let n: usize = n_str.parse().unwrap();
                                        link.send_message(Msg::ItemClick(n));
                                        break;
                                    }
                                }
                                cur_el = el.parent_element();
                            }
                            None => break,
                        }
                    }
                }
            });

        Column::new()
            .class(props.class.clone())
            .node_ref(self.container_ref.clone())
            .with_child(
                Container::new() // scollable for header
                    .attribute("style", "flex: 0 0 auto;")
                    .class("pwt-overflow-hidden")
                    .node_ref(self.header_scroll_ref.clone())
                    .with_child(
                        DataTableHeader::new(self.container_width, props.headers.clone(), self.sorters.sorters())
                            .header_class(props.header_class.clone())
                            .on_size_change(ctx.link().callback(Msg::ColumnWidthChange))
                            .on_sort_change(ctx.link().callback(|(col, ctrl)| Msg::ChangeSort(col, ctrl)))
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
