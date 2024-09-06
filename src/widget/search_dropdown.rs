use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

use anyhow::Error;
use derivative::Derivative;

use yew::html::{IntoEventCallback, Scope};
use yew::virtual_dom::Key;

use crate::prelude::*;
use crate::props::{CssLength, FieldBuilder, RenderFn, WidgetBuilder};
use crate::state::DataStore;
use crate::widget::data_table::{
    DataTable, DataTableColumn, DataTableHeader, DataTableKeyboardEvent, DataTableMouseEvent,
};
use crate::widget::{Dropdown, DropdownController};

use pwt_macros::{builder, widget};

#[derive(Clone)]
/// Parameters passed to the [SearchDropdown] picker callback.
///
/// The select function trigger a selection and closes the dropdown.
pub struct SearchDropdownRenderArgs<S: DataStore + 'static> {
    /// The [DataStore] used by the [SearchDropdown].
    pub store: S,
    /// Drowdown controller.
    pub controller: DropdownController,

    link: Scope<PwtSearchDropdown<S>>,
}

impl<S: DataStore + 'static> SearchDropdownRenderArgs<S> {
    /// Trigger a selection and close the dropdown.
    pub fn select(&self, key: Key) {
        self.link.send_message(Msg::Select(key));
        self.controller.change_value(String::from("")); // close dropdown, clear filter
    }
}

/// Load callback with filter parameter.
///
/// The callback gets called with the current value of the dropdown input, and
/// should return the filtered data.
pub struct FilteredLoadCallback<T> {
    callback: Rc<dyn Fn(String) -> Pin<Box<dyn Future<Output = Result<T, Error>>>>>,
}

impl<T> FilteredLoadCallback<T> {
    /// Create a new instance.
    pub fn new<F, R>(callback: F) -> Self
    where
        F: 'static + Fn(String) -> R,
        R: 'static + Future<Output = Result<T, Error>>,
    {
        Self {
            callback: Rc::new(move |filter| {
                let future = callback(filter);
                Box::pin(future)
            }),
        }
    }

    pub async fn apply(&self, filter: String) -> Result<T, Error> {
        (self.callback)(filter).await
    }
}

impl<T> Clone for FilteredLoadCallback<T> {
    fn clone(&self) -> Self {
        Self {
            callback: Rc::clone(&self.callback),
        }
    }
}

impl<T> PartialEq for FilteredLoadCallback<T> {
    fn eq(&self, _other: &Self) -> bool {
        true // never trigger redraw
    }
}

/// Text box which presents search results in the dropdown.
///
/// Text input is passed to the async load callback as a filter. The user
/// can then select a value from  the dropdown.
///
/// # Note
///
/// This widget does not interact with a form context, so it ignore
/// form context related properties like (name, required, submit,
/// submit_empty).
#[widget(pwt=crate, comp=PwtSearchDropdown<S>, @input)]
#[derive(Derivative, Properties)]
#[derivative(Clone(bound = ""), PartialEq(bound = ""))]
#[builder]
pub struct SearchDropdown<S: DataStore + 'static> {
    /// Select callback, emitted when the user selects something from the picker.
    #[builder_cb(IntoEventCallback, into_event_callback, Key)]
    #[prop_or_default]
    pub on_select: Option<Callback<Key>>,

    /// Data loader callback.
    loader: FilteredLoadCallback<S>,

    /// Function to generate the picker widget.
    picker: RenderFn<SearchDropdownRenderArgs<S>>,
}

