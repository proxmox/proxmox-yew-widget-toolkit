use serde_json::Value;

use yew::prelude::*;
use yew::html::{IntoEventCallback, Scope};

use crate::widget::form::{FieldOptions, FormContext, ValidateFn};

/// Text Field state handling
///
/// Handles FormContext interaction.
pub(crate) struct TextFieldStateHandle {
    form_ctx: Option<FormContext>,
    _form_ctx_handle: Option<ContextHandle<FormContext>>,
    name: Option<AttrValue>,
    on_change: Option<Callback<String>>,
    validate: Option<ValidateFn<Value>>,
    value: String,
    valid: Result<(), String>,
}

impl TextFieldStateHandle {

    pub fn new<COMP: Component>(
        scope: &Scope<COMP>,
        on_form_ctx_change: impl Into<Callback<FormContext>>,
        name: Option<AttrValue>,
        value: String,
        validate: Option<ValidateFn<Value>>,
        options: FieldOptions,
        on_change: impl IntoEventCallback<String>,
    ) -> Self {
        let mut _form_ctx_handle = None;
        let mut form_ctx = None;

        if let Some(name) = &name {

            let on_form_ctx_change = on_form_ctx_change.into();

            if let Some((form, handle)) = scope.context::<FormContext>(on_form_ctx_change) {
                form.register_field(
                    name,
                    value.clone().into(),
                    validate.clone(),
                    options,
                );

                form_ctx = Some(form);
                _form_ctx_handle = Some(handle);
            }
        }
        let valid = validate.as_ref().map(|v| {
            v.validate(&value.clone().into()).map_err(|e| e.to_string())
        }).unwrap_or(Ok(()));

        Self {
            _form_ctx_handle,
            form_ctx,
            value,
            valid,
            validate,
            on_change: on_change.into_event_callback(),
             name,
        }
    }

    pub fn update(&mut self, form_ctx: FormContext) -> bool {
        self.form_ctx = Some(form_ctx);
        let (value, valid) = self.get_field_data();
        let changed = self.value != value || self.valid != valid;
        self.value = value;
        self.valid = valid;
        changed
    }

    pub fn get_field_data(&self) -> (String, Result<(), String>) {
        if self.name.is_some() && self.form_ctx.is_some() {
            let name = self.name.as_ref().unwrap();
            let form_ctx = self.form_ctx.as_ref().unwrap();
            (
                form_ctx.get_field_text(name),
                form_ctx.get_field_valid(name),
            )
        } else {
            let value = self.value.clone();
            let valid = self.validate.as_ref().map(|v| {
                v.validate(&value.clone().into()).map_err(|e| e.to_string())
            }).unwrap_or(Ok(()));
            (value, valid)
        }
    }

    pub fn set_value(&mut self, value: String, set_default: bool) {

        if self.name.is_some() && self.form_ctx.is_some() {
            let name = self.name.as_ref().unwrap();
            let form_ctx = self.form_ctx.as_ref().unwrap();
            if set_default {
                form_ctx.set_default(name, value.clone().into());
            }
            form_ctx.set_value(name, value.into());
        } else {
            self.value = value.clone();
            self.valid = self.validate.as_ref().map(|v| {
                v.validate(&value.clone().into()).map_err(|e| e.to_string())
            }).unwrap_or(Ok(()));
        }

        if let Some(on_change) = &self.on_change {
            on_change.emit(self.value.clone());
        }
    }

    pub fn form_ctx(&self) -> Option<&FormContext> {
        self.form_ctx.as_ref()
    }
}
