use std::rc::Rc;

use derivative::Derivative;

use gloo_timers::callback::Timeout;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::{IntoPropValue, IntoEventCallback, Scope};

use crate::prelude::*;
use crate::widget::Container;
use crate::widget::focus::{focus_next_tabable, init_roving_tabindex};

use super::{DataTableColumn, DataTableColumnWidth, Header, HeaderGroup, ResizableHeader};

fn header_to_rows<T: 'static>(
    header: &Header<T>,
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
            rows[start_row].push(
                Container::new()
                    .tag("th")
                    .attribute(
                        "style",
                        format!("grid-row: {} / 10;grid-column-start: {}", start_row + 1, start_col + 1)
                    )
                    .with_child(
                        ResizableHeader::new()
                            .content(html!{{&column.name}})
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
                    .into()
            );
            1
        }
        Header::Group(group) => group_to_rows(group, link, start_row, start_col, rows),
    }
}


fn group_to_rows<T: 'static>(
    group: &HeaderGroup<T>,
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
        span += header_to_rows(child, link, child_start_row, start_col + span, rows);
    }

    if let Some(content) = group.content.clone() {
        if span == 0 { span = 1; }
        rows[start_row].push(
            Container::new()
                .tag("th")
                .class("pwt-datatable2-group-header-item")
                .attribute("style", format!("grid-column: {} / span {}", start_col + 1, span))
                .with_child(content)
                .into()
        );
    }

    span
}

#[derive(Properties)]
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct DataTableHeader<T: 'static> {
    pub node_ref: Option<NodeRef>,
    pub key: Option<Key>,

    pub parent_width: usize,

    headers: Rc<Vec<Header<T>>>,

    on_size_change: Option<Callback<Vec<usize>>>,
}


impl<T: 'static> DataTableHeader<T> {

    /// Create a new instance.
    pub fn new(parent_width: usize, headers: Rc<Vec<Header<T>>>) -> Self {
        yew::props!(Self { parent_width,  headers })
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
}


pub enum Msg {
    ResizeColumn(usize, usize),
    ColumnSizeReset(usize),
    ColumnSizeChange(usize, i32),
}

pub struct PwtDataTableHeader<T: 'static> {
    node_ref: NodeRef,
    columns: Vec<DataTableColumn<T>>,

    column_widths: Vec<Option<usize>>, // for column resize
    observed_widths: Vec<Option<usize>>,
    initial_widths: Vec<Option<usize>>, // store first observed widths

    real_widths: Vec<usize>,

    timeout: Option<Timeout>,
}

static RESERVED_SPACE: usize = 20;

fn compute_column_widths<T: 'static>(
    columns: &[DataTableColumn<T>],
    column_widths: &[Option<usize>],
    initial_widths: &[Option<usize>],
    parent_width: usize,
) -> Vec<usize> {

    let parent_width = parent_width.saturating_sub(RESERVED_SPACE); // reserve space

    let mut widths = vec![0; columns.len()];
    let mut done = vec![false; columns.len()];

    let min_width = 40;

    let mut flex_count = 0;

    for (col_num, column) in columns.iter().enumerate() {
        if done[col_num] { continue; }
        if let Some(Some(w)) = column_widths.get(col_num) {
            widths[col_num] = (*w).max(min_width);
            done[col_num] = true;
            continue;
        }
        match column.width {
            DataTableColumnWidth::Auto => {
                widths[col_num] = initial_widths
                    .get(col_num)
                    .map(|w| match w { Some(w) => *w + 1, None => 0 })
                    .unwrap_or(0);
                done[col_num] = true;
                continue;
            }
            DataTableColumnWidth::Fixed(w) => {
                widths[col_num] = w.max(min_width);
                done[col_num] = true;
                continue;
            }
            DataTableColumnWidth::Flex(flex) => {
                flex_count += flex;
            }
        }
    }

    if flex_count > 0 {
        let used_width = widths.iter().fold(0, |mut sum, w| { sum += w; sum });
        let rest = parent_width.saturating_sub(used_width);
        let flex_unit: f64 = (rest as f64)/(flex_count as f64);

        for (col_num, column) in columns.iter().enumerate() {
            if done[col_num] { continue; }

            if let DataTableColumnWidth::Flex(flex) = column.width {
                let w = (flex as f64) * flex_unit;
                widths[col_num] = (w as usize).max(min_width);
                done[col_num] = true;
            }
        }
    }

    let used_width = widths.iter().fold(0, |mut sum, w| { sum += w; sum });
    let rest = parent_width.saturating_sub(used_width);

    if rest > 0 && !columns.is_empty() {
        //log::info!("REST {}", rest);
        for (col_num, column) in columns.iter().enumerate() {
            if column_widths.get(col_num).is_some() { continue; }
            match column.width {
                DataTableColumnWidth::Flex(_flex) => {
                    widths[columns.len() - 1] += rest;
                    break;
                }
                _ => {}
            }
        }
    }

    widths
}

