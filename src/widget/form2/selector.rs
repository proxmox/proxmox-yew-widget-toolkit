use std::rc::Rc;

use anyhow::{bail, Error};
use serde_json::Value;
use derivative::Derivative;


use yew::prelude::*;
use yew::virtual_dom::Key;
use yew::html::{IntoEventCallback, IntoPropValue};

use proxmox_schema::Schema;

use crate::prelude::*;
use crate::props::{ExtractKeyFn, IntoLoadCallback, LoadCallback, RenderFn};
use crate::state::Loader;
use crate::widget::Dropdown;
use crate::widget::form::ValidateFn;

use super::{FieldOptions, FormContext};

use pwt_macros::widget;

pub struct CreatePickerArgs<T> {
    pub list: Rc<Vec<T>>,
    pub selected: Key,
    pub on_select: Callback<Key>,
}

fn my_data_cmp_fn<T>(a: &Option<Rc<Vec<T>>>, b: &Option<Rc<Vec<T>>>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Rc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}

/// Combobox like selector
///
/// Supports async data loading and generic picker widget
/// implementations.
///
/// Note: Please use a trackable [LoadCallback] to avoid unnecessary
/// reloads.
#[widget(PwtSelector<T>, @input, @element)]
#[derive(Derivative, Properties)]
// Note: use derivative to avoid Clone/PartialEq requirement on T
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct Selector<T: 'static> {
    pub name: AttrValue,

    pub loader: Option<LoadCallback<Vec<T>>>,
    #[derivative(PartialEq(compare_with="my_data_cmp_fn"))]
    pub data: Option<Rc<Vec<T>>>,
    pub default: Option<AttrValue>,
    #[prop_or_default]
    pub editable: bool,
    #[prop_or_default]
    pub autoselect: bool,
    /// Extract Key from item
    ///
    /// Onyl used to auto-select the first entry (if default is not set)
    pub extract_key: Option<ExtractKeyFn<T>>,
    pub on_select: Option<Callback<Key>>,
    pub picker: RenderFn<CreatePickerArgs<T>>,
    pub validate: Option<ValidateFn<(String, Rc<Vec<T>>)>>,
}

impl<T: 'static> Selector<T> {

    pub fn new(
        name: impl IntoPropValue<AttrValue>,
        picker: RenderFn<CreatePickerArgs<T>>,
    ) -> Self {
        yew::props!(Self {
            name: name.into_prop_value(),
            picker,
        })
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

    /// Builder style method to set the load callback.
    pub fn loader(mut self, callback: impl IntoLoadCallback<Vec<T>>) -> Self {
        self.set_loader(callback);
        self
    }

    /// Method to set the load callback.
    pub fn set_loader(&mut self, callback: impl IntoLoadCallback<Vec<T>>) {
        self.loader = callback.into_load_callback();
    }

    /// Builder style method to set the data
    pub fn data(mut self, data: impl IntoPropValue<Option<Rc<Vec<T>>>>) -> Self {
        self.set_data(data);
        self
    }

    /// Method to set the data
    pub fn set_data(&mut self, data: impl IntoPropValue<Option<Rc<Vec<T>>>>) {
        self.data = data.into_prop_value();
    }

    pub fn extract_key(mut self, extract_fn: impl Into<ExtractKeyFn<T>>) -> Self {
        self.extract_key = Some(extract_fn.into());
        self
    }

    /// Builder style method to set the validate callback
    pub fn validate(
        mut self,
        validate: impl 'static + Fn(&(String, Rc<Vec<T>>)) -> Result<(), Error>,
    ) -> Self {
        self.set_validate(validate);
        self
    }

    /// Method to set the validate callback
    pub fn set_validate(
        &mut self,
        validate: impl 'static + Fn(&(String, Rc<Vec<T>>)) -> Result<(), Error>,
    ) {
        self.validate = Some(ValidateFn::new(validate));
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
}

pub enum Msg {
    Select(String),
    FormCtxUpdate(FormContext),
    UpdateList,
}

pub struct PwtSelector<T> {
    loader: Loader<Vec<T>>,
    last_loader: Option<LoadCallback<Vec<T>>>, //change tracking
    last_data: Option<Rc<Vec<T>>>, //change tracking
    value: String,

    real_validate: ValidateFn<Value>,

    form_ctx: Option<FormContext>,
    _form_ctx_handle: Option<ContextHandle<FormContext>>,
}

fn create_selector_validation_cb<T: 'static>(props: Selector<T>, data: Option<Rc<Vec<T>>>) -> ValidateFn<Value> {
    ValidateFn::new(move |value: &Value| {
        let value = match value {
            Value::Null => String::new(),
            Value::String(v) => v.clone(),
            _ => { // should not happen
                log::error!("PwtField: got wrong data type in validate (field '{}')!", props.name);
                String::new()
            }
        };

        if value.is_empty() {
            if props.input_props.required {
                bail!("Field may not be empty.");
            } else {
                return Ok(());
            }
        }

        match &data {
            Some(list) => {
                match &props.validate {
                    Some(cb) => {
                        cb.validate(&(value.into(), Rc::clone(&list)))
                    }
                    None => Ok(()),
                }
            }
            _ => {
                bail!("no data loaded");
            }
        }
    })
}

