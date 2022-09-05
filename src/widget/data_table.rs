use std::rc::Rc;
use std::cmp::Ordering;

use derivative::Derivative;
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::IntoEventCallback;

use crate::state::{DataFilter, Selection};
use crate::props::{ExtractKeyFn, SorterFn, IntoSorterFn, RenderFn};
use crate::widget::{Fa, SizeObserver};
use crate::widget::Resizable;

#[derive(Properties)]
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct DataTableColumn<T> {
    #[prop_or(String::from("100px"))]
    pub width: String,
    pub name: String,
    #[prop_or(String::from("flex-start"))]
    pub justify: String, // flex-start, flex-end, center
    pub render: RenderFn<T>,
    pub sorter: Option<SorterFn<T>>,
}

impl<T> DataTableColumn<T> {

    pub fn new(name: impl Into<String>) -> Self {
        yew::props!(Self {
            name: name.into(),
            render: RenderFn::new(|_| html!{ "-" }),
        })
    }

    pub fn width(mut self, width: impl Into<String>) -> Self {
        self.set_width(width);
        self
    }

    pub fn set_width(&mut self, width: impl Into<String>) {
        self.width = width.into();
    }

    pub fn flex(mut self, flex: usize) -> Self {
        self.set_flex(flex);
        self
    }

    pub fn set_flex(&mut self, flex: usize) {
        self.set_width(format!("{}fr", flex));
    }

    pub fn justify(mut self, justify: impl Into<String>) -> Self {
        self.set_justify(justify);
        self
    }

    pub fn set_justify(&mut self, justify: impl Into<String>) {
        self.justify = justify.into();
    }

    pub fn render(mut self, render: impl Into<RenderFn<T>>) -> Self {
        self.render = render.into();
        self
    }

    pub fn sorter(mut self, sorter: impl IntoSorterFn<T>) -> Self {
        self.sorter = sorter.into_sorter_fn();
        self
    }
}

pub enum Msg {
    ResizeColumn(usize, i32),
    ViewportResize(i32, i32),
    ScrollTo(i32, i32),
    ChangeSort(usize, bool),
    SelectItem(Key),
}

// DataTable properties
#[derive(Properties)]
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct DataTable<T>
where
    T: 'static,
{

    #[prop_or_default]
    node_ref: NodeRef,
    pub key: Option<Key>,

    pub columns: Vec<DataTableColumn<T>>,

    #[prop_or_default]
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    pub items: Rc<Vec<T>>,

    #[prop_or_default]
    pub bordered: bool,

    #[prop_or(true)]
    pub hover: bool,

    #[prop_or_default]
    pub striped: bool,

    /// Height of each row (pixels).
    #[prop_or(30)]
    pub row_height: i32,

    /// set class for table cells (default is "pwt-truncate pwt-p-2")
    pub cell_class: Option<String>,
    /// set class for table header cells (default is "pwt-p-2")
    pub header_class: Option<String>,

    pub extract_key: Option<ExtractKeyFn<T>>,

    #[prop_or_default]
    pub multiselect: bool,

    #[prop_or_default]
    pub sorter: Vec<(usize, bool)>,

    pub onselect: Option<Callback<Vec<Key>>>,

    pub onrowclick: Option<Callback<Key>>,
    pub onrowdblclick: Option<Callback<Key>>,
}

impl <T> DataTable<T> {

    pub fn new(columns: Vec<DataTableColumn<T>>) -> Self {
        yew::props!(DataTable<T> { columns })
    }

    pub fn data(mut self, data: Rc<Vec<T>>) -> Self {
        self.set_data(data);
        self
    }

    pub fn set_data(&mut self, data: Rc<Vec<T>>) {
        self.items = data;
    }

    pub fn row_height(mut self, row_height: i32) -> Self {
        self.set_row_height(row_height);
        self
    }

    pub fn set_row_height(&mut self, row_height: i32) {
        self.row_height = row_height;
    }

    pub fn striped(mut self, striped: bool) -> Self {
        self.set_striped(striped);
        self
    }

    pub fn set_striped(&mut self, striped: bool) {
        self.striped = striped;
    }

    pub fn bordered(mut self, bordered: bool) -> Self {
        self.set_bordered(bordered);
        self
    }

    pub fn set_bordered(&mut self, bordered: bool) {
        self.bordered = bordered;
    }

    pub fn sorter(mut self,  sorter: Vec<(usize, bool)>) -> Self {
        self.sorter = sorter;
        self
    }

    pub fn onselect(mut self, cb: impl IntoEventCallback<Vec<Key>>) -> Self {
        self.onselect = cb.into_event_callback();
        self
    }

    pub fn onrowclick(mut self, cb: impl IntoEventCallback<Key>) -> Self {
        self.onrowclick = cb.into_event_callback();
        self
    }

