use std::rc::Rc;

use derivative::Derivative;
use indexmap::IndexMap;

use gloo_timers::callback::Timeout;
use wasm_bindgen::JsCast;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::Scope;

use crate::prelude::*;
use crate::widget::{get_unique_element_id, Container, Fa};
use crate::widget::menu::{Menu, MenuEvent, MenuItem, MenuCheckbox};

use super::{
    IndexedHeader, IndexedHeaderSingle, IndexedHeaderGroup,
    HeaderState, ResizableHeader, HeaderMsg, DataTableHeaderRenderArgs,
    RowSelectionStatus, DataTableHeaderTableLink, DataTableHeaderKeyboardEvent,
};

#[derive(Properties)]
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
#[doc(hidden)] // only used inside this crate
pub struct HeaderWidget<T: 'static> {
    pub node_ref: Option<NodeRef>,
    pub key: Option<Key>,

    headers: Rc<Vec<IndexedHeader<T>>>,

    on_message: Callback<HeaderMsg<T>>,

    /// set class for header cells
    #[prop_or_default]
    pub header_class: Classes,

    #[prop_or(RowSelectionStatus::Nothing)]
    pub selection_status: RowSelectionStatus,

    /// Allow the header to take focus.
    #[prop_or(true)]
    pub focusable: bool,

    reserve_scroll_space: usize,
}

