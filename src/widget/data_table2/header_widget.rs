use std::rc::Rc;

use derivative::Derivative;

use gloo_timers::callback::Timeout;
use wasm_bindgen::JsCast;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::{IntoEventCallback, Scope};

use crate::prelude::*;
use crate::props::SorterFn;
use crate::widget::{get_unique_element_id, Container, Fa, Menu, MenuEvent, MenuItem, MenuCheckbox};

use super::{
    IndexedHeader, IndexedHeaderSingle, IndexedHeaderGroup,
    HeaderState, ResizableHeader,
};

#[derive(Properties)]
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
#[doc(hidden)] // only used inside this crate
pub struct HeaderWidget<T: 'static> {
    pub node_ref: Option<NodeRef>,
    pub key: Option<Key>,

    headers: Rc<Vec<IndexedHeader<T>>>,

    pub on_size_change: Option<Callback<Vec<f64>>>,
    pub on_hidden_change: Option<Callback<Vec<bool>>>,
    pub on_sort_change: Option<Callback<SorterFn<T>>>,

    /// set class for header cells
    #[prop_or_default]
    pub header_class: Classes,
}


impl<T: 'static> HeaderWidget<T> {

    /// Create a new instance.
    pub fn new(headers: Rc<Vec<IndexedHeader<T>>>) -> Self {
        yew::props!(Self { headers })
    }

    /*
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Builder style method to set the yew `node_ref`
    pub fn node_ref(mut self, node_ref: impl IntoPropValue<Option<NodeRef>>) -> Self {
        self.node_ref = node_ref.into_prop_value();
        self
    }
     */

    /// Builder style method to set the size change callback
    pub fn on_size_change(mut self, cb: impl IntoEventCallback<Vec<f64>>) -> Self {
        self.on_size_change = cb.into_event_callback();
        self
    }

    /// Builder style method to set the hidden change callback
    pub fn on_hidden_change(mut self, cb: impl IntoEventCallback<Vec<bool>>) -> Self {
        self.on_hidden_change = cb.into_event_callback();
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
    ResizeColumn(usize, f64),
    ColumnSizeReset(usize),
    ColumnSizeChange(usize, f64),
    ColumnSortChange(usize, bool, Option<bool>),
    HideClick(usize, bool),
    MoveCursor(bool),
    FocusCell(usize),
}

pub struct PwtHeaderWidget<T: 'static> {
    node_ref: NodeRef,

    unique_id: String,

    // Sort order state for columns.
    state: HeaderState<T>,

    // Active cell
    cursor: Option<usize>,

    observed_widths: Vec<Option<f64>>,

    timeout: Option<Timeout>,
}

static RESERVED_SPACE: usize = 20;