impl<T: 'static> PwtSelector<T> {

    fn update_validator(&mut self, props: &Selector<T>) {
        self.real_validate = self.loader.with_state(|state| {
            log::info!("UPDATE SELECTOR VALIDATION FUNCTION");
            let data = match &state.data {
                Some(Ok(list)) => Some(Rc::clone(list)),
                _ => None,
            };
            create_selector_validation_cb(props.clone(), data)
        });

        // also update the FormContext validator
        if let Some(form_ctx) = &self.form_ctx {
            form_ctx.set_validate(&props.name, Some(self.real_validate.clone()));
        }
    }

    fn get_field_data(&self, props: &Selector<T>) -> (String, Result<(), String>) {
        if let Some(form_ctx) = &self.form_ctx {
            (
                form_ctx.get_field_text(&props.name),
                form_ctx.get_field_valid(&props.name),
            )
        } else {
            (
                self.value.clone(),
                self.real_validate.validate(&self.value.clone().into())
                    .map_err(|e| e.to_string())
            )
        }
    }

    fn set_value(&mut self, ctx: &Context<Self>, value: String) {
        if self.value == value { return; }

        let props = ctx.props();

        self.value = value.clone();

        if let Some(form_ctx) = &self.form_ctx {
            form_ctx.set_value(&props.name, value.into());
        }

        if let Some(on_select) = &props.on_select {
            on_select.emit(Key::from(self.value.clone()));
        }
    }
}

impl<T: 'static> Component for PwtSelector<T> {
    type Message = Msg;
    type Properties = Selector<T>;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let loader = Loader::new(ctx.link().callback(|_| Msg::UpdateList))
            .data(props.data.clone())
            .loader(props.loader.clone());

        loader.load();

        let value = String::new();

        let mut _form_ctx_handle = None;
        let mut form_ctx = None;

        let on_form_ctx_change = Callback::from({
            let link = ctx.link().clone();
            move |form_ctx: FormContext| link.send_message(Msg::FormCtxUpdate(form_ctx))
        });

        let real_validate = create_selector_validation_cb(props.clone(), props.data.clone());

        if let Some((form, handle)) = ctx.link().context::<FormContext>(on_form_ctx_change) {
            form.register_field(
                &props.name,
                value.clone().into(),
                Some(real_validate.clone()),
                FieldOptions::from_field_props(&props.input_props),
            );

            form_ctx = Some(form);
            _form_ctx_handle = Some(handle);
        }

        Self {
            _form_ctx_handle,
            form_ctx,
            value,
            loader,
            last_loader: props.loader.clone(),
            last_data: props.data.clone(),
            real_validate,

        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::FormCtxUpdate(form_ctx) => {
                let value = form_ctx.get_field_text(&props.name);
                self.form_ctx = Some(form_ctx);
                if self.value == value { return false; }
                self.value = value;
                true
            }
            Msg::UpdateList => {
                // update validation function, because the closure include the loaded data
                self.update_validator(props);

                let (value, _valid) = self.get_field_data(props);

                if self.loader.has_valid_data() {
                    if value.is_empty() {

                        let mut default = props.default.clone();

                        if default.is_none() && props.autoselect {
                            if let Some(extract_key) = &props.extract_key {
                                default = self.loader.with_state(|state| {
                                    match &state.data {
                                        Some(Ok(list)) => {
                                            list.get(0).map(|item| AttrValue::from(extract_key.apply(item).to_string()))
                                        }
                                        _ => None,
                                    }
                                });
                            }
                        }

                        if let Some(default) = default {
                            self.set_value(ctx, default.to_string().clone());
                            return true; // set_value already validates
                        }
                    }
                }
                true
            }
            Msg::Select(value) => {
                self.set_value(ctx, value);
                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        let props = ctx.props();

        let data_changed = !my_data_cmp_fn(&props.data, &self.last_data);

        if data_changed {
            self.last_data = props.data.clone();
            self.loader.set_data(props.data.clone());
        }

        if props.loader != self.last_loader {
            self.last_loader = props.loader.clone();
            self.loader.set_loader(props.loader.clone());
            self.loader.load();
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let (value, valid) = self.get_field_data(props);

        let picker = RenderFn::new({
            let value = value.clone();
            let loader = self.loader.clone();
            let props = props.clone();
            let picker = props.picker.clone();

            move |on_select: &Callback<Key>| {
                loader.render(|list: Rc<Vec<T>>| {
                    if list.is_empty() {
                        html!{<div class="pwt-p-2">{"List does not contain any items."}</div>}
                    } else {
                        picker.apply(&CreatePickerArgs {
                            list,
                            selected: Key::from(value.clone()),
                            on_select: on_select.clone(),
                        })
                    }
                })
            }
        });

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