    pub fn onrowdblclick(mut self, cb: impl IntoEventCallback<Key>) -> Self {
        self.onrowdblclick = cb.into_event_callback();
        self
    }

    pub fn extract_key(mut self, extract_fn: impl Into<ExtractKeyFn<T>>) -> Self {
        self.extract_key = Some(extract_fn.into());
        self
    }

    /// Builder style method to set the yew `node_ref`
    pub fn node_ref(mut self, node_ref: ::yew::html::NodeRef) -> Self {
        self.node_ref = node_ref;
        self
    }

    /// Builder style method to set the yew `key` property
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = Some(key.into());
        self
    }
}

#[doc(hidden)]
pub struct PwtDataTable<T> {
    viewport_height: i32,
    scroll_top: i32,
    visible_rows: i32,

    column_widths: Vec<Option<i32>>, // for column resize
    sorter: Vec<(usize, bool)>,

    data: DataFilter<T>,

    selection: Selection,

    size_observer: Option<SizeObserver>,
}

fn combined_sorter_fn<T: 'static>(
    sorters: &[(usize, bool)],
    columns: &[DataTableColumn<T>]
) -> SorterFn<T> {
    let sorters: Vec<(SorterFn<T>, bool)> = sorters
        .iter()
        .filter_map(|(sort_idx, ascending)| {
            match &columns[*sort_idx].sorter {
                None => None,
                Some(sorter) => Some((sorter.clone(), *ascending)),
            }
        })
        .collect();

    SorterFn::new(move |a: &T, b: &T| {
        for (sort_fn, ascending) in &sorters {
            match if *ascending {
                sort_fn.cmp(a, b)
            } else {
                sort_fn.cmp(b, a)
            } {
                Ordering::Equal => { /* continue */ },
                other => return other,
            }
        }
        Ordering::Equal
    })
}

