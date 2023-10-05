use std::marker::PhantomData;
use std::rc::Rc;

use derivative::Derivative;
use web_sys::HtmlInputElement;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::prelude::*;
use crate::props::{FilterFn, TextFilterFn, IntoTextFilterFn};
use crate::state::{DataStore, Selection, SelectionObserver};
use crate::widget::data_table::{DataTable, DataTableMouseEvent};
use crate::widget::{Column, Input, Row};

use pwt_macros::builder;

/// Display a [DataTable] with optional text filter field.
///
/// Allows you to select one or more items from a table. This is usually used
/// to implement [Dropdown](crate::widget::Dropdown) pickers.
///
/// # Note
///
/// This widget overwrites the store filter, so the passed store can not
/// use any filter function. A workaround is to filter data on load instead.
#[derive(Derivative, Properties)]
#[derivative(Clone(bound = ""), PartialEq(bound = ""))]
#[builder]
pub struct GridPicker<S: DataStore> {
    #[prop_or_default]
    node_ref: NodeRef,
    /// Yew key property.
    #[prop_or_default]
    pub key: Option<Key>,

    table: DataTable<S>,

    /// Selection object.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub selection: Option<Selection>,

    /// Select callback.
    #[builder_cb(IntoEventCallback, into_event_callback, Key)]
    #[prop_or_default]
    pub on_select: Option<Callback<Key>>,

    /// Filter change event.
    ///
    /// Filter change often change the number of displayed items, so
    /// the size of the widget is likely to change. This callback is
    /// useful to reposition the dropdown.
    #[builder_cb(IntoEventCallback, into_event_callback, String)]
    #[prop_or_default]
    pub on_filter_change: Option<Callback<String>>,

    /// Show filter
    ///
    /// Default behavior is to show the filter for pickers with more than 10 items.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub show_filter: Option<bool>,

    /// Custom filter function.
    ///
    /// The default filter function is:
    /// ```
    /// # fn filter(item: &str, query: &str) -> bool {
    /// item.to_lowercase().contains(&query.to_lowercase())
    /// # }
    /// ```
    #[builder_cb(IntoTextFilterFn, into_text_filter_fn, S::Record)]
    #[prop_or_default]
    pub filter: Option<TextFilterFn<S::Record>>,
}

impl<S: DataStore> GridPicker<S> {
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
    pub fn key(mut self, key: impl IntoOptionalKey) -> Self {
        self.set_key(key);
        self
    }

    /// Method to set the yew `key` property
    pub fn set_key(&mut self, key: impl IntoOptionalKey) {
        self.key = key.into_optional_key();
    }
}

pub enum Msg {
    FilterUpdate(String),
}

#[doc(hidden)]
pub struct PwtGridPicker<S> {
    filter: String,
    store: S,
    _selection_observer: Option<SelectionObserver>,
    _phantom: PhantomData<S>,
}

impl<S: DataStore> PwtGridPicker<S> {
    fn update_filter(&mut self, props: &GridPicker<S>, filter: String) {
        self.filter = filter;

        if let Some(ref on_filter_change) = props.on_filter_change {
            on_filter_change.emit(self.filter.clone());
        }

        if self.filter.is_empty() {
            self.store.set_filter(None);
        } else {
            self.store.set_filter(if let Some(filter) = &props.filter {
                let filter_function = filter.clone();
                let query = self.filter.clone();
                FilterFn::new(move |item| {
                    filter_function.apply(item, &query)
                })
            } else {
                let extract_key_fn = self.store.get_extract_key_fn();
                let filter = self.filter.clone();
                FilterFn::new(move |item| {
                    let key = extract_key_fn.apply(item);
                    key.to_lowercase().contains(&filter.to_lowercase())
                })
            });
        }
    }
}

impl<S: DataStore + 'static> Component for PwtGridPicker<S> {
    type Message = Msg;
    type Properties = GridPicker<S>;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let _selection_observer = match &props.selection {
            Some(selection) => Some(selection.add_listener({
                let on_select = props.on_select.clone();
                move |selection: Selection| {
                    if let Some(on_select) = &on_select {
                        if let Some(key) = selection.selected_key() {
                            on_select.emit(key);
                        }
                    }
                }
            })),
            None => None,
        };

        let mut me = Self {
            _selection_observer,
            _phantom: PhantomData::<S>,
            filter: String::new(),
            store: props.table.get_store(),
        };

        me.update_filter(props, String::new()); // clear store filter

        me
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::FilterUpdate(filter) => {
                self.update_filter(props, filter);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let table: Html = props
            .table
            .clone()
            .key(Key::from("picker-table"))
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

        let show_filter = props.show_filter.unwrap_or_else(|| {
            if self.store.data_len() > 10 {
                true
            } else {
                false
            }
        });

        if show_filter {
            let filter_invalid = false;
            let filter = Row::new()
                .key(Key::from("picker-filter"))
                .gap(2)
                .class("pwt-p-2 pwt-w-100 pwt-align-items-center")
                .with_child(html! {<label for="testinput">{"Filter"}</label>})
                .with_child(
                    Input::new()
                        .attribute("autocomplete", "off")
                        .attribute("size", "1") // make size minimal
                        .class("pwt-input")
                        .class("pwt-w-100")
                        .class(if filter_invalid {
                            "is-invalid"
                        } else {
                            "is-valid"
                        })
                        .attribute("value", self.filter.clone())
                        .attribute("aria-invalid", filter_invalid.then(|| "true"))
                        .oninput(ctx.link().callback(move |event: InputEvent| {
                            let input: HtmlInputElement = event.target_unchecked_into();
                            Msg::FilterUpdate(input.value())
                        })),
                );

            view.add_child(filter);
        }

        view.add_child(table);

        view.into()
    }
}

impl<S: DataStore + 'static> Into<VNode> for GridPicker<S> {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtGridPicker<S>>(Rc::new(self), key);
        VNode::from(comp)
    }
}