impl <T: 'static> PwtDataTableHeader<T> {

    fn resize_columns(&mut self, props: &DataTableHeader<T>) {
        self.real_widths = compute_column_widths(
            &self.columns,
            &self.column_widths,
            &self.initial_widths,
            props.parent_width,
        );
    }

    fn compute_grid_style(&self, parent_width: usize) -> String {

        let mut width: usize = self.real_widths.iter().fold(0, |mut sum, w| { sum += w; sum });
        width += RESERVED_SPACE;

        let fill = parent_width.saturating_sub(width);

        let mut grid_style = format!(
            "user-select: none; width:{}px; display:grid; grid-template-columns:",
            width + fill,
        );

        for (col_idx, column) in self.columns.iter().enumerate() {
            let width = self.real_widths[col_idx];
            grid_style.push(' ');
            if let DataTableColumnWidth::Auto = column.width {
                if width == 0 {
                    grid_style.push_str("auto");
                    continue;
                }
            }
            grid_style.push_str(&format!("{}px", width));
        }

        grid_style.push_str(&format!(" {}px;", RESERVED_SPACE + fill));

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
        let initial_widths = Vec::new();
        let real_widths = compute_column_widths(&columns, &column_widths, &initial_widths, props.parent_width);

        Self {
            node_ref: props.node_ref.clone().unwrap_or(NodeRef::default()),
            columns,
            real_widths,
            column_widths,
            initial_widths,
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
                        if let DataTableColumnWidth::Flex(_flex) = self.columns[i].width {
                            if let Some(Some(observed_width)) = self.observed_widths.get(i) {
                                self.column_widths[i] = Some(*observed_width + 1);
                            }
                        }
                    }
                }

                self.resize_columns(props);
                true
            }
            Msg::ColumnSizeReset(col_num) => {
                if let Some(elem) = self.column_widths.get_mut(col_num) {
                    *elem = None;
                    self.resize_columns(props);
                    true
                } else {
                    false
                }
            }
            Msg::ColumnSizeChange(col_num, width) => {
                self.observed_widths.resize((col_num + 1).max(self.observed_widths.len()), None);
                self.observed_widths[col_num] = Some(width as usize);
                self.initial_widths.resize((col_num + 1).max(self.initial_widths.len()), None);
                if let Some(opt_elem) = self.initial_widths.get_mut(col_num) {
                    if opt_elem.is_none() {
                        // + 1 pixel to avoid rounding problems
                        // Note: Element sizes are float, but we only get i32 from SizeObserver
                        *opt_elem = Some((width + 1) as usize);
                    }

                }
                //log::info!("COL {} SIZE CHANGE {} --- {}", col_num, width, self.observed_widths.len());

                let observed_widths: Vec<usize> = self.observed_widths.iter().filter_map(|w| w.clone()).collect();
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

        let column_count = header_to_rows(&header, ctx.link(), 0, 0, &mut rows);

        let rows: Vec<Html> = rows.into_iter().map(|row| row.into_iter()).flatten().collect();

        // add some space at the end to make room for the tables vertical scrollbar
        let last = Container::new()
            .attribute("style", format!("grid-row: 1 / 10; grid-column-start: {};", column_count + 1));

        Container::new()
            .tag("table")
            .class("pwt-d-grid")
            .class("pwt-datatable2-header")
            .attribute("style", self.compute_grid_style(props.parent_width))
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
        if props.parent_width != old_props.parent_width {
            log::info!("WIDTH CHANGE");
            self.resize_columns(props);
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