impl <T> Component for PwtDataTable<T>
where
    T: 'static,
{
    type Message = Msg;
    type Properties = DataTable<T>;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let mut data = DataFilter::new()
            .data(props.items.clone());

        data.set_sorter(combined_sorter_fn(&props.sorter, &props.columns));

        Self {
            viewport_height: 0,
            visible_rows: 0,
            scroll_top: 0,
            column_widths: Vec::new(),
            sorter: props.sorter.clone(),
            data,
            selection: Selection::new().multiselect(props.multiselect),
            size_observer: None,
         }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::SelectItem(key) => {
                self.selection.select(&key);
                if let Some(onselect) = &props.onselect {
                    let keys = self.selection.selected_keys(&props.items, props.extract_key.as_ref());
                    onselect.emit(keys);
                }
                true
            }
            Msg::ViewportResize(_width, height) => {
                //log::info!("VR {} {}", width, height);
                self.viewport_height = height;
                self.visible_rows = (height / ctx.props().row_height) + 3;
                true
            }
            Msg::ScrollTo(_x, y) => {
                //log::info!("ST {} {}", x, y);
                self.scroll_top = y;
                true
            }
            Msg::ResizeColumn(col_idx, width) => {
                self.column_widths.resize((col_idx + 1).max(self.column_widths.len()), None);
                self.column_widths[col_idx] = Some(width+2);
                //log::info!("SC {:?}", self.column_widths);
                true
            }
            Msg::ChangeSort(col_idx, ctrl_key) => {
                if ctx.props().columns[col_idx].sorter.is_none() {
                    return false;
                }

                if ctrl_key { // add sorter or reverse direction if exists
                    let mut found = false;
                    for (idx, ascending) in self.sorter.iter_mut() {
                        if *idx == col_idx {
                            *ascending = !*ascending;
                            found = true;
                        }
                    }
                    if !found {
                        self.sorter.push((col_idx, true));
                    }
                } else {
                    if self.sorter.len() == 1 {
                        let (cur_idx, ascending) = self.sorter[0];
                        if cur_idx == col_idx {
                            self.sorter = vec![(col_idx, !ascending)];
                        } else {
                            self.sorter = vec![(col_idx, true)];
                        }
                    } else {
                        self.sorter = vec![(col_idx, true)];
                    }
                }

                self.data.set_sorter(combined_sorter_fn(&self.sorter, &props.columns));

                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();

        if !Rc::ptr_eq(&props.items, &old_props.items) { // data changed
            if props.extract_key.is_none() {
                self.selection.clear();
            } else {
                /* we have unique keys to identify records, so we can keep
                 * the selection */
            }

            // make sure selection only contains valid records
            if let Some(onselect) = &props.onselect {
                let keys = self.selection.selected_keys(&props.items, props.extract_key.as_ref());
                onselect.emit(keys);
            }

            self.data.set_data(Rc::clone(&props.items));
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let row_count = self.data.filtered_data_len();
        let mut start = (self.scroll_top / props.row_height) as usize;
        if start > 0 { start -= 1; }
        if (start & 1) == 1 { start -= 1; } // make it work with striped rows

        let end = (start+self.visible_rows as usize).min(row_count) as usize;

        let cell_class = props.cell_class.clone()
            .unwrap_or_else(|| String::from("pwt-text-truncate pwt-p-2"));

        let mut content = Vec::new();

        for (_i, record_num, item) in self.data.filtered_data_range(start..end) {
            let mut row = Vec::new();

            let key = match &props.extract_key {
                None => Key::from(record_num),
                Some(extract_fn) => extract_fn.apply(item),
            };

            let selected = self.selection.is_selected(&key);

            let onclick = ctx.link().callback({
                let key = key.clone();
                let callback = props.onrowclick.clone();
                move |_| {
                    if let Some(callback) = &callback {
                        callback.emit(key.clone());
                    }
                    Msg::SelectItem(key.clone())
                }
            });

            let ondblclick = Callback::from({
                let key = key.clone();
                let callback = props.onrowdblclick.clone();
                move |_| {
                    if let Some(callback) = &callback {
                        callback.emit(key.clone());
                    }
                }
            });

            for (n, column) in props.columns.iter().enumerate() {
                let item_style = format!("justify-content:{}; grid-column:{};", column.justify, n+1);
                let class = if selected { Some("selected") } else {None };
                row.push(html!{
                    <td {class} style={item_style}><div class={&cell_class}>{ column.render.apply(item) }</div></td>
                });
            }
            content.push(html!{
                <tr {key} {onclick} {ondblclick}>{row}</tr>
            });
        };

        let virtual_height = row_count * props.row_height as usize;

        let node_ref = props.node_ref.clone();
        let onscroll = ctx.link().batch_callback(move |_: Event| {
            if let Some(el) = node_ref.cast::<web_sys::Element>() {
                Some(Msg::ScrollTo(el.scroll_left(), el.scroll_top()))
            } else {
                None
            }
        });

        let content_style = format!("height:{}px", virtual_height);
        let window_style = format!(
            "position:relative;top:{}px;",
            start * (props.row_height as usize),
        );
        let viewport_style = "height:100%;overflow:auto;";

        let mut grid_style = format!("display:grid; grid-auto-rows:{}px; grid-template-columns: ", props.row_height);

        for (col_idx, column) in props.columns.iter().enumerate() {
            if let Some(Some(width)) = self.column_widths.get(col_idx) {
                grid_style.push_str(&format!("{}px", width));
            } else {
                grid_style.push_str(&column.width);
            }
            grid_style.push(' ');
        }
        grid_style.push(';');

        let mut headers = Vec::new();

        let link = ctx.link();

        for (n, column) in props.columns.iter().enumerate() {

            let mut sort_icon = html!{};
            for (sort_idx, ascending) in &self.sorter {
                if *sort_idx == n {
                    sort_icon = if *ascending {
                        Fa::new("long-arrow-up").class("pwt-ps-1").into()
                    } else {
                        Fa::new("long-arrow-down").class("pwt-ps-1").into()
                    };
                    break;
                }
            }

            let onresize = link.callback(move |width| Msg::ResizeColumn(n, width));
            let onsort = link.callback(move |event: MouseEvent| Msg::ChangeSort(n, event.ctrl_key()));

            let item_style = format!("position: sticky; top: 0px; grid-column:{}; justify-content:{};", n+1, column.justify);

            let header_class = classes!{
                "pwt-nowrap",
                props.header_class.clone().or_else(|| Some(String::from("pwt-p-2"))),
            };

            headers.push(html!{
                <th style={item_style}>
                    <Resizable {onresize} justify={column.justify.clone()}>
                    <div class={header_class} unselectable="on" onclick={onsort} style="overflow: hidden; text-overflow: ellipsis; user-select: none">{ column.name.clone() }{sort_icon}</div>
                    </Resizable>
                </th>
            });
        }

        let table_class = classes!(
            "datatable",
            props.hover.then(|| "table-hover"),
            props.striped.then(|| "table-striped"),
            props.bordered.then(|| "table-bordered"),
        );

        let classes = "pwt-flex-fill";
        html!{
            <div class={classes} {onscroll} ref={props.node_ref.clone()} style={viewport_style}>
              <div style={content_style}>
                 <div style={window_style}>
                <table class={table_class} style={grid_style}>
                     <thead><tr>{headers}</tr></thead>
                     <tbody>{content}</tbody>
                   </table>
                </div>
              </div>
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            if let Some(el) = ctx.props().node_ref.cast::<web_sys::Element>() {
                let link = ctx.link().clone();
                let size_observer = SizeObserver::new(&el, move |(width, height)| {
                    link.send_message(Msg::ViewportResize(width, height));
                });
                self.size_observer = Some(size_observer);
            }
        }
    }
}

impl <T> Into<VNode> for DataTable<T> {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtDataTable<T>>(Rc::new(self), key);
        VNode::from(comp)
    }

}
