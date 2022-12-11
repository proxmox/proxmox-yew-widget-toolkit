use std::rc::Rc;
use std::marker::PhantomData;

use derivative::Derivative;

use yew::prelude::*;
use yew::html::{IntoEventCallback, IntoPropValue};
use yew::virtual_dom::{Key, VComp, VNode};

use crate::prelude::*;
use crate::widget::Column;
use crate::widget::data_table2::{DataTable, DataTableMouseEvent};
//use crate::widget::form::Input;
use crate::state::{Selection2, SelectionObserver, DataStore};

#[derive(Derivative, Properties)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct GridPicker2<S: DataStore> {
    #[prop_or_default]
    node_ref: NodeRef,
    /// Yew key property.
    pub key: Option<Key>,

    table: DataTable<S>,

    /// Selection object.
    pub selection: Option<Selection2>,

    /// Select callback.
    pub on_select: Option<Callback<Key>>,

    /// Filter change event.
    ///
    /// Filter change often change the number of displayed items, so
    /// the size of the widget is likely to change. This callback is
    /// useful to reposition the dropdown.
    pub on_filter_change: Option<Callback<()>>,

    /// Show filter
    ///
    /// Defaul behavior is to show the filter for pickers with more than 10 items.
    pub show_filter: Option<bool>,
}

impl<S: DataStore> GridPicker2<S> {

    // Create a new instance.
    pub fn new(table: DataTable<S>) -> Self {
        yew::props!(Self { table })
    }

    /// Builder style method to set the yew `node_ref`
    pub fn node_ref(mut self, node_ref: ::yew::html::NodeRef) -> Self {
        self.set_node_ref(node_ref);
        self
    }

    /// Method to set the yew `node_ref`
    pub fn set_node_ref(&mut self, node_ref: ::yew::html::NodeRef) {
        self.node_ref = node_ref;
    }

    /// Builder style method to set the yew `key` property
    pub fn key(mut self, key: impl IntoPropValue<Option<Key>>) -> Self {
        self.set_key(key);
        self
    }

    /// Method to set the yew `key` property
    pub fn set_key(&mut self, key: impl IntoPropValue<Option<Key>>) {
        self.key = key.into_prop_value();
    }

    /// Builder style method to set the selection model.
    pub fn selection(mut self, selection: impl IntoPropValue<Option<Selection2>>) -> Self {
        self.selection = selection.into_prop_value();
        self
    }

    pub fn on_select(mut self, cb: impl IntoEventCallback<Key>) -> Self {
        self.on_select = cb.into_event_callback();
        self
    }

    pub fn on_filter_change(mut self, cb: impl IntoEventCallback<()>) -> Self {
        self.on_filter_change = cb.into_event_callback();
        self
    }

    pub fn show_filter(mut self, show_filter: impl IntoPropValue<Option<bool>>) -> Self {
        self.set_show_filter(show_filter);
        self
    }

    pub fn set_show_filter(&mut self, show_filter: impl IntoPropValue<Option<bool>>) {
        self.show_filter = show_filter.into_prop_value();
    }
}

#[doc(hidden)]
pub struct PwtGridPicker2<S> {
    filter: String,
    _selection_observer: Option<SelectionObserver>,
    _phantom: PhantomData<S>,
}

impl<S: DataStore + 'static> Component for PwtGridPicker2<S> {
    type Message = ();
    type Properties = GridPicker2<S>;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let _selection_observer = match &props.selection {
            Some(selection) => Some(selection.add_listener({
                let on_select = props.on_select.clone();
                move |selection: Selection2| {
                    if let Some(on_select) = &on_select {
                        if let Some(key) = selection.selected_key() {
                            on_select.emit(key);
                        }
                    }
                }
            })),
            None => None,
        };

        Self {
            _selection_observer,
            _phantom: PhantomData::<S>,
            filter: String::new(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let table: Html = props.table.clone()
            .autoselect(false)
            .hover(true)
            .header_focusable(false)
            .selection(props.selection.clone())
            .on_row_click(|event: &mut DataTableMouseEvent| {
                let key = event.record_key.clone();
                if let Some(selection) = &event.selection {
                    selection.select(key);
                }
            })
            .into();

        let mut view = Column::new()
            .node_ref(props.node_ref.clone())
            .class("pwt-flex-fill pwt-overflow-auto");

        view.add_child(table);

        view.into()
    }
}

impl<S: DataStore + 'static> Into<VNode> for GridPicker2<S> {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtGridPicker2<S>>(Rc::new(self), key);
        VNode::from(comp)
    }
}
