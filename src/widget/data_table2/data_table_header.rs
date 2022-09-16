use std::rc::Rc;

use derivative::Derivative;

use gloo_timers::callback::Timeout;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::{IntoPropValue, IntoEventCallback, Scope};

use crate::prelude::*;
use crate::widget::{Container, Fa};
use crate::widget::focus::{focus_next_tabable, init_roving_tabindex};

use super::{DataTableColumn, Header, HeaderGroup, ResizableHeader};

#[derive(Properties)]
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct DataTableHeader<T: 'static> {
    pub node_ref: Option<NodeRef>,
    pub key: Option<Key>,

    headers: Rc<Vec<Header<T>>>,

    pub on_size_change: Option<Callback<Vec<usize>>>,
    pub on_sort_change: Option<Callback<(usize, bool)>>,

    /// set class for header cells
    #[prop_or_default]
    pub header_class: Classes,

    /// Sort order state for columns.
    sorters: Vec<(usize, bool)>,
}


impl<T: 'static> DataTableHeader<T> {

    /// Create a new instance.
    pub fn new(headers: Rc<Vec<Header<T>>>, sorters: &[(usize, bool)]) -> Self {
        let sorters: Vec<(usize, bool)> = sorters.into();
        yew::props!(Self { headers, sorters })
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
    pub fn on_sort_change(mut self, cb: impl IntoEventCallback<(usize, bool)>) -> Self {
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
    ColumnSizeChange(usize, i32),
}

fn header_to_rows<T: 'static>(
    header: &Header<T>,
    props: &DataTableHeader<T>,
    link: &Scope<PwtDataTableHeader<T>>,
    start_row: usize,
    start_col: usize,
    rows: &mut Vec<Vec<Html>>,
) -> usize {
    loop {
        if rows.len() < (start_row + 1) {
            rows.push(Vec::new());
        } else {
            break;
        }
    }
    match header {
        Header::Single(column) => {
            let sort_order = props.sorters.iter().find_map(|(idx, asc)| {
                (*idx == start_col).then(|| *asc)
            });

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
                            .content(html!{<>{sort_icon}{&column.name}</>})
                            .on_resize({
                                let link = link.clone();
                                move |width| {
                                    let width: usize = if width > 0 { width as usize } else  { 0 };
                                    link.send_message(Msg::ResizeColumn(start_col, width));
                                }
                            })
                            .on_size_reset(link.callback(move |_| Msg::ColumnSizeReset(start_col)))
                            .on_size_change(link.callback(move |w| Msg::ColumnSizeChange(start_col, w)))
                    )
                    .ondblclick({
                        let on_sort_change = props.on_sort_change.clone();
                        move |event: MouseEvent| {
                            if let Some(on_sort_change) = &on_sort_change {
                                on_sort_change.emit((start_col, event.ctrl_key()));
                            }
                        }
                    })
                    .into()
            );
            1
        }
        Header::Group(group) => group_to_rows(group, props, link, start_row, start_col, rows),
    }
}


fn group_to_rows<T: 'static>(
    group: &HeaderGroup<T>,
    props: &DataTableHeader<T>,
    link: &Scope<PwtDataTableHeader<T>>,
    start_row: usize,
    start_col: usize,
    rows: &mut Vec<Vec<Html>>,
) -> usize {
    loop {
        if rows.len() < (start_row + 1) {
            rows.push(Vec::new());
        } else {
            break;
        }
    }

    let child_start_row =  if group.content.is_some() { start_row + 1 } else { start_row };
    let mut span = 0;
    for child in &group.children {
        span += header_to_rows(child, props, link, child_start_row, start_col + span, rows);
    }

    if let Some(content) = group.content.clone() {
        if span == 0 { span = 1; }
        rows[start_row].push(
            Container::new()
                .tag("th")
            //.attribute("role", "group")
                .attribute("role", "columnheader")
                .class("pwt-datatable2-group-header-item")
                .class(props.header_class.clone())
                .attribute("style", format!("grid-column: {} / span {}", start_col + 1, span))
                .with_child(content)
                .into()
        );
    }

    span
}

pub struct PwtDataTableHeader<T: 'static> {
    node_ref: NodeRef,
    columns: Vec<DataTableColumn<T>>,

    column_widths: Vec<Option<usize>>, // for column resize
    observed_widths: Vec<Option<usize>>,

    timeout: Option<Timeout>,
}

static RESERVED_SPACE: usize = 20;

impl <T: 'static> PwtDataTableHeader<T> {

    fn compute_grid_style(&self) -> String {

        let mut grid_style = String::from("user-select: none; display:grid; grid-template-columns:");
        for (col_idx, column) in self.columns.iter().enumerate() {
            if let Some(Some(width)) = self.column_widths.get(col_idx) {
                grid_style.push_str(&format!("{}px", width));
            } else {
                grid_style.push_str(&column.width);
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
        let mut columns = Vec::new();

        for header in props.headers.iter() {
            header.extract_column_list(&mut columns);
        }

        let column_widths = Vec::new();

        Self {
            node_ref: props.node_ref.clone().unwrap_or(NodeRef::default()),
            columns,
            column_widths,
            observed_widths: Vec::new(),
            timeout: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::ResizeColumn(col_num, width) => {
                log::info!("resize col {} to {}", col_num, width);
                self.column_widths.resize((col_num + 1).max(self.column_widths.len()), None);
                self.column_widths[col_num] = Some(width.max(40));

                // Set flex columns on the left to fixed size to avoid unexpected effects.
                for i in 0..col_num {
                    if self.column_widths[i].is_none() {
                        if self.columns[i].width.contains("fr") { // flex columns
                            if let Some(Some(observed_width)) = self.observed_widths.get(i) {
                                self.column_widths[i] = Some(*observed_width + 1);
                            }
                        }
                    }
                }
                true
            }
            Msg::ColumnSizeReset(col_num) => {
                if let Some(elem) = self.column_widths.get_mut(col_num) {
                    *elem = None;
                     true
                } else {
                    false
                }
            }
            Msg::ColumnSizeChange(col_num, width) => {
                self.observed_widths.resize((col_num + 1).max(self.observed_widths.len()), None);
                self.observed_widths[col_num] = Some(width as usize);

                let observed_widths: Vec<usize> = self.observed_widths.iter()
                    .filter_map(|w| w.clone())
                    .collect();

                if self.columns.len() == observed_widths.len() {
                    if let Some(on_size_change) = props.on_size_change.clone() {
                        // use timeout to reduce the number of on_size_change callbacks
                        self.timeout = Some(Timeout::new(1, move || {
                            on_size_change.emit(observed_widths);
                        }));
                    }
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        //let grid_size = props.grid_size();
        //log::info!("GRID SIZE {:?}", grid_size);

        let mut rows = Vec::new();

        let header = HeaderGroup::new().children(props.headers.as_ref().clone()).into();

        let column_count = header_to_rows(&header, props, ctx.link(), 0, 0, &mut rows);

        let rows: Vec<Html> = rows.into_iter().map(|row| row.into_iter()).flatten().collect();

        // add some space at the end to make room for the tables vertical scrollbar
        let last = Container::new()
            .attribute("style", format!("grid-row: 1 / 10; grid-column-start: {};", column_count + 1));

        Container::new()
            .tag("table")
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

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        if props.sorters != old_props.sorters {
            return true;
        }

        false
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
