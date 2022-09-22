use std::rc::Rc;

use derivative::Derivative;

use gloo_timers::callback::Timeout;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::{IntoPropValue, IntoEventCallback, Scope};

use crate::prelude::*;
use crate::props::SorterFn;
use crate::widget::{Container, Fa};
use crate::widget::focus::{focus_next_tabable, init_roving_tabindex};

use super::{
    IndexedHeader, IndexedHeaderSingle, IndexedHeaderGroup,
    HeaderMenu, HeaderState, ResizableHeader,
};

#[derive(Properties)]
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct DataTableHeader<T: 'static> {
    pub node_ref: Option<NodeRef>,
    pub key: Option<Key>,

    headers: Rc<Vec<IndexedHeader<T>>>,

    pub on_size_change: Option<Callback<Vec<usize>>>,
    pub on_sort_change: Option<Callback<SorterFn<T>>>,

    /// set class for header cells
    #[prop_or_default]
    pub header_class: Classes,
}


impl<T: 'static> DataTableHeader<T> {

    /// Create a new instance.
    pub fn new(headers: Rc<Vec<IndexedHeader<T>>>) -> Self {
        yew::props!(Self { headers })
    }

    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Builder style method to set the yew `node_ref`
    pub fn node_ref(mut self, node_ref: impl IntoPropValue<Option<NodeRef>>) -> Self {
        self.node_ref = node_ref.into_prop_value();
        self
    }

    /// Builder style method to set the size change callback
    pub fn on_size_change(mut self, cb: impl IntoEventCallback<Vec<usize>>) -> Self {
        self.on_size_change = cb.into_event_callback();
        self
    }

    /// Builder style method to set the sort change callback
    ///
    /// Callback partameters: (column_num, ctrl, order)
    pub fn on_sort_change(mut self, cb: impl IntoEventCallback<SorterFn<T>>) -> Self {
        self.on_sort_change = cb.into_event_callback();
        self
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
}

pub enum Msg {
    ResizeColumn(usize, usize),
    ColumnSizeReset(usize),
    ColumnSizeChange(usize, i32), // fixme
    ColumnSortChange(usize, bool, Option<bool>),
    HideClick(usize)
}


fn column_to_rows<T: 'static>(
    state: &HeaderState<T>,
    cell: &IndexedHeaderSingle<T>,
    props: &DataTableHeader<T>,
    link: &Scope<PwtDataTableHeader<T>>,
    start_row: usize,
    rows: &mut Vec<Vec<Html>>,
) {
    rows.resize((start_row + 1).max(rows.len()), Vec::new());

    let start_col = cell.start_col;
    let cell_idx = cell.cell_idx;

    let sort_order = state.get_column_sorter(cell_idx);
    let sort_icon = match sort_order {
        Some(ascending) => {
            if ascending {
                Fa::new("long-arrow-up").class("pwt-pe-1").into()
            } else {
                Fa::new("long-arrow-down").class("pwt-pe-1").into()
            }
        }
        None =>  html!{},
    };

    rows[start_row].push(
        Container::new()
            .tag("th")
            .attribute("role", "columnheader")
            .attribute(
                "style",
                format!("grid-row: {} / 10;grid-column-start: {}", start_row + 1, start_col + 1)
            )
            .with_child(
                ResizableHeader::new()
                    .class(props.header_class.clone())
                    .class("pwt-w-100 pwt-h-100")
                    .content(html!{<>{sort_icon}{&cell.column.name}</>})
                    .on_resize({
                        let link = link.clone();
                        move |width| {
                            let width: usize = if width > 0 { width as usize } else  { 0 };
                            link.send_message(Msg::ResizeColumn(start_col, width));
                        }
                    })
                    .on_size_reset(link.callback(move |_| Msg::ColumnSizeReset(start_col)))
                    .on_size_change(link.callback(move |w| Msg::ColumnSizeChange(start_col, w)))
                    .picker({
                        let headers = Rc::clone(&props.headers);
                        let link = link.clone();
                        let hidden = Vec::from(state.hidden_cells());
                        move |_: &()| {
                            HeaderMenu::new(Rc::clone(&headers), &hidden)
                                .on_sort_change(link.callback(move |asc| {
                                    Msg::ColumnSortChange(cell_idx, false, Some(asc))
                                }))
                                .on_hide_click(link.callback(Msg::HideClick))
                                .into()
                        }
                    })
            )
            .ondblclick(link.callback(move |event: MouseEvent| {
                Msg::ColumnSortChange(cell_idx, event.ctrl_key(), None)
            }))
            .into()
    );
}

fn header_list_to_rows<T: 'static>(
    state: &HeaderState<T>,
    list: &[IndexedHeader<T>],
    props: &DataTableHeader<T>,
    link: &Scope<PwtDataTableHeader<T>>,
    start_row: usize,
    rows: &mut Vec<Vec<Html>>,
) {
    for child in list {
        match child {
            IndexedHeader::Single(column) => {
                column_to_rows(state, column, props, link, start_row, rows);
            }
            IndexedHeader::Group(group) => {
                group_to_rows(state, group, props, link, start_row, rows);
            }
        }
    }
}

