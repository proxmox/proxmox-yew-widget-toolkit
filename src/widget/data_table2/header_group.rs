use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::{IntoPropValue, Scope};

use crate::prelude::*;
use crate::widget::{Column, Container, Row};

use super::ResizableHeader;

#[derive(Clone, PartialEq)]
pub enum Header {
    Single(ResizableHeader),
    Group(HeaderGroup),
}

impl From<ResizableHeader> for Header {
    fn from(header: ResizableHeader) -> Self {
        Self::Single(header)
    }
}

impl From<HeaderGroup> for Header {
    fn from(group: HeaderGroup) -> Self {
        Self::Group(group)
    }
}

impl Header {

    fn render_header(&self) -> Html {
        match self {
            Header::Single(header) => header.clone().into(),
            Header::Group(group) => group.render(),
        }
    }

    fn grid_size(&self) -> (usize, usize) /* (rows, cols) */ {
        match self {
            Header::Single(_header) => (1, 1),
            Header::Group(group) => group.grid_size(),
        }
    }

    fn convert_to_rows(
        &self,
        link: &Scope<PwtHeaderGroup>,
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
        match self {
            Header::Single(header) => {
                rows[start_row].push(
                    Container::new()
                        .attribute(
                            "style",
                            format!("grid-row: {} / 10;grid-column-start: {}", start_row + 1, start_col + 1)
                        )
                        .with_child(
                            header.clone()
                                .on_resize({
                                    let link = link.clone();
                                    move |width| {
                                        let width: usize = if width > 0 { width as usize } else  { 0 };
                                        link.send_message(Msg::ResizeColumn(start_col, width));
                                    }
                                })
                        )
                        .into()
                );
                1
            }
            Header::Group(group) => group.convert_to_rows(link, start_row, start_col, rows),
        }
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct HeaderGroup {
    pub node_ref: Option<NodeRef>,
    pub key: Option<Key>,

    pub content: Option<VNode>,

    #[prop_or_default]
    pub children: Vec<Header>,
}


impl HeaderGroup {

    /// Create a new instance.
    pub fn  new() -> Self {
        yew::props!(Self {})
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

    /// Builder style method to set the header text
    pub fn content(mut self, content: impl IntoPropValue<Option<VNode>>) -> Self {
        self.set_content(content);
        self
    }

    /// Method to set the header text
    pub fn set_content(&mut self, content: impl IntoPropValue<Option<VNode>>) {
        self.content = content.into_prop_value();
    }

    pub fn with_child(mut self, header: impl Into<Header>) -> Self {
        self.add_child(header);
        self
    }

    pub fn add_child(&mut self, header: impl Into<Header>) {
        self.children.push(header.into())
    }

    fn render(&self) -> Html {
        Column::new()
            .with_optional_child(
                self.content.clone().map(|content| {
                    Container::new()
                        .class("pwt-datatable2-group-header-item")
                        .with_child(content)
                })
            )
            .with_child({
                let children = self.children.iter().map(|header| {
                    header.render_header()
                });
                Row::new()
                    .class("pwt-flex-fill")
                    .class("pwt-align-items-stretch")
                    .children(children)
            })
            .into()
    }

    fn grid_size(&self) -> (usize, usize) /* (rows, cols) */ {
        let mut rows = 0;
        let mut cols = 0;

        for child in &self.children {
            let (child_rows, child_cols) = child.grid_size();
            cols += child_cols;
            if child_rows > rows { rows = child_rows; }
        }

        if self.content.is_some() {
            rows += 1;
        }

        (rows, cols)
    }

    fn convert_to_rows(
        &self,
        link: &Scope<PwtHeaderGroup>,
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

        let child_start_row =  if self.content.is_some() { start_row + 1 } else { start_row };
        let mut span = 0;
        for child in &self.children {
            span += child.convert_to_rows(link, child_start_row, start_col + span, rows);
        }

        if let Some(content) = self.content.clone() {
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
}

pub enum Msg {
    ResizeColumn(usize, usize),
}

pub struct PwtHeaderGroup {
    node_ref: NodeRef,
}

impl Component for PwtHeaderGroup {
    type Message = Msg;
    type Properties =  HeaderGroup;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        Self {
            node_ref: props.node_ref.clone().unwrap_or(NodeRef::default()),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::ResizeColumn(col_num, width) => {
                log::info!("resize col {} to {}", col_num, width);
                true
            }

        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        //let grid_size = props.grid_size();
        //log::info!("GRID SIZE {:?}", grid_size);

        let mut rows = Vec::new();
        let column_count = props.convert_to_rows(ctx.link(), 0, 0, &mut rows);

        let rows: Vec<Html> = rows.into_iter().map(|row| {
            row.into_iter()
        }).flatten().collect();

        Row::new()
            .node_ref(self.node_ref.clone())
            .class("pwt-datatable2-header")
            .class("pwt-justify-content-start")
            .attribute("tabindex", "0")
            .with_child(
                Container::new()
                    .class("pwt-d-grid")
                    .attribute("style", format!("grid-template-columns: repeat({}, auto);", column_count))
                    .children(rows)
            )
            .into()
    }
}


impl Into<VNode> for HeaderGroup {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtHeaderGroup>(Rc::new(self), key);
        VNode::from(comp)
    }

}
