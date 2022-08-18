use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use serde_json::Value;

use yew::prelude::*;
use yew::html::IntoPropValue;

use crate::props::FieldStdProps;
use crate::widget::form::ValidateFn;

#[derive(Clone, Debug, PartialEq)]
pub struct FieldOptions {
    pub submit: bool,
    pub submit_empty: bool,
}

impl FieldOptions {
    pub fn new() -> Self {
        Self {
            submit: true,
            submit_empty: false,
        }
    }

    pub fn from_field_props(props: &FieldStdProps) -> Self {
        Self {
            submit: props.submit,
            submit_empty: props.submit_empty,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FieldState {
    pub validate: Option<ValidateFn<Value>>,
    pub initial_value: Value,
    pub initial_valid: Result<(), String>,
    pub value: Value,
    pub valid: Result<(), String>,
    options: FieldOptions,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FormContext {
    pub(crate) ctr: usize,
    on_change: Callback<()>,
    inner: Rc<RefCell<FormContextInner>>,
}

#[derive(Debug, PartialEq)]
pub struct FormContextInner {
    field_state: HashMap<AttrValue, FieldState>,
}

impl FormContextInner {
    pub fn new() -> Self {
        Self {
            field_state: HashMap::new(),
        }
    }
}

impl FormContext {

    pub fn new(on_change: Callback<()>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(FormContextInner::new())),
            on_change,
            ctr: 0,
        }
    }

    pub fn register_field(
        &self,
        name: impl IntoPropValue<AttrValue>,
        value: Value,
        validate: Option<ValidateFn<Value>>,
        options: FieldOptions,
    ) {
        let valid = if let Some(validate) = &validate {
            validate.validate(&value)
                .map_err(|e| e.to_string())
        } else {
            Ok(())
        };

        let state = FieldState {
            validate,
            initial_value: value.clone(),
            initial_valid: valid.clone(),
            value: value,
            valid: valid,
            options,
        };
        self.set_field_state(name, state);
    }

    pub fn get_field_valid(&self, name: impl IntoPropValue<AttrValue>) -> Result<(), String> {
        let name = name.into_prop_value();
        self.inner.borrow()
            .field_state
            .get(&name)
            .map(|state| state.valid.clone())
            .unwrap_or(Ok(()))
    }

    pub fn get_field_value(&self, name: impl IntoPropValue<AttrValue>) -> Value {
        let name = name.into_prop_value();
        self.inner.borrow()
            .field_state
            .get(&name)
            .map(|state| state.value.clone())
            .unwrap_or(Value::Null)

    }

    pub fn get_field_text(&self, name: impl IntoPropValue<AttrValue>) -> String {
        self.get_field_value(name)
            .as_str()
            .unwrap_or("")
            .to_string()
    }

    pub fn set_value(
        &self,
        name: impl IntoPropValue<AttrValue>,
        value: Value,
    ) {
        let name = name.into_prop_value();
        self.with_field_state_mut(&name, move |state| {
            state.value = value;
        });

        self.validate_field(&name);
    }

    /// Trigger re-validation of some field
    pub fn validate_field(
        &self,
        name: impl IntoPropValue<AttrValue>,
    ) {
        let name = name.into_prop_value();

        // Note: borrow read-only during validation, so that validate
        // callback can read other field values.
        let valid = match self.inner.borrow().field_state.get(&name) {
            None => {
                log::error!("FormContext::validate_field failed - field {name} not registered");
                return;
            }
            Some(state) => {
                match &state.validate {
                    Some(cb) => cb.validate(&state.value).map_err(|e| e.to_string()),
                    None => Ok(()),
                }
            }
        };

        self.with_field_state_mut(&name, move |state| {
            state.valid = valid;
        });
    }

    pub fn reset_form(&self) {
        for field in self.inner.borrow_mut().field_state.values_mut() {
            if field.value != field.initial_value {
                field.value = field.initial_value.clone();
                field.valid = field.initial_valid.clone();
                //field.version += 1;
                //field.changed = true;
            }
        }
        self.on_change.emit(());
    }

    pub fn dirty(&self) -> bool {
        for (_name, state) in &self.inner.borrow().field_state {
            if state.value != state.initial_value {
                return true;
            }
        }
        false
    }

    fn set_field_state(
        &self,
        name: impl IntoPropValue<AttrValue>,
        state: FieldState,
    ) {
        self.inner.borrow_mut().field_state.insert(name.into_prop_value(), state);
        self.on_change.emit(());
    }

    fn with_field_state_mut(
        &self,
        name: &AttrValue,
        cb: impl FnOnce(&mut FieldState),
    ) {
        let mut form = self.inner.borrow_mut();
        let changed = match form.field_state.get_mut(name) {
            None => {
                log::error!("FormContext::set_values failed - field {name} not registered");
                return;
            }
            Some(state) => {
                let old = state.clone();
                cb(state);
                &old != state
            }
        };
        drop(form); // release borrow_mut
        if changed {
            self.on_change.emit(());
        }
    }

}
