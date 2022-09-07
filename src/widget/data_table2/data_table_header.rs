use std::rc::Rc;

use derivative::Derivative;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::{IntoPropValue, Scope};

use crate::prelude::*;
use crate::widget::{Container, Row};

use super::{DataTableColumn, Header, HeaderGroup, ResizableHeader};


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

    //#[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    //pub columns: Rc<Vec<DataTableColumn<T>>>,

    headers: Rc<Vec<Header<T>>>,
}


impl<T: 'static> DataTableHeader<T> {

    /// Create a new instance.
    pub fn  new(headers: Rc<Vec<Header<T>>>) -> Self {
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
}

pub enum Msg {
    ResizeColumn(usize, usize),
    ColumnSizeReset(usize),
}

pub struct PwtDataTableHeader<T: 'static> {
    node_ref: NodeRef,
    columns: Vec<DataTableColumn<T>>,
    column_widths: Vec<Option<usize>>, // for column resize
}

impl <T: 'static> PwtDataTableHeader<T> {

    fn comput_grid_style(&self) -> String {

       let mut grid_style = format!("display:grid; grid-template-columns: ");

        for (col_idx, column) in self.columns.iter().enumerate() {
            if let Some(Some(width)) = self.column_widths.get(col_idx) {
                grid_style.push_str(&format!("{}px", width));
            } else {
                grid_style.push_str(&column.width);
            }
            grid_style.push(' ');
        }
        grid_style.push(';');

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

        Self {
            node_ref: props.node_ref.clone().unwrap_or(NodeRef::default()),
            columns,
            column_widths: Vec::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        //let props = ctx.props();
        match msg {
            Msg::ResizeColumn(col_num, width) => {
                //log::info!("resize col {} to {}", col_num, width);
                self.column_widths.resize((col_num + 1).max(self.column_widths.len()), None);
                self.column_widths[col_num] = Some(width.max(40));
                true
            }
            Msg::ColumnSizeReset(col_num) => {
                if let Some(elem) = self.column_widths.get_mut(col_num) {
                    *elem = None;
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

        header_to_rows(&header, ctx.link(), 0, 0, &mut rows);

        let rows: Vec<Html> = rows.into_iter().map(|row| row.into_iter()).flatten().collect();

        Row::new()
            .node_ref(self.node_ref.clone())
            .class("pwt-datatable2-header")
            .class("pwt-justify-content-start")
            .attribute("tabindex", "0")
            .with_child(
                Container::new()
                    .class("pwt-d-grid")
                    .attribute("style", self.comput_grid_style())
                    .children(rows)
            )
            .into()
    }
}

impl<T: 'static> Into<VNode> for DataTableHeader<T> {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtDataTableHeader<T>>(Rc::new(self), key);
        VNode::from(comp)
    }
}
