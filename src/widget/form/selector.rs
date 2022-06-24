use std::rc::Rc;

use anyhow::Error;

use yew::prelude::*;
use yew::virtual_dom::Key;
use yew::html::{IntoEventCallback, IntoPropValue};

use proxmox_schema::Schema;

use crate::prelude::*;
use crate::props::{ExtractKeyFn, IntoLoadCallback, LoadCallback, RenderFn};
use crate::state::Loader;
use crate::widget::Dropdown;
use crate::widget::form::ValidateFn;

use pwt_macros::widget;

pub struct CreatePickerArgs<T> {
    pub list: Rc<Vec<T>>,
    pub selected: Key,
    pub on_select: Callback<String>,
}

/// Combobox like selector
///
/// Supports async data loading and generic picker widget
/// implementations.
///
/// Note: Please use a trackable [LoadCallback] to avoid unnecessary
/// reloads.
#[widget(PwtSelector<T>, @input, @element)]
#[derive(Clone, Properties)]
pub struct Selector<T: 'static> {
    pub loader: Option<LoadCallback<Vec<T>>>,
    pub data: Option<Rc<Vec<T>>>,
    pub default: Option<String>,
    #[prop_or_default]
    pub editable: bool,
    #[prop_or_default]
    pub autoselect: bool,
    /// Extract Key from item
    ///
    /// Onyl use to auto-select the first entry (if default is not set)
    pub extract_key: Option<ExtractKeyFn<T>>,
    pub on_select: Option<Callback<Key>>,
    pub picker: RenderFn<CreatePickerArgs<T>>,
    pub validate: Option<ValidateFn<(String, Rc<Vec<T>>)>>,
}

impl<T> PartialEq for Selector<T> {
    fn eq(&self, other: &Self) -> bool {
        match (&self.data, &other.data) {
            (None, None) => { /* fall through */ }
            (None, Some(_)) | (Some(_), None) => return false,
            (Some(ref d1), Some(ref d2)) => {
                if !Rc::ptr_eq(d1, d2) {
                    return false;
                }
            }
        }

        self.std_props == other.std_props &&
            self.input_props == other.input_props &&
            self.loader == other.loader &&
            self.default == other.default &&
            self.editable == other.editable &&
            self.extract_key == other.extract_key &&
            self.on_select == other.on_select &&
            self.validate == other.validate
    }
}

impl<T: 'static> Selector<T> {

    pub fn new(picker: RenderFn<CreatePickerArgs<T>>) -> Self {
        yew::props!(Self { picker })
    }

    pub fn default(mut self, default: impl IntoPropValue<Option<String>>) -> Self {
        self.set_default(default);
        self
    }

    pub fn set_default(&mut self, default: impl IntoPropValue<Option<String>>) {
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
    pub fn schema(mut self, schema: &'static Schema, optional: bool) -> Self {
        self.set_schema(schema, optional);
        self
    }

    /// Method to set the validation schema
    pub fn set_schema(&mut self, schema: &'static Schema, optional: bool) {
        self.validate = Some(ValidateFn::new(move |(value, _list): &(String, _)| {
            if optional && value.is_empty() {
                Ok(())
            } else {
                schema.parse_simple_value(&value)?;
                Ok(())
            }
        }));
    }

    pub fn on_select(mut self, cb: impl IntoEventCallback<Key>) -> Self {
        self.on_select = cb.into_event_callback();
        self
    }
}

pub enum Msg {
    Select(String),
    UpdateList,
}

pub struct PwtSelector<T> {
    loader: Loader<Vec<T>>,
    last_loader: Option<LoadCallback<Vec<T>>>, //change tracking
    value: String,
    valid: Result<(), String>,
}

impl<T: 'static> PwtSelector<T> {

    fn validate(&self, ctx: &Context<Self>, value: &str) -> Result<(), String> {
        let props = ctx.props();

        if value.is_empty() {
            if props.input_props.required {
                return Err(String::from("Field may not be empty."));
            } else {
                return Ok(())
            }
        }

        self.loader.with_state(|state| {
            match &state.data {
                Some(Ok(list)) => {
                    match &props.validate {
                        Some(cb) => {
                            cb.validate(&(value.into(), Rc::clone(list))).map_err(|e| e.to_string())
                        }
                        None => Ok(()),
                    }
                }
                _ => {
                    Err(String::from("no data loaded"))
                }
            }
        })
    }

    fn get_valid(&self, ctx: &Context<Self>) -> Result<(), String> {
        match &ctx.props().input_props.form_ref {
            Some(form_ref) => form_ref.get_valid(),
            None => self.valid.clone()
        }
    }

    fn get_value(&self, ctx: &Context<Self>) -> String {
        match  &ctx.props().input_props.form_ref {
            Some(form_ref) => form_ref.get_text(),
            None => self.value.clone(),
        }
    }

    fn set_value(&mut self, ctx: &Context<Self>, value: String, default: Option<String>) {
        let props = ctx.props();

        self.value = value;
        self.valid = self.validate(ctx, &self.value);

        if let Some(form_ref) = &props.input_props.form_ref {
            form_ref.form.with_field_state_mut(&form_ref.field_name, |field| {
                field.value = self.value.clone().into();
                field.valid = self.valid.clone();
                if let Some(default) = &default {
                    field.initial_valid = self.validate(ctx, &default);
                    field.initial_value = default.clone().into();
                 }
            });
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

        let mut me = Self {
            value,
            valid: Ok(()),
            loader,
            last_loader: props.loader.clone(),
        };

        me.valid = me.validate(ctx, &me.value);

        props.input_props.register_form_field(me.value.clone().into(), me.valid.clone());

        me
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateList => {
                let props = ctx.props();
                let value = self.get_value(ctx);

                if self.loader.has_valid_data() {
                     if value.is_empty() {

                        let mut default = props.default.clone();

                        if default.is_none() && props.autoselect {
                            if let Some(extract_key) = &props.extract_key {
                                default = self.loader.with_state(|state| {
                                    match &state.data {
                                        Some(Ok(list)) => {
                                            list.get(0).map(|item| extract_key.apply(item).to_string())
                                        }
                                        _ => None,
                                    }
                                });
                            }
                        }

                        if let Some(default) = default {
                            self.set_value(ctx, default.clone(), Some(default));
                            return true; // set_value already validates
                        }
                    }
                }

                // alway re-validate value
                self.valid = self.validate(ctx, &value);
                if let Some(form_ref) = &props.input_props.form_ref {
                    form_ref.form.with_field_state_mut(&form_ref.field_name, |field| {
                        field.valid = self.valid.clone();
                    })
                }

                true
            }
            Msg::Select(value) => {
                self.set_value(ctx, value, None);
                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        let props = ctx.props();

        if props.loader != self.last_loader {
            self.last_loader = props.loader.clone();
            self.loader.set_loader(props.loader.clone());
            self.loader.load();
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let valid = self.get_valid(ctx);
        let value = self.get_value(ctx);

        let picker = RenderFn::new({
            let value = value.clone();
            let loader = self.loader.clone();
            let props = props.clone();
            let picker = props.picker.clone();

            move |on_select: &Callback<String>| {
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