impl <T: 'static> PwtHeaderWidget<T> {

    fn compute_grid_style(&self) -> String {

        let mut grid_style = String::from("user-select: none; display:grid; grid-template-columns:");
        for (col_idx, cell) in self.state.columns().iter().enumerate() {
            if self.state.get_column_hidden(col_idx) { continue; }
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


    fn header_list_to_rows(
        &self,
        list: &[IndexedHeader<T>],
        props: &HeaderWidget<T>,
        link: &Scope<PwtHeaderWidget<T>>,
        start_row: usize,
        start_col: usize,
        rows: &mut Vec<Vec<Html>>,
    ) -> usize {
        let mut span = 0;

        for child in list {

            let cell_idx = child.cell_idx();
            let hidden = self.state.get_cell_hidden(cell_idx);
            if hidden { continue; }

            match child {
                IndexedHeader::Single(column) => {
                    self.column_to_rows(column, props, link, start_row, start_col + span, rows);
                    span += 1;
                }
                IndexedHeader::Group(group) => {
                    let cols = self.group_to_rows(group, props, link, start_row, start_col + span, rows);
                    span += cols;
                }
            }
        }

        span
    }

    fn unique_cell_id(&self, cell_idx: usize) -> String {
        format!("{}-cell-{}", self.unique_id, cell_idx)
    }

    fn column_to_rows(
        &self,
        cell: &IndexedHeaderSingle<T>,
        props: &HeaderWidget<T>,
        link: &Scope<PwtHeaderWidget<T>>,
        start_row: usize,
        start_col: usize,
        rows: &mut Vec<Vec<Html>>,
    ) {
        rows.resize((start_row + 1).max(rows.len()), Vec::new());

        let column_idx = cell.start_col;
        let cell_idx = cell.cell_idx;
        let active = self.cursor.map(|cursor| cursor == cell_idx).unwrap_or(false);
        let tabindex = if active || (self.cursor.is_none() && (cell_idx == 0)) { 0 } else { -1 };

        let unique_id = self.unique_cell_id(cell_idx);

        let sort_order = self.state.get_column_sorter(cell_idx);
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

        // reserve some space for the sort icon
        let sort_space = match sort_order {
            None => html!{"\u{00a0}\u{00a0}"},
            Some(_) => html!{},
        };

        rows[start_row].push(
            Container::new()
                .key(Key::from(cell_idx))
                .tag("th")
                .attribute("role", "columnheader")
                .attribute(
                    "style",
                    format!("grid-row: {} / 10;grid-column-start: {}", start_row + 1, start_col + 1)
                )
                .with_child(
                    ResizableHeader::new()
                        .id(unique_id)
                        .tabindex(tabindex)
                        .class(props.header_class.clone())
                        .class("pwt-w-100 pwt-h-100")
                        .content(html!{<>{sort_icon}{&cell.column.name}{sort_space}</>})
                        .on_resize(link.callback(move |width: f64| Msg::ResizeColumn(column_idx, width.max(0.0))))
                        .on_size_reset(link.callback(move |_| Msg::ColumnSizeReset(column_idx)))
                        .on_size_change(link.callback(move |w| Msg::ColumnSizeChange(column_idx, w)))
                        .menu_builder({
                            let headers = Rc::clone(&props.headers);
                            let link = link.clone();
                            let hidden_cells = Vec::from(self.state.hidden_cells());
                            move || build_header_menu(&headers, &link, cell_idx, &hidden_cells)
                        })
                )
                .onfocusin(link.callback(move |_| Msg::FocusCell(cell_idx)))
                .ondblclick(link.callback(move |event: MouseEvent| {
                    Msg::ColumnSortChange(cell_idx, event.ctrl_key(), None)
                }))
                .onkeydown({
                    let link = link.clone();
                    move |event: KeyboardEvent| {
                        match event.key_code() {
                            13 => {
                                link.send_message(Msg::ColumnSortChange(cell_idx, event.ctrl_key(), None));
                                event.prevent_default();
                            }
                            _ => {},
                        };
                    }
                })
                .into()
        );
    }

    fn group_to_rows(
        &self,
        group: &IndexedHeaderGroup<T>,
        props: &HeaderWidget<T>,
        link: &Scope<PwtHeaderWidget<T>>,
        start_row: usize,
        start_col: usize,
        rows: &mut Vec<Vec<Html>>,
    ) -> usize {
        rows.resize((start_row + 1).max(rows.len()), Vec::new());

        let cell_idx = group.cell_idx;
        let active = self.cursor.map(|cursor| cursor == cell_idx).unwrap_or(false);
        let tabindex = if active || (self.cursor.is_none() && (cell_idx == 0)) { "0" } else { "-1" };
        let unique_id = self.unique_cell_id(cell_idx);

        let span = self.header_list_to_rows(&group.children, props, link, start_row + 1, start_col, rows);
        let span = span.max(1); // at least one column for the group header

        rows[start_row].push(
            Container::new()
                .tag("th")
                .key(Key::from(cell_idx))
                .attribute("role", "columnheader")
                .attribute("tabindex", tabindex)
                .attribute("id", unique_id)
                .class("pwt-datatable2-group-header-item")
                .class(props.header_class.clone())
                .attribute("style", format!(
                    "grid-column: {} / span {}",
                    start_col + 1,
                    span,
                ))
                .with_child(group.name.clone())
                .into()
        );
        span
    }

    fn focus_active_cell(&self) {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();

        let get_cell_el = |cell_idx| -> Option<web_sys::HtmlElement> {
            let id = self.unique_cell_id(cell_idx);
            let el = match document.get_element_by_id(&id) {
                Some(el) => el,
                None => return None,
            };
            match el.dyn_into::<web_sys::HtmlElement>() {
                Ok(el) => Some(el),
                Err(_) => None,
            }
        };

        for cell_idx in 0..self.state.cell_count() {
            if let Some(el) = get_cell_el(cell_idx) {
                el.set_tab_index(-1);
            }
        }

        let cell_idx = match self.cursor {
            Some(cursor) => cursor,
            None => return,
        };

        let el = match get_cell_el(cell_idx) {
            Some(el) => el,
            None => return,
        };

        el.set_tab_index(0);
        let _ = el.focus();
    }
}

impl <T: 'static> Component for PwtHeaderWidget<T> {
    type Message = Msg;
    type Properties = HeaderWidget<T>;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let state = HeaderState::new(Rc::clone(&props.headers));

        if let Some(on_sort_change) = &props.on_sort_change {
            let sorter = state.create_combined_sorter_fn();
            on_sort_change.emit(sorter);
        }

        let mut observed_widths = Vec::new();
        for (col_idx, cell) in state.columns().iter().enumerate() {
            observed_widths.push(if state.get_column_hidden(col_idx) { Some(0.) } else { None });
        }

        Self {
            unique_id: get_unique_element_id(),
            node_ref: props.node_ref.clone().unwrap_or(NodeRef::default()),
            state,
            cursor: None,
            observed_widths,
            timeout: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::ResizeColumn(col_idx, width) => {
                self.state.set_width(col_idx, Some(width.max(40.0)));

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
                self.observed_widths[col_num] = Some(width);

                let observed_widths: Vec<f64> = self.observed_widths.iter()
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
            Msg::HideClick(cell_idx, visible) => {
                self.state.set_hidden(cell_idx, !visible);
                if let Some(on_hidden_change) = &props.on_hidden_change {
                    on_hidden_change.emit(self.state.hidden_columns());
                }
                true
            }
            Msg::FocusCell(cell_idx) => {
                self.cursor = Some(cell_idx);
                true
            }
            Msg::MoveCursor(direction) => {
                let last = self.state.cell_count().saturating_sub(1);
                let cursor = match self.cursor {
                    Some(cursor) => cursor,
                    None => return false,
                };
                self.cursor = Some(match direction {
                    false => if cursor > 0 { cursor - 1 }  else { last },
                    true => if (cursor + 1) <= last { cursor + 1 } else { 0 },
                });
                self.focus_active_cell();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let mut rows = Vec::new();

        self.header_list_to_rows(
            props.headers.as_ref(),
            props,
            ctx.link(),
            0,
            0,
            &mut rows,
        );

        let rows: Vec<Html> = rows.into_iter().map(|row| row.into_iter()).flatten().collect();

        let column_count = self.state.column_count();

        // add some space at the end to make room for the tables vertical scrollbar
        let last = Container::new()
            .key(Key::from("last")) // important: all children need a key
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
                let link = ctx.link().clone();
                move |event: KeyboardEvent| {
                    match event.key_code() {
                        39 => link.send_message(Msg::MoveCursor(true)),
                        37 => link.send_message(Msg::MoveCursor(false)),
                        _ => return,
                    }
                    event.prevent_default();
                }
            })
            .into()
    }
}