fn group_to_rows<T: 'static>(
    state: &HeaderState<T>,
    group: &IndexedHeaderGroup<T>,
    props: &DataTableHeader<T>,
    link: &Scope<PwtDataTableHeader<T>>,
    start_row: usize,
    rows: &mut Vec<Vec<Html>>,
) {
    rows.resize((start_row + 1).max(rows.len()), Vec::new());

    header_list_to_rows(state, &group.children, props, link, start_row + 1, rows);

    rows[start_row].push(
        Container::new()
            .tag("th")
            .attribute("role", "columnheader")
            .attribute("tabindex", "-1")
            .class("pwt-datatable2-group-header-item")
            .class(props.header_class.clone())
            .attribute("style", format!(
                "grid-column: {} / span {}",
                group.start_col + 1,
                group.colspan,
            ))
            .with_child(group.name.clone())
            .into()
    );
}

pub struct PwtDataTableHeader<T: 'static> {
    node_ref: NodeRef,
    /// Sort order state for columns.
    state: HeaderState<T>,

    observed_widths: Vec<Option<usize>>,

    timeout: Option<Timeout>,
}

static RESERVED_SPACE: usize = 20;

impl <T: 'static> PwtDataTableHeader<T> {

    fn compute_grid_style(&self) -> String {

        let mut grid_style = String::from("user-select: none; display:grid; grid-template-columns:");
        for (col_idx, cell) in self.state.columns().iter().enumerate() {
            if let Some(width) = self.state.get_width(col_idx) {
                grid_style.push_str(&format!("{}px", width));
            } else {
                grid_style.push_str(&cell.column.width);
            }
            grid_style.push(' ');
        }

        grid_style.push_str(&format!(" {}px;", RESERVED_SPACE));

        grid_style
    }
}

impl <T: 'static> Component for PwtDataTableHeader<T> {
    type Message = Msg;
    type Properties = DataTableHeader<T>;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let state = HeaderState::new(Rc::clone(&props.headers));

        Self {
            node_ref: props.node_ref.clone().unwrap_or(NodeRef::default()),
            state,
            observed_widths: Vec::new(),
            timeout: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::ResizeColumn(col_idx, width) => {
                self.state.set_width(col_idx, Some(width.max(40)));

                // Set flex columns on the left to fixed size to avoid unexpected effects.
                self.state.copy_observed_widths(col_idx, &self.observed_widths);

                true
            }
            Msg::ColumnSizeReset(col_idx) => {
                self.state.set_width(col_idx, None);
                true
            }
            Msg::ColumnSizeChange(col_num, width) => {
                self.observed_widths.resize((col_num + 1).max(self.observed_widths.len()), None);
                self.observed_widths[col_num] = Some(width as usize);

                let observed_widths: Vec<usize> = self.observed_widths.iter()
                    .filter_map(|w| w.clone())
                    .collect();

                if self.state.columns().len() == observed_widths.len() {
                    if let Some(on_size_change) = props.on_size_change.clone() {
                        // use timeout to reduce the number of on_size_change callbacks
                        self.timeout = Some(Timeout::new(1, move || {
                            on_size_change.emit(observed_widths);
                        }));
                    }
                }
                true
            }
            Msg::ColumnSortChange(cell_idx, ctrl_key, opt_order) => {
                if ctrl_key {
                    self.state.add_column_sorter(cell_idx, opt_order);
                } else {
                    self.state.set_column_sorter(cell_idx, opt_order);
                }
                if let Some(on_sort_change) = &props.on_sort_change {
                    let sorter = self.state.create_combined_sorter_fn();
                    on_sort_change.emit(sorter);
                }
                true
            }
            Msg::HideClick(cell_idx) => {
                self.state.toggle_hidden(cell_idx);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let mut rows = Vec::new();

        header_list_to_rows(
            &self.state,
            props.headers.as_ref(),
            props,
            ctx.link(),
            0,
            &mut rows,
        );

        let rows: Vec<Html> = rows.into_iter().map(|row| row.into_iter()).flatten().collect();

        let column_count = self.state.column_count();

        // add some space at the end to make room for the tables vertical scrollbar
        let last = Container::new()
            .attribute("style", format!("grid-row: 1 / 10; grid-column-start: {};", column_count + 1));

        Container::new()
            .tag("table")
            .attribute("role", "grid")
            .attribute("aria-label", "table header")
            .node_ref(self.node_ref.clone())
            .class("pwt-d-grid")
            .attribute("style", self.compute_grid_style())
            .children(rows)
            .with_child(last)
            .onkeydown({
                let inner_ref =  self.node_ref.clone();
                move |event: KeyboardEvent| {
                    match event.key_code() {
                        39 => { // left
                            focus_next_tabable(&inner_ref, false, true);
                        }
                        37 => { // right
                            focus_next_tabable(&inner_ref, true, true);
                        }
                        _ => return,
                    }
                    event.prevent_default();
                }
            })
            .into()
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            init_roving_tabindex(&self.node_ref);
        }
    }
}

impl<T: 'static> Into<VNode> for DataTableHeader<T> {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtDataTableHeader<T>>(Rc::new(self), key);
        VNode::from(comp)
    }
}