impl<T: 'static> HeaderWidget<T> {
    /// Create a new instance.
    pub fn new(headers: Rc<Vec<IndexedHeader<T>>>, on_message: Callback<HeaderMsg<T>>) -> Self {
        yew::props!(Self {
            headers,
            on_message,
            reserve_scroll_space: 0,
        })
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

    /// Builder style method to set the focusable flag.
    pub fn focusable(mut self, focusable: bool) -> Self {
        self.set_focusable(focusable);
        self
    }

    /// Method  to set the focusable flag.
    pub fn set_focusable(&mut self, focusable: bool) {
        self.focusable = focusable;
    }

    /// Builder style method to set the row selection status.
    pub fn selection_status(mut self, status: RowSelectionStatus) -> Self {
        self.selection_status = status;
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

    /// Builder style method to reserve scroll space.
    pub fn reserve_scroll_space(mut self, reserve_scroll_space: usize) -> Self {
        self.set_reserve_scroll_space(reserve_scroll_space);
        self
    }

    /// Method to set if space should be reserved for the scroller.
    pub fn set_reserve_scroll_space(&mut self, reserve_scroll_space: usize) {
        self.reserve_scroll_space = reserve_scroll_space;
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

impl <T: 'static> PwtHeaderWidget<T> {

    fn compute_grid_style(&self, ctx: &Context<Self>) -> String {

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

        let scrollbar_size = ctx.props().reserve_scroll_space;
        if scrollbar_size > 0 {
            grid_style.push_str(&format!(" {scrollbar_size}px;"));
        }

        grid_style
    }

    fn header_list_to_row(
        &self,
        list: &[IndexedHeader<T>],
        props: &HeaderWidget<T>,
        link: &Scope<PwtHeaderWidget<T>>,
        start_row: usize,
        start_col: usize,
        header_row: &mut Vec<Html>,
    ) -> usize {
        let mut span = 0;

        for child in list {

            let cell_idx = child.cell_idx();
            let hidden = self.state.get_cell_hidden(cell_idx);
            if hidden { continue; }

            match child {
                IndexedHeader::Single(column) => {
                    self.column_to_header_row(column, props, link, start_row, start_col + span, header_row);
                    span += 1;
                }
                IndexedHeader::Group(group) => {
                    let cols = self.group_to_header_row(group, props, link, start_row, start_col + span, header_row);
                    span += cols;
                }
            }
        }

        span
    }

    fn unique_cell_id(&self, cell_idx: usize) -> String {
        format!("{}-cell-{}", self.unique_id, cell_idx)
    }

    fn column_to_header_row(
        &self,
        cell: &IndexedHeaderSingle<T>,
        props: &HeaderWidget<T>,
        link: &Scope<PwtHeaderWidget<T>>,
        start_row: usize,
        start_col: usize,
        header_row: &mut Vec<Html>,
    ) {
        let column_idx = cell.start_col;
        let cell_idx = cell.cell_idx;
        let active = self.cursor.map(|cursor| cursor == cell_idx).unwrap_or(false);
        let tabindex = if active || (self.cursor.is_none() && (cell_idx == 0)) {
            AttrValue::Static("0")
        } else {
            AttrValue::Static("-1")
        };

        let unique_id = self.unique_cell_id(cell_idx);

        let sortable = cell.column.sorter.is_some();

        let sort_order = if sortable {
            self.state.get_column_sorter(cell_idx)
        } else {
            None
        };

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

        let aria_sort =  match sort_order {
            Some(true) => "ascending",
            Some(false) => "descending",
            None => "none",
        };

        // reserve some space for the sort icon
        let sort_space = match sort_order {
            None => html!{"\u{00a0}\u{00a0}"},
            Some(_) => html!{},
        };


        let mut attributes = IndexMap::new();
        let mut header_class = props.header_class.clone();
        let header_content = match &cell.column.render_header {
            Some(render_header) => {
                let mut args = DataTableHeaderRenderArgs {
                    column_index: column_idx,
                    selection_status: props.selection_status,
                    link: DataTableHeaderTableLink { on_message: props.on_message.clone() },
                    attributes,
                    class: header_class.clone(),
                };
                let content = render_header.apply(&mut args);
                header_class = args.class;
                attributes = args.attributes;
                content
            }
            None => html!{<>{sort_icon}{&cell.column.name}{sort_space}</>},
        };

        if props.focusable {
            attributes.insert(AttrValue::Static("tabindex"), tabindex);
        }
        attributes.insert(AttrValue::Static("aria-label"), cell.column.name.clone());

        header_row.push(
            Container::new()
                .key(Key::from(cell_idx))
                .tag("th")
                .attribute("role", "columnheader")
                .attribute("aria-sort", aria_sort)
                .attribute(
                    "style",
                    format!("grid-row: {} / 10;grid-column-start: {}", start_row + 1, start_col + 1)
                )
                .with_child(
                    ResizableHeader::new()
                        .id(unique_id)
                        .class(header_class.clone())
                        .class("pwt-w-100 pwt-h-100")
                        .attributes(attributes)
                        .content(header_content)
                        .resizable(cell.column.resizable)
                        .show_menu(cell.column.show_menu)
                        .on_resize(link.callback(move |width: f64| Msg::ResizeColumn(column_idx, width.max(0.0))))
                        .on_size_reset(link.callback(move |_| Msg::ColumnSizeReset(column_idx)))
                        .on_size_change(link.callback(move |w| Msg::ColumnSizeChange(column_idx, w)))
                        .menu_builder({
                            let headers = Rc::clone(&props.headers);
                            let link = link.clone();
                            let hidden_cells = self.state.hidden_cells();
                            move || build_header_menu(&headers, &link, cell_idx, &hidden_cells)
                        })
                )
                .onfocusin(link.callback(move |_| Msg::FocusCell(cell_idx)))
                .onclick({
                    let link = link.clone();
                    move |event: MouseEvent| {
                        if sortable {
                            link.send_message(Msg::ColumnSortChange(cell_idx, event.ctrl_key(), None));
                        }
                    }
                })
                .onkeydown({
                    let link = link.clone();
                    let on_header_keydown = cell.column.on_header_keydown.clone();
                    let on_message = props.on_message.clone();
                    move |event: KeyboardEvent| {
                        if let Some(on_header_keydown) = &on_header_keydown {
                            let mut arg =  DataTableHeaderKeyboardEvent {
                                inner: event.clone(),
                                stop_propagation: false,
                                on_message: on_message.clone(),
                            };
                            on_header_keydown.emit(&mut arg);
                            if arg.stop_propagation {
                                return;
                            }
                        }
                        if sortable && event.key().as_str() == "Enter" {
                            link.send_message(Msg::ColumnSortChange(cell_idx, event.ctrl_key(), None));
                            event.prevent_default();
                        }
                    }
                })
                .into()
        );
    }

    fn group_to_header_row(
        &self,
        group: &IndexedHeaderGroup<T>,
        props: &HeaderWidget<T>,
        link: &Scope<PwtHeaderWidget<T>>,
        start_row: usize,
        start_col: usize,
        header_row: &mut Vec<Html>,
    ) -> usize {
        let cell_idx = group.cell_idx;
        let active = self.cursor.map(|cursor| cursor == cell_idx).unwrap_or(false);
        let tabindex = if active || (self.cursor.is_none() && (cell_idx == 0)) { "0" } else { "-1" };
        let unique_id = self.unique_cell_id(cell_idx);

        let mut child_rows = Vec::new();

        let span = self.header_list_to_row(&group.children, props, link, start_row + 1, start_col, &mut child_rows);
        let span = span.max(1); // at least one column for the group header

        header_row.push(
            Container::new()
                .tag("th")
                .key(Key::from(cell_idx))
            // Note: ARIA has no notation for group headers. We need
            // to hide them to get correct column order.
                .attribute("role", "none")
                .attribute("aria-hidden", "true")
                .attribute("tabindex", props.focusable.then(|| tabindex))
                .attribute("id", unique_id)
                .class("pwt-datatable-group-header-item")
                .class(props.header_class.clone())
                .attribute("style", format!(
                    "grid-column: {} / span {}",
                    start_col + 1,
                    span,
                ))
                .with_child(group.name.clone())
                .into()
        );

        header_row.extend(child_rows);

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

        let sorter = state.create_combined_sorter_fn();
        props.on_message.emit(HeaderMsg::ChangeSort(sorter));

        let mut observed_widths = Vec::new();
        for (col_idx, _cell) in state.columns().iter().enumerate() {
            observed_widths.push(if state.get_column_hidden(col_idx) { Some(0.0) } else { None });
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
                self.state.set_width(col_idx, Some(width.max(50.0)));

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
                    let on_message = props.on_message.clone();
                    // use timeout to reduce the number of on_size_change callbacks
                    self.timeout = Some(Timeout::new(1, move || {
                        on_message.emit(HeaderMsg::ColumnWidthChange(observed_widths));
                    }));
                }
                true
            }
            Msg::ColumnSortChange(cell_idx, ctrl_key, opt_order) => {
                if ctrl_key {
                    self.state.add_column_sorter(cell_idx, opt_order);
                } else {
                    self.state.set_column_sorter(cell_idx, opt_order);
                }
                let sorter = self.state.create_combined_sorter_fn();
                props.on_message.emit(HeaderMsg::ChangeSort(sorter));
                true
            }
            Msg::HideClick(cell_idx, visible) => {
                self.state.set_hidden(cell_idx, !visible);
                props.on_message.emit(HeaderMsg::ColumnHiddenChange(self.state.hidden_columns()));
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

        let mut header_row = Vec::new();

        self.header_list_to_row(
            props.headers.as_ref(),
            props,
            ctx.link(),
            0,
            0,
            &mut header_row,
        );

        let column_count = self.state.column_count();

        // add some space at the end to make room for the tables vertical scrollbar
        let scrollbar_size = ctx.props().reserve_scroll_space;
        if scrollbar_size > 0 {
            header_row.push(
                Container::new()
                    .key(Key::from("last")) // important: all children need a key
                    .attribute(
                        "style",
                        format!("grid-row: 1 / 10; grid-column-start: {};", column_count + 1),
                    )
                    .into(),
            );
        }

        Container::new()
            .tag("table")
            .attribute("role", "row")
            .node_ref(self.node_ref.clone())
            .class("pwt-d-grid")
            .attribute("style", self.compute_grid_style(ctx))
            .children(header_row)
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

    let cell = IndexedHeader::lookup_cell(headers, cell_idx).unwrap();
    let column = match cell {
        IndexedHeader::Single(single) => &single.column,
        _ => panic!("unable to build header menu for group"),
    };
    let sortable = column.sorter.is_some();

    Menu::new()
        .with_item(
            MenuItem::new("Sort Ascending")
                .icon_class("fa fa-long-arrow-up")
                .disabled(!sortable)
                .on_select(link.callback(move |_| Msg::ColumnSortChange(cell_idx, false, Some(true))))
            )
        .with_item(
            MenuItem::new("Sort Descending")
                .icon_class("fa fa-long-arrow-down")
                .disabled(!sortable)
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

impl<T: 'static> Into<VNode> for HeaderWidget<T> {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtHeaderWidget<T>>(Rc::new(self), key);
        VNode::from(comp)
    }
}
