use anyhow::{bail, Error};
use derivative::Derivative;
use serde_json::Value;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;
use yew::virtual_dom::Key;

use super::{
    IntoValidateFn, ManagedField, ManagedFieldContext, ManagedFieldMaster, ManagedFieldState,
    ValidateFn,
};
use crate::prelude::*;
use crate::props::{IntoLoadCallback, IntoOptionalRenderFn, LoadCallback, RenderFn};
use crate::state::{DataStore, Selection};
use crate::widget::{error_message, Dropdown};

use pwt_macros::{builder, widget};

/// Parameters passed to the [Selector::picker] callback.
pub struct SelectorRenderArgs<S: DataStore> {
    /// The [DataStore] used by the [Selector].
    pub store: S,
    /// The selection.
    pub selection: Selection,
    /// This callback must be called by the picker to select
    /// something.
    pub on_select: Callback<Key>,
}

pub type PwtSelector<S> = ManagedFieldMaster<SelectorField<S>>;

/// Helper widget to implement [Combobox](super::Combobox) like selectors.
///
/// This helper simplifies the implementation of  [Combobox](super::Combobox) like
/// selectors with complex layouts (table, trees).
///
/// - Extends the [Dropdown] widget.
///
/// - Use a shared `DataStore` as data storage (either a
/// [Store](crate::state::Store) or
/// [TreeStore](crate::state::TreeStore)).
///
/// - Ability to load data using a [LoadCallback] (with reasonable
/// error handling).
///
/// - Handles [FormContext](super::FormContext) interaction.
///
/// # Note
///
/// Please use a trackable [LoadCallback] to avoid unnecessary
/// reloads.
#[widget(pwt=crate, comp=ManagedFieldMaster<SelectorField<S>>, @input)]
#[derive(Derivative, Properties)]
#[derivative(Clone(bound = ""), PartialEq(bound = ""))]
#[builder]
pub struct Selector<S: DataStore + 'static> {
    store: S,
    /// The default value.
    #[builder(IntoPropValue, into_prop_value)]
    pub default: Option<AttrValue>,

    /// Make the input editable.
    #[prop_or_default]
    #[builder]
    pub editable: bool,

    /// Autoselect flag.
    ///
    /// If there is no default, automatically select the first loaded
    /// data item.
    #[prop_or_default]
    #[builder]
    pub autoselect: bool,

    /// Change callback
    #[builder_cb(IntoEventCallback, into_event_callback, Key)]
    pub on_change: Option<Callback<Key>>,

    /// Picker render function
    pub picker: RenderFn<SelectorRenderArgs<S>>,
    /// Validate callback.
    pub validate: Option<ValidateFn<(String, S)>>,
    /// Data loader callback.
    pub loader: Option<LoadCallback<S::Collection>>,

    /// Display the output of this function instead of displaying values directly.
    ///
    /// Note: selectors using this feature are not editable (editable property is ignored)!
    #[builder_cb(IntoOptionalRenderFn, into_optional_render_fn, AttrValue)]
    pub render_value: Option<RenderFn<AttrValue>>,
}

impl<S: DataStore> Selector<S> {
    /// Creates a new instance
    pub fn new(store: S, picker: impl Into<RenderFn<SelectorRenderArgs<S>>>) -> Self {
        yew::props!(Self {
            store,
            picker: picker.into(),
        })
    }

    /// Builder style method to set the validate callback
    pub fn validate(mut self, validate: impl IntoValidateFn<(String, S)>) -> Self {
        self.set_validate(validate);
        self
    }

    /// Method to set the validate callback
    pub fn set_validate(&mut self, validate: impl IntoValidateFn<(String, S)>) {
        self.validate = validate.into_validate_fn();
    }

    /// Builder style method to set the load callback.
    pub fn loader(mut self, callback: impl IntoLoadCallback<S::Collection>) -> Self {
        self.set_loader(callback);
        self
    }

    /// Method to set the load callback.
    pub fn set_loader(&mut self, callback: impl IntoLoadCallback<S::Collection>) {
        self.loader = callback.into_load_callback();
    }
}

pub enum Msg<S: DataStore> {
    Select(String),
    DataChange,
    LoadResult(Result<S::Collection, Error>),
}

#[doc(hidden)]
pub struct SelectorField<S: DataStore> {
    selection: Selection,
    load_error: Option<String>,
    _store_observer: S::Observer,
}

fn create_selector_validation_cb<S: DataStore + 'static>(props: &Selector<S>) -> ValidateFn<Value> {
    let store = props.store.clone();
    let required = props.input_props.required;
    let validate = props.validate.clone();
    ValidateFn::new(move |value: &Value| {
        let value = match value {
            Value::Null => String::new(),
            Value::String(v) => v.clone(),
            _ => {
                // should not happen
                log::error!("PwtField: got wrong data type in validate!");
                String::new()
            }
        };

        if value.is_empty() {
            if required {
                bail!("Field may not be empty.");
            } else {
                return Ok(());
            }
        }

        if !store.is_empty() {
            match &validate {
                Some(cb) => cb.validate(&(value.into(), store.clone())),
                None => Ok(()),
            }
        } else {
            bail!("no data loaded");
        }
    })
}

