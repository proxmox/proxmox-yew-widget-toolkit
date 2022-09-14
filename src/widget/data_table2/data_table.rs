use std::rc::Rc;

use derivative::Derivative;
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::IntoPropValue;

use crate::prelude::*;
use crate::state::{optional_rc_ptr_eq, DataFilter};
use crate::widget::{Container, Column, SizeObserver};

use super::{DataTableColumn, DataTableHeader, Header};

pub enum Msg {
    ColumnWidthChange(Vec<usize>),
    ScrollTo(i32, i32),
    ViewportResize(i32, i32),
    ContainerResize(i32, i32),
    TableHeight(usize),
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

    /// set class for table cells (default is "pwt-truncate pwt-p-2")
    #[prop_or_default]
    pub cell_class: Classes,

    #[prop_or_default]
    pub bordered: bool,

    #[prop_or_default]
    pub borderless: bool,

    #[prop_or(true)]
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

    row_height: usize,

    container_ref: NodeRef,
    container_size_observer: Option<SizeObserver>,
    container_width: usize,
}

fn render_empty_row_with_sizes(widths: &[usize]) -> Html {
    Container::new()
        .tag("tr")
        .key(Key::from("sizes"))
        .children(
            widths.iter().map(|w| html!{
                <td style={format!("width:{w}px;height:0px;")}></td>
            })
        )
        .into()
}

impl<T: 'static> PwtDataTable<T> {

    fn render_row(&self, props: &DataTable<T>, item: &T, record_num: usize, selected: bool) -> Html {

        let key = Key::from(record_num); // fixme: use extract key

        // Make sure our rows have a minimum height
        // Note: setting min-height on <tr> or <td> does not work
        let minheight_cell_style = AttrValue::Rc(format!("height: {}px;", props.min_row_height).into());

        Container::new()
            .tag("tr")
            .key(key)
            .attribute("id", format!("record-nr-{}", record_num))
            .children(
                self.columns.iter().enumerate().map(|(_column_num, column)| {
                    let item_style = format!(
                        "vertical-align: {}; text-align: {};",
                        props.vertical_align.as_deref().unwrap_or("baseline"),
                        column.justify,
                    );
                    let class = if selected { Some("selected") } else {None };
                    Container::new()
                        .tag("td")
                        .attribute("style", item_style)
                        .class(class)
                        .with_child(html!{
                            <div class={self.cell_class.clone()}>{
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
            .tag("table")
            .class("pwt-datatable2-content")
            .class(props.hover.then(|| "table-hover"))
            .class(props.striped.then(|| "table-striped"))
            .class(props.bordered.then(|| "table-bordered"))
            .class(props.borderless.then(|| "table-borderless"))
            .node_ref(self.table_ref.clone())
            .attribute("style", format!("table-layout: fixed;width:1px; position:relative;top:{}px;", offset))
            .with_child(render_empty_row_with_sizes(&self.column_widths));

        for (_i, record_num, item) in self.store.filtered_data_range(start..end) {
            let selected = false;
            let row = self.render_row(props, item, record_num, selected);
            table.add_child(row);
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

        let store = DataFilter::new()
            .data(props.data.clone());

        let mut columns = Vec::new();
        for header in props.headers.iter() {
            header.extract_column_list(&mut columns);
        }

        let cell_class = if props.cell_class.is_empty() {
            Classes::from("pwt-text-truncate pwt-p-2")
        } else {
            props.cell_class.clone()
        };

        let row_count = props.data.as_ref().map(|data| data.len()).unwrap_or(0);
        let virtual_scroll = props.virtual_scroll.unwrap_or(row_count >= VIRTUAL_SCROLL_TRIGGER);

        let mut me = Self {
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
            table_height: 0,

            container_ref: NodeRef::default(),
            container_size_observer: None,
            container_width: 0,

            row_height: props.min_row_height,
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
            Msg::ContainerResize(width, height) => {
                self.container_width = width.max(0) as usize;
                true
            }
            Msg::TableHeight(height) => {
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
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let scroll_content = if !self.column_widths.is_empty() {
            self.render_scroll_content(props)
        } else {
            html!{}
        };

        let viewport = Container::new()
            .node_ref(self.scroll_ref.clone())
            .class("pwt-flex-fill")
            .attribute("style", "overflow: auto; outline: 0")
             // fixme: howto handle focus?
            .attribute("tabindex", "0")
            .with_child(scroll_content)
            .onscroll(ctx.link().batch_callback(move |event: Event| {
                let target: Option<web_sys::HtmlElement> = event.target_dyn_into();
                target.map(|el| Msg::ScrollTo(el.scroll_left(), el.scroll_top()))
            }));

        Column::new()
            .class(props.class.clone())
            .node_ref(self.container_ref.clone())
            .with_child(
                Container::new() // scollable for header
                    .attribute("style", "flex: 0 0 auto;")
                    .class("pwt-overflow-hidden")
                    .node_ref(self.header_scroll_ref.clone())
                    .with_child(
                        DataTableHeader::new(self.container_width, props.headers.clone())
                            .on_size_change(ctx.link().callback(Msg::ColumnWidthChange))
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
        }

        if let Some(el) = self.table_ref.cast::<web_sys::HtmlElement>() {
            let height = el.offset_height().max(0) as usize;
            if self.table_height != height {
                ctx.link().send_message(Msg::TableHeight(height));
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
