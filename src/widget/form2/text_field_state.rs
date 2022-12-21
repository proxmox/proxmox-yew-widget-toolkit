use serde_json::Value;

use yew::prelude::*;

use crate::props::FieldStdProps;
use crate::widget::form::ValidateFn;

use super::{FieldHandle, FormContext, FormObserver};

pub(crate) struct TextFieldState {
    value: String,
    valid: Result<(), String>,
    real_validate: ValidateFn<Value>,
    form_ctx: Option<FormContext>,
    field_handle: Option<FieldHandle>,
    _form_ctx_handle: Option<ContextHandle<FormContext>>,
    _form_ctx_observer: Option<FormObserver>,
    on_form_data_change: Callback<FormContext>,
}

impl TextFieldState {

    pub fn create<COMP: Component>(
        ctx: &Context<COMP>,
        props: &FieldStdProps,
        on_form_ctx_change: Callback<FormContext>,
        on_form_data_change: Callback<FormContext>,
        real_validate: ValidateFn<Value>,
    ) -> TextFieldState {

        let mut state = Self {
            value: String::new(),
            valid: Ok(()),
            real_validate,
            form_ctx: None,
            field_handle: None,
            _form_ctx_handle: None,
            _form_ctx_observer: None,
            on_form_data_change: on_form_data_change.clone(),
        };

        if props.name.is_some() {
            if let Some((form, handle)) = ctx.link().context::<FormContext>(on_form_ctx_change) {
                state._form_ctx_handle = Some(handle);
                state._form_ctx_observer = Some(form.add_listener(on_form_data_change));
                state.form_ctx = Some(form);
            }
        }

        state
    }

    /// Get current field value and validation result.
    pub fn get_field_data(&self) -> (String, Result<(), String>) {
        if let Some(field_handle) = &self.field_handle {
            let value = field_handle.get_text();
            let valid = field_handle.get_valid();
            (value, valid)
        } else {
            (self.value.clone(), self.valid.clone())
        }
    }

    /// Force value - for fields without name (no FormContext)
    pub fn force_value(
        &mut self,
        value: String,
        valid: Option<Result<(), String>>,
    ) {
        self.valid = valid.unwrap_or_else(|| {
            self.real_validate.validate(&value.clone().into())
                .map_err(|e| e.to_string())
        });
        self.value = value;
    }

    /// Set the field value
    pub fn set_value(&mut self, value: String) {
        if let Some(field_handle) = &mut self.field_handle {
            field_handle.set_value(value.clone().into());
        } else {
            self.value = value.clone();
            self.valid = self.real_validate.validate(&value.clone().into())
                .map_err(|e| e.to_string());
        }
    }

    /// Register the field inm the FormContext
    pub fn register_field(&mut self, props: &FieldStdProps, value: String) {
        let name = match &props.name {
            Some(name) => name,
            None => {
                self.field_handle = None;
                return;
            }
        };

        let form_ctx = match &self.form_ctx {
            Some(form_ctx) => form_ctx.clone(),
            None => {
                self.field_handle = None;
                return;
            }
        };

        let field_handle = form_ctx.register_field(
            name,
            value.into(),
            Some(self.real_validate.clone()),
            props.submit,
            props.submit_empty,
        );

        self.field_handle = Some(field_handle);
    }

    pub fn update_form_context_hook(&mut self, props: &FieldStdProps, form_ctx: FormContext) -> bool {
        self._form_ctx_observer = Some(form_ctx.add_listener(self.on_form_data_change.clone()));
        self.form_ctx = Some(form_ctx);
        self.register_field(props, self.value.clone());
        true
    }

    pub fn update_form_data_hook(&mut self) -> bool {
        if let Some(field_handle) = &self.field_handle {
            let value = field_handle.get_text();
            if value != self.value {
                self.value = value;
                return true;
            }
        }
        false
    }
}
