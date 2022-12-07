use anyhow::{bail, Error};
use serde_json::Value;
use derivative::Derivative;

use yew::prelude::*;
use yew::virtual_dom::Key;
use yew::html::{IntoEventCallback, IntoPropValue};

use proxmox_schema::Schema;

use crate::prelude::*;
use crate::props::{RenderFn, IntoLoadCallback, LoadCallback};
use crate::state::{DataStore, Selection2};
use crate::widget::Dropdown;
use crate::component::error_message;

use super::{FieldOptions, FormContext, TextFieldStateHandle, ValidateFn};

use pwt_macros::widget;

pub struct Selector2RenderArgs<S: DataStore> {
    pub store: S,
    pub selection: Selection2,
    pub on_select: Callback<Key>,
}

/// Combobox like selector
///
/// Supports async data loading and generic picker widget
/// implementations.
///
/// Note: Please use a trackable [LoadCallback] to avoid unnecessary
/// reloads.
#[widget(PwtSelector2<S>, @input, @element)]
#[derive(Derivative, Properties)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct Selector2<S: DataStore + 'static> {
    store: S,
    /// Name of the form field.
    ///
    /// The field register itself with this `name` in the FormContext
    /// (if any).
    pub name: Option<AttrValue>,
    pub default: Option<AttrValue>,
    #[prop_or_default]
    pub editable: bool,
    #[prop_or_default]
    pub autoselect: bool,
    pub on_select: Option<Callback<Key>>,
    pub picker: RenderFn<Selector2RenderArgs<S>>,
    pub validate: Option<ValidateFn<(String, S)>>,
    pub loader: Option<LoadCallback<S::Collection>>,
}

impl<S: DataStore> Selector2<S> {

    /// Creates a new instance
    pub fn new(
        store: S,
        picker: impl Into<RenderFn<Selector2RenderArgs<S>>>,
    ) -> Self {
        yew::props!(Self {
            store,
            picker: picker.into(),
        })
    }

    /// Builder style method to set the field name.
    pub fn name(mut self, name: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_name(name);
        self
    }

    /// Method to set the field name.
    pub fn set_name(&mut self, name: impl IntoPropValue<Option<AttrValue>>) {
        self.name = name.into_prop_value();
    }

    /// Builder style method to set the default item.
    pub fn default(mut self, default: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_default(default);
        self
    }

    /// Method to set the default item.
    pub fn set_default(&mut self, default: impl IntoPropValue<Option<AttrValue>>) {
        self.default = default.into_prop_value();
    }

    /// Builder style method to set the editable flag
    pub fn editable(mut self, editable: bool) -> Self {
        self.set_editable(editable);
        self
    }

    /// Method to set the editable flag
    pub fn set_editable(&mut self, editable: bool) {
        self.editable = editable;
    }

    /// Builder style method to set the autoselect flag
    pub fn autoselect(mut self, autoselect: bool) -> Self {
        self.set_autoselect(autoselect);
        self
    }

    /// Method to set the autoselect flag
    pub fn set_autoselect(&mut self, autoselect: bool) {
        self.autoselect = autoselect;
    }

    /// Builder style method to set the validate callback
    pub fn validate(
        mut self,
        validate: impl Into<ValidateFn<(String, S)>>,
    ) -> Self {
        self.set_validate(validate);
        self
    }

    /// Method to set the validate callback
    pub fn set_validate(
        &mut self,
        validate: impl Into<ValidateFn<(String, S)>>,
    ) {
        self.validate = Some(validate.into());
    }

    /// Builder style method to set the validation schema
    pub fn schema(mut self, schema: &'static Schema) -> Self {
        self.set_schema(schema);
        self
    }

    /// Method to set the validation schema
    pub fn set_schema(&mut self, schema: &'static Schema) {
        self.validate = Some(ValidateFn::new(move |(value, _list): &(String, _)| {
            schema.parse_simple_value(&value)?;
            Ok(())
        }));
    }

