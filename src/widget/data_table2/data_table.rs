use std::rc::Rc;
use std::cmp::Ordering;

use derivative::Derivative;
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::{IntoEventCallback, IntoPropValue};

use crate::prelude::*;
use crate::state::{optional_rc_ptr_eq, DataFilter};
use crate::widget::Column;

use super::{DataTableColumn, DataTableHeader, Header};

pub enum Msg {
    ColumnWidthChange(Vec<usize>),
}
// DataTable properties
#[derive(Properties)]
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct DataTable<T: 'static> {

    #[prop_or_default]
    node_ref: NodeRef,
    pub key: Option<Key>,

    headers: Rc<Vec<Header<T>>>,

    #[derivative(PartialEq(compare_with="optional_rc_ptr_eq::<T>"))]
    pub data: Option<Rc<Vec<T>>>,

}

impl <T: 'static> DataTable<T> {

    pub fn new(headers: Rc<Vec<Header<T>>>) -> Self {
        yew::props!(DataTable<T> { headers })
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

    pub fn data(mut self, data: impl IntoPropValue<Option<Rc<Vec<T>>>>) -> Self {
        self.set_data(data);
        self
    }

    pub fn set_data(&mut self, data: impl IntoPropValue<Option<Rc<Vec<T>>>>) {
        self.data = data.into_prop_value();
    }
}

#[doc(hidden)]
pub struct PwtDataTable<T: 'static> {
    store: DataFilter<T>,
    columns: Vec<DataTableColumn<T>>,
    column_widths: Vec<usize>,
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

        Self {
            store,
            columns,
            column_widths: Vec::new(),
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::ColumnWidthChange(column_widths) => {
                log::info!("CW {:?}", column_widths);
                self.column_widths = column_widths;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        //let row_count = self.data.filtered_data_len();

        let render_subgrid = |width: &[usize]| {
            let class = "pwt-datatable2-cell";

            let template = width.iter().fold(String::new(), |mut acc, w| {
                if !acc.is_empty() {
                    acc.push(' ');
                }
                acc.push_str(&format!("{w}px"));
                acc
            });

            let style = format!("grid-column: 1 / -1; display:grid; grid-template-columns: {};", template);

            let subgrid = html!{
                <div {style}>
                    <div {class} style="grid-column-start: 1;">{"CHILD1XXXXX"}</div>
                    <div {class} style="grid-column-start: 2;">{"CHILD2"}</div>
                    <div {class} style="grid-column-start: 3;">{"CHILD3"}</div>
                    <div {class} style="grid-column-start: 4;">{"CHILD4"}</div>
                    <div {class} style="grid-column-start: 5;">{"CHILD5"}</div>
                    <div {class} style="grid-column-start: 6;">{"CHILD6"}</div>
                    </div>
            };
            subgrid
        };

        let subgrid = (!self.column_widths.is_empty()).then(|| render_subgrid(&self.column_widths));

        Column::new()
            .with_child(
                DataTableHeader::new(props.headers.clone())
                    .on_size_change(ctx.link().callback(Msg::ColumnWidthChange))

            )
            .with_optional_child(subgrid)
            .into()
    }
}

impl<T: 'static> Into<VNode> for DataTable<T> {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtDataTable<T>>(Rc::new(self), key);
        VNode::from(comp)
    }
}