impl<S: DataStore + 'static> SelectorField<S> {
    fn load(&self, ctx: &ManagedFieldContext<Self>) {
        let props = ctx.props();
        let link = ctx.link().clone();
        if let Some(loader) = props.loader.clone() {
            wasm_bindgen_futures::spawn_local(async move {
                let res = loader.apply().await;
                link.send_message(Msg::LoadResult(res));
            });
        } else {
            // just trigger a data change to set the default value.
            link.send_message(Msg::DataChange);
        }
    }
}

impl<S: DataStore + 'static> ManagedField for SelectorField<S> {
    type Message = Msg<S>;
    type Properties = Selector<S>;

    fn setup(props: &Self::Properties) -> ManagedFieldState {
        let validate = create_selector_validation_cb(props);

        let default: Value = props.default.as_deref().unwrap_or("").to_string().into();
        let value = default.clone();

        let valid = validate.validate(&value).map_err(|err| err.to_string());

        ManagedFieldState {
            value,
            valid,
            validate,
            default,
            radio_group: false,
            unique: false,
        }
    }

    fn value_changed(&mut self, ctx: &super::ManagedFieldContext<Self>) {
        let props = ctx.props();
        let state = ctx.state();
        let key = Key::from(state.value.as_str().unwrap_or(""));

        self.selection.select(key.clone());

        if let Some(on_change) = &props.on_change {
            on_change.emit(key);
        }
    }

    fn create(ctx: &ManagedFieldContext<Self>) -> Self {
        let props = ctx.props();

        let default = props.default.as_deref().unwrap_or("").to_string();
        let selection = Selection::new();
        if !default.is_empty() {
            selection.select(default.clone());
        }

        let _store_observer = props
            .store
            .add_listener(ctx.link().callback(|_| Msg::DataChange));

        let me = Self {
            selection,
            load_error: None,
            _store_observer,
        };

        me.load(ctx);

        me
    }

    fn update(&mut self, ctx: &ManagedFieldContext<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::LoadResult(res) => {
                match res {
                    Ok(data) => {
                        self.load_error = None;
                        props.store.set_data(data);
                   }
                    Err(err) => {
                        props.store.clear();
                        let default = props.default.as_deref().unwrap_or("").to_string();
                        ctx.link().update_value(default.to_string());
                        self.load_error = Some(err.to_string());
                    }
                }
                true
            }
            Msg::DataChange => {
                let state = ctx.state();
                let value = state.value.as_str().unwrap_or("").to_owned();

                if self.load_error.is_none() {
                    if value.is_empty() {
                        let mut default = props.default.clone();

                        if default.is_none() && props.autoselect {
                            if let Some((_pos, node)) = props.store.filtered_data().next() {
                                default = Some(AttrValue::from(node.key().to_string()));
                            }
                        }

                        if let Some(default) = default {
                            ctx.link().update_value(default.to_string());
                            ctx.link().update_default(default.to_string());
                        }
                    }
                }
                ctx.link().validate(); // re-evaluate
                true
            }
            Msg::Select(value) => {
                if !props.input_props.disabled {
                    ctx.link().update_value(value);
                }
                false
            }
        }
    }

    fn changed(&mut self, ctx: &ManagedFieldContext<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();

        let mut reload = false;

        if props.store != old_props.store {
            self._store_observer = props
                .store
                .add_listener(ctx.link().callback(|_| Msg::DataChange));
            reload = true;
        }

        if props.loader != old_props.loader {
            reload = true;
        }

        if reload {
            self.load(ctx);
        }

        true
    }

    fn view(&self, ctx: &ManagedFieldContext<Self>) -> Html {
        let props = ctx.props();
        let state = ctx.state();

        let value = state.value.as_str().unwrap_or("").to_owned();
        let valid = state.valid.clone();

        let picker = {
            let picker = props.picker.clone();
            let store = props.store.clone();
            let selection = self.selection.clone();

            let load_error = self.load_error.clone();

            move |on_select: &Callback<Key>| {
                if let Some(load_error) = &load_error {
                    return error_message(&format!("Error: {}", load_error), "pwt-p-2");
                }

                if store.is_empty() {
                    return html! {
                        <div class="pwt-p-2">{"List does not contain any items."}</div>
                    };
                }

                let render_picker_args = SelectorRenderArgs {
                    store: store.clone(),
                    selection: selection.clone(),
                    on_select: on_select.clone(), // fixme
                };
                picker.apply(&render_picker_args)
            }
        };

        let tip = match &valid {
            Err(msg) => Some(msg.to_string()),
            Ok(_) => None,
        };

        Dropdown::new(picker)
            .with_std_props(&props.std_props)
            .with_input_props(&props.input_props)
            .editable(props.editable)
            .class(if valid.is_ok() {
                "is-valid"
            } else {
                "is-invalid"
            })
            .on_change(ctx.link().callback(|key: String| Msg::Select(key)))
            .value(value)
            .render_value(props.render_value.clone())
            .tip(tip)
            .into()
    }
}