    /// Builder style method to set the on_select callback
    pub fn on_select(mut self, cb: impl IntoEventCallback<Key>) -> Self {
        self.on_select = cb.into_event_callback();
        self
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
    FormCtxUpdate(FormContext),
    DataChange,
    LoadResult(Result<S::Collection, Error>),
}

#[doc(hidden)]
pub struct PwtSelector2<S: DataStore> {
    state: TextFieldStateHandle,
    selection: Selection2,
    load_error: Option<String>,
    _store_observer: S::Observer,
}

fn create_selector_validation_cb<S: DataStore + 'static>(
    props: &Selector2<S>,
) -> ValidateFn<Value> {
    let store = props.store.clone();
    let required = props.input_props.required;
    let validate = props.validate.clone();
    ValidateFn::new(move |value: &Value| {
        let value = match value {
            Value::Null => String::new(),
            Value::String(v) => v.clone(),
            _ => { // should not happen
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
        } else  {
            bail!("no data loaded");
        }
    })
}

impl<S: DataStore + 'static> PwtSelector2<S> {

    fn load(&self, ctx: &Context<Self>) {
        let props = ctx.props();
        let link = ctx.link().clone();
        if let Some(loader) = props.loader.clone() {
            wasm_bindgen_futures::spawn_local(async move {
                let res = loader.apply().await;
                link.send_message(Msg::LoadResult(res));
            });
        }
    }
}

impl<S: DataStore + 'static> Component for PwtSelector2<S> {
    type Message = Msg<S>;
    type Properties = Selector2<S>;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let value = String::new();

        let on_form_ctx_change = Callback::from({
            let link = ctx.link().clone();
            move |form_ctx: FormContext| link.send_message(Msg::FormCtxUpdate(form_ctx))
        });

        let real_validate = create_selector_validation_cb(props);

        // TODO: do we need this? - TEST AGAIN?
        let on_change = Callback::from({
            let on_select = props.on_select.clone();
            move |value: String| {
                if let Some(on_select) = &on_select {
                    on_select.emit(Key::from(value));
                }
            }
        });

        let state = TextFieldStateHandle::new(
            ctx.link(),
            on_form_ctx_change,
            props.name.clone(),
            value,
            Some(real_validate),
            FieldOptions::from_field_props(&props.input_props),
            on_change,
        );

        let selection = Selection2::new();
        let _store_observer = props.store.add_listener(ctx.link().callback(|_| {
            Msg::DataChange
        }));

        let me = Self {
            state,
            selection,
            load_error: None,
            _store_observer,
        };

        me.load(ctx);

        me
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::LoadResult(res) => {
                match res {
                    Ok(data) => {
                        props.store.set_data(data);
                    }
                    Err(err) => {
                        self.load_error = Some(err.to_string());
                    }
                }
                true
            }
            Msg::FormCtxUpdate(form_ctx) => self.state.update(form_ctx),
            Msg::DataChange => {
                let (value, _valid) = self.state.get_field_data();

                if self.load_error.is_none() {
                    if value.is_empty() {

                        let mut default = props.default.clone();

                        if default.is_none() && props.autoselect {

                            if let Some((_pos, node)) = props.store.filtered_data().next() {
                                default = Some(AttrValue::from(node.key().to_string()));
                            }
                        }

                        if let Some(default) = default {
                            self.state.set_value(default.to_string().clone(), true);
                            return true; // set_value already validates
                        }
                    }
                }
                true
            }
            Msg::Select(value) => {
                if props.input_props.disabled { return true; }
                self.state.set_value(value, false);
                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        if props.store != old_props.store {
            self._store_observer = props.store.add_listener(ctx.link().callback(|_| {
                Msg::DataChange
            }));
            self.load(ctx);
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let (value, valid) = self.state.get_field_data();

        let picker = {
            let picker = props.picker.clone();
            let store = props.store.clone();
            let selection = self.selection.clone();
            selection.select(Key::from(value.as_str()));

            let load_error = self.load_error.clone();

            move |on_select: &Callback<Key>| {

                if let Some(load_error) = &load_error {
                    return error_message(&format!("Error: {}", load_error), "pwt-p-2");
                }

                if store.is_empty() {
                    return html!{
                        <div class="pwt-p-2">{"List does not contain any items."}</div>
                    };
                }

                let render_picker_args = Selector2RenderArgs {
                    store: store.clone(),
                    selection: selection.clone(),
                    on_select: on_select.clone(), // fixme
                };
                picker.apply(&render_picker_args)
            }
        };

        let tip = match &valid {
            Err(msg) => Some(html!{msg}),
            Ok(_) => None,
        };

        Dropdown::new(picker)
            .with_std_props(&props.std_props)
            .with_input_props(&props.input_props)
            .editable(props.editable)
            .class(if valid.is_ok() { "is-valid" } else { "is-invalid" })
            .on_change(ctx.link().callback(|key: String| Msg::Select(key)))
            .value(value)
            .tip(tip)
            .into()
    }

}