impl<S: DataStore + 'static> SearchDropdown<S> {
    /// Create a new instance with custom picker.
    pub fn new<Fut, F>(picker: impl Into<RenderFn<SearchDropdownRenderArgs<S>>>, loader: F) -> Self
    where
        F: Fn(String) -> Fut + 'static,
        Fut: Future<Output = Result<S, Error>> + 'static,
    {
        let loader = FilteredLoadCallback::new(loader);
        yew::props!(Self {
            loader,
            picker: picker.into()
        })
    }

    /// Create a new instance using a combobox like dropdown (simple text lines).
    pub fn simple<Fut, F>(render: impl Into<RenderFn<S::Record>>, loader: F) -> Self
    where
        F: Fn(String) -> Fut + 'static,
        Fut: Future<Output = Result<S, Error>> + 'static,
    {
        let loader = FilteredLoadCallback::new(loader);
        let render = render.into();

        let picker: RenderFn<SearchDropdownRenderArgs<S>> =
            RenderFn::new(move |args: &SearchDropdownRenderArgs<S>| {
                let columns = Rc::new(vec![DataTableColumn::new("Value")
                    .show_menu(false)
                    .render(render.clone())
                    .into()]);

                DataTable::new(columns, args.store.clone())
                    .max_height(CssLength::Em(20.0))
                    .show_header(false)
                    .on_row_click({
                        let args = args.clone();
                        move |event: &mut DataTableMouseEvent| {
                            args.select(event.record_key.clone());
                        }
                    })
                    .on_row_keydown({
                        let args = args.clone();
                        move |event: &mut DataTableKeyboardEvent| match event.key().as_str() {
                            " " | "Enter" => {
                                args.select(event.record_key.clone());
                            }
                            _ => {}
                        }
                    })
                    .into()
            });

        yew::props!(Self { loader, picker })
    }

    /// Create a new instance using a [DataTable] dropdown with specified columns.
    pub fn table<Fut, F>(columns: Rc<Vec<DataTableHeader<S::Record>>>, loader: F) -> Self
    where
        F: Fn(String) -> Fut + 'static,
        Fut: Future<Output = Result<S, Error>> + 'static,
    {
        let loader = FilteredLoadCallback::new(loader);

        let picker: RenderFn<SearchDropdownRenderArgs<S>> =
            RenderFn::new(move |args: &SearchDropdownRenderArgs<S>| {
                DataTable::new(columns.clone(), args.store.clone())
                    .max_height(CssLength::Em(20.0))
                    .header_focusable(false)
                    .on_row_click({
                        let args = args.clone();
                        move |event: &mut DataTableMouseEvent| {
                            args.select(event.record_key.clone());
                        }
                    })
                    .on_row_keydown({
                        let args = args.clone();
                        move |event: &mut DataTableKeyboardEvent| match event.key().as_str() {
                            " " | "Enter" => {
                                args.select(event.record_key.clone());
                            }
                            _ => {}
                        }
                    })
                    .into()
            });

        yew::props!(Self { loader, picker })
    }
}

pub enum Msg<S: DataStore> {
    UpdateFilter(String),
    LoadResult(Result<S, Error>),
    Select(Key),
}

#[doc(hidden)]
pub struct PwtSearchDropdown<S: DataStore + 'static> {
    filter: String,
    load_error: Option<String>,
    store: Option<S>,
}

impl<S: DataStore + 'static> Component for PwtSearchDropdown<S> {
    type Message = Msg<S>;
    type Properties = SearchDropdown<S>;

    fn create(ctx: &Context<Self>) -> Self {
        let me = Self {
            filter: String::new(),
            load_error: None,
            store: None,
        };
        me.reload(ctx);
        me
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateFilter(filter) => {
                self.filter = filter;
                self.reload(ctx);
                true
            }
            Msg::LoadResult(result) => {
                match result {
                    Ok(store) => {
                        self.store = Some(store);
                        self.load_error = None;
                    }
                    Err(err) => {
                        self.store = None;
                        self.load_error = Some(err.to_string());
                    }
                }
                true
            }
            Msg::Select(key) => {
                if let Some(on_select) = &ctx.props().on_select {
                    on_select.emit(key);
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let store = self.store.clone();
        let load_error = self.load_error.clone();
        let link = ctx.link().clone();
        let picker = props.picker.clone();

        Dropdown::new(move |controller: &DropdownController| -> Html {
            if let Some(store) = &store {
                if let Some(load_error) = &load_error {
                    crate::widget::error_message(&format!("Error: {}", load_error))
                        .padding(2)
                        .into()
                } else {
                    let args = SearchDropdownRenderArgs {
                        store: store.clone(),
                        controller: controller.clone(),
                        link: link.clone(),
                    };
                    picker.apply(&args)
                }
            } else {
                crate::widget::error_message("no data loaded")
                    .padding(2)
                    .into()
            }
        })
        .with_std_props(&props.std_props)
        .with_input_props(&props.input_props)
        .value(self.filter.clone())
        .editable(true)
        .on_change(ctx.link().callback(Msg::UpdateFilter))
        .into()
    }
}

impl<S: DataStore + 'static> PwtSearchDropdown<S> {
    fn reload(&self, ctx: &Context<Self>) {
        let loader = ctx.props().loader.clone();
        let filter = self.filter.clone();
        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            let res = loader.apply(filter).await;
            link.send_message(Msg::LoadResult(res));
        });
    }
}