fn build_header_menu<T>(
    headers: &[IndexedHeader<T>],
    link: &Scope<PwtHeaderWidget<T>>,
    cell_idx: usize,
    hidden_cells: &[bool],
) -> Menu {

    let mut columns_menu = Menu::new();
    headers_to_menu(&mut columns_menu, 0, headers, link, &mut 0, hidden_cells);

    Menu::new()
        .with_item(
            MenuItem::new("Sort Ascending")
                .icon_class("fa fa-long-arrow-up")
                .on_select(link.callback(move |_| Msg::ColumnSortChange(cell_idx, false, Some(true))))
        )
        .with_item(
            MenuItem::new("Sort Descending")
                .icon_class("fa fa-long-arrow-down")
                .on_select(link.callback(move |_| Msg::ColumnSortChange(cell_idx, false, Some(false))))
        )
        .with_separator()
        .with_item(
            MenuItem::new("Columns")
                .menu(columns_menu)
        )
}

fn headers_to_menu<T>(
    menu: &mut Menu,
    indent_level: usize,
    headers: &[IndexedHeader<T>],
    link: &Scope<PwtHeaderWidget<T>>,
    cell_idx: &mut usize,
    hidden_cells: &[bool],
) {
    let indent: Html = (0..indent_level)
        .map(|_| html!{ <span aria-hidden="" class="pwt-ps-4"/> })
        .collect();

    for header in headers {
        let on_change = {
            let cell_idx = *cell_idx;
            link.callback(move |event: MenuEvent| {
                event.keep_open(true);
                Msg::HideClick(cell_idx, event.checked)
            })
        };

        match header {
            IndexedHeader::Single(cell) => {
                let label = html!{<>{indent.clone()}{cell.column.name.clone()}</>};
                menu.add_item(
                    MenuCheckbox::new(label)
                        .checked(!hidden_cells[*cell_idx])
                        .on_change(on_change)
                );
                *cell_idx += 1;
            }
            IndexedHeader::Group(group) => {
                let label = html!{<>{indent.clone()}{group.name.clone()}</>};
                menu.add_item(
                    MenuCheckbox::new(label)
                        .checked(!hidden_cells[*cell_idx])
                        .on_change(on_change)
                );
                *cell_idx += 1;
                headers_to_menu(menu, indent_level + 1, &group.children, link, cell_idx, hidden_cells);
            }
        }
    }
}

/*
fn headers_to_menu_old<T>(
    headers: &[IndexedHeader<T>],
    link: &Scope<PwtHeaderWidget<T>>,
    cell_idx: &mut usize,
    hidden_cells: &[bool],
) -> Menu {

    let mut menu = Menu::new();

    for header in headers {
        match header {
            IndexedHeader::Single(cell) => {
                menu.add_item(
                    MenuCheckbox::new(cell.column.name.clone())
                        .checked(!hidden_cells[*cell_idx])
                        .on_change(link.callback({
                            let cell_idx = *cell_idx;
                            move |checked| Msg::HideClick(cell_idx, checked)
                        }))
                );
                *cell_idx += 1;
            }
            IndexedHeader::Group(group) => {
                let mut item = MenuItem::new(group.name.clone());
                *cell_idx += 1;
                item.set_menu(headers_to_menu_old(&group.children, link, cell_idx, hidden_cells));
                menu.add_item(item);
             }
         }
     }
    menu
}
 */

impl<T: 'static> Into<VNode> for HeaderWidget<T> {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtHeaderWidget<T>>(Rc::new(self), key);
        VNode::from(comp)
    }
}
