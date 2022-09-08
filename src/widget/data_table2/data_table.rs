use std::rc::Rc;
use std::cmp::Ordering;

use derivative::Derivative;
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::{IntoEventCallback, IntoPropValue};

use crate::prelude::*;
use crate::state::{optional_rc_ptr_eq, DataFilter};
use crate::widget::{Container, Column};

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

    #[prop_or_default]
    pub class: Classes,

    headers: Rc<Vec<Header<T>>>,

    #[derivative(PartialEq(compare_with="optional_rc_ptr_eq::<T>"))]
    pub data: Option<Rc<Vec<T>>>,

    /// set class for table cells (default is "pwt-truncate pwt-p-2")
    pub cell_class: Option<String>,

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

    /// Builder style method to add a html class
    pub fn class(mut self, class: impl Into<Classes>) -> Self {
        self.add_class(class);
        self
    }

    /// Method to add a html class
    pub fn add_class(&mut self, class: impl Into<Classes>) {
        self.class.push(class);
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

            let mut table = Container::new()
                .tag("table")
                .attribute("style", "table-layout: fixed;width:1px;")
                .with_child(html!{
                    <tr>{
                        width.iter().map(|w| html!{
                            <td style={format!("width:{w}px;height:0px;")}></td>
                        }).collect::<Html>()
                    }</tr>
                });

            let cell_class = props.cell_class.clone()
                .unwrap_or_else(|| String::from("pwt-text-truncate pwt-p-2"));

            for (_i, record_num, item) in self.store.filtered_data() {

                let mut row = Container::new()
                    .tag("tr");

                let selected = false;

                for (column_num, column) in self.columns.iter().enumerate() {
                    let item_style = format!("text-align:{};", column.justify);
                    let class = if selected { Some("selected") } else {None };
                    row.add_child(html!{
                        <td {class} style={item_style}><div class={&cell_class}>{ column.render.apply(item) }</div></td>
                    });
                }
                table.add_child(row);
            }


            let scroll = Container::new()
                .class("pwt-flex-fill")
                .class("pwt-overflow-auto")
                .with_child(table);

            scroll
        };

        let subgrid = (!self.column_widths.is_empty()).then(|| render_subgrid(&self.column_widths));

        Column::new()
            .class(props.class.clone())
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
