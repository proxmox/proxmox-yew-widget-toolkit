use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use serde_json::{json, Value};

use yew::prelude::*;

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
}

#[derive(Clone, PartialEq)]
pub struct FieldState {
    pub initial_value: Value,
    pub initial_valid: Result<(), String>,
    pub value: Value,
    pub valid: Result<(), String>,

    options: FieldOptions,
    version: usize, // to trigger property change
    changed: bool, // flag to indicate changes
}


#[derive(Clone, PartialEq, Properties)]
pub struct FieldFormRef {
    pub field_name: String,
    pub form: FormState,
    pub version: usize,
}

impl FieldFormRef {

    pub(crate) fn register_field(
        &self,
        value: Value,
        valid: Result<(), String>,
        options: FieldOptions,
    ) {
        let state = FieldState {
            initial_value: value.clone(),
            initial_valid: valid.clone(),
            value: value,
            valid,
            options,
            version: 1,
            changed: false,
        };
        self.form.set_field_state(&self.field_name, state);
    }

    pub(crate) fn get_valid(&self) -> Result<(), String> {
        self.form.get_field_valid(&self.field_name)
    }

    pub(crate) fn get_value(&self) -> Value {
        self.form.get_field_value(&self.field_name)
    }

    pub(crate) fn get_text(&self) -> String {
        self.form.get_field_text(&self.field_name)
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct ButtonFormRef {
    pub form: FormState,
    pub valid: bool,
    pub dirty: bool,
}

#[derive(Clone, Properties)]
pub struct FormState {
    data: Rc<RefCell<FormStateInner>>,
    onchange: Callback<()>, // fixme: rename to on_change
}

// do not compare content, only update on version change
impl PartialEq for FormState {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.data, &other.data)
    }
}

pub struct FormStateInner {
    field_state: HashMap<String, FieldState>,
    loaded: bool,
    show_advanced: bool,
}

impl FormStateInner {
    pub fn new() -> Self {
        Self {
            field_state: HashMap::new(),
            loaded: false,
            show_advanced: false,
        }
    }
}

impl FormState {
    pub fn new(onchange: Callback<()>) -> Self {
        Self {
            data: Rc::new(RefCell::new(FormStateInner::new())),
            onchange,
        }
    }

    /// Returns the loaded flag (see [Self::load_form])
    pub fn loaded(&self) -> bool {
        self.data.borrow().loaded
    }

    /// Returns the show_advanced flag
    pub fn show_advanced(&self) -> bool {
        self.data.borrow().show_advanced
    }

    /// Set the show_advanced flag
    pub fn set_show_advanced(&self, show_advanced: bool) {
        let mut form = self.data.borrow_mut();

        if form.show_advanced != show_advanced {
            form.show_advanced = show_advanced;
            self.onchange.emit(());
        }
    }

    pub fn field_ref(&self, field_name: &str) -> FieldFormRef {
        let version = self.data.borrow()
            .field_state
            .get(field_name)
            .map(|field| field.version)
            .unwrap_or(0);

        FieldFormRef {
            form: self.clone(),
            field_name: field_name.to_string(),
            version,
        }
    }

    pub fn button_ref(&self) -> ButtonFormRef {
        ButtonFormRef {
            form: self.clone(),
            valid: self.valid(),
            dirty: self.dirty(),
        }
    }

    pub fn reset_form(&self) {
        for field in self.data.borrow_mut().field_state.values_mut() {
            if field.value != field.initial_value {
                field.value = field.initial_value.clone();
                field.valid = field.initial_valid.clone();
                field.version += 1;
                field.changed = true;
            }
        }
        self.onchange.emit(());
    }

    /// Reset a field back to the initial values
    pub fn reset_field(&self, name: &str) {
        if let Some(field) = self.data.borrow_mut().field_state.get_mut(name) {
           if field.value != field.initial_value {
               field.value = field.initial_value.clone();
               field.valid = field.initial_valid.clone();
               field.version += 1;
               field.changed = true;
               self.onchange.emit(());
            }
        }
    }

    /// Clear a field (set to Value::Null)
    pub fn clear_field(&self, name: &str) {
        if let Some(field) = self.data.borrow_mut().field_state.get_mut(name) {
            if field.value != Value::Null {
                field.value = Value::Null;
                field.valid = Ok(());
                field.version += 1;
                field.changed = true;
                self.onchange.emit(());
            }
        }
    }

    /// Set form data and 'loaded' flag
    ///
    /// This also clears the changed flag for all fields
    pub fn load_form(&self, data: Value) {
        let mut form = self.data.borrow_mut();
        for (name, field) in form.field_state.iter_mut() {
            field.initial_value = data[name].clone();
            field.initial_valid = Ok(());
            field.value = field.initial_value.clone();
            field.valid = field.initial_valid.clone();
            field.version += 1;
            field.changed = false;
        }
        form.loaded = true;
        self.onchange.emit(());
    }

    pub fn get_submit_data(&self) -> Value {
        let mut data = json!({});

        for (name, state) in self.data.borrow().field_state.iter() {
            if !state.options.submit { continue; }
            let value = state.value.clone();
            match value {
                Value::Null => { /* do not include */ },
                Value::Bool(v) => {
                    data[name] = Value::Bool(v);
                },
                Value::String(v) => {
                    if !v.is_empty() || state.options.submit_empty {
                        data[name] = Value::String(v);
                    }
                }
                // Array/Object
                v => data[name] = v,
            }
        }

        data
    }

    /// Returns the changed flag, and reset it back to false
    pub fn get_field_changed(&self, name: &str) -> bool {
        self.data.borrow_mut()
            .field_state
            .get_mut(name)
            .map(|state| {
                let changed = state.changed;
                state.changed = false;
                changed
            })
            .unwrap_or(false)
    }

    pub fn get_field_valid(&self, name: &str) -> Result<(), String> {
        self.data.borrow()
            .field_state
            .get(name)
            .map(|state| state.valid.clone())
            .unwrap_or(Ok(()))
    }

    pub fn get_field_value(&self, name: &str) -> Value {
        self.data.borrow()
            .field_state
            .get(name)
            .map(|state| state.value.clone())
            .unwrap_or(Value::Null)

    }

    pub fn get_field_text(&self, name: &str) -> String {
        self.get_field_value(name)
            .as_str()
            .unwrap_or("")
            .to_string()
    }

    pub(crate) fn set_field_state(
        &self,
        name: &str,
        state: FieldState,
    ) {
        self.data.borrow_mut().field_state.insert(name.to_owned(), state);
        self.onchange.emit(());
    }

    pub(crate) fn with_field_state_mut(
        &self,
        name: &str,
        cb: impl Fn(&mut FieldState),
    ) {
        let mut form = self.data.borrow_mut();
        match form.field_state.get_mut(name) {
            None => return,
            Some(state) => {
                let old = state.clone();
                cb(state);
                if &old != state {
                    state.version += 1;
                    state.changed = true;
                    self.onchange.emit(());
                }
            }
        }
    }

    pub fn valid(&self) -> bool {
        for state in self.data.borrow().field_state.values() {
            if state.valid.is_err() {
                return false;
            }
        }
        true
    }

    pub fn dirty(&self) -> bool {
        for (_name, state) in &self.data.borrow().field_state {
            if state.value != state.initial_value {
                //log::info!("DIRTY {} {:?} != {:?}", name, state.value, state.initial_value);
                return true;
            }
        }
        false
    }

}

// Propxmox API related helpers

pub fn delete_empty_values(record: &Value, param_list: &[&str]) -> Value {
    let mut new = json!({});
    let mut delete: Vec<String> = Vec::new();

    for (param, v) in record.as_object().unwrap().iter() {
        if !param_list.contains(&param.as_str()) {
            new[param] = v.clone();
            continue;
        }
        if v.is_null() || (v.is_string() && v.as_str().unwrap().is_empty()) {
            delete.push(param.to_string());
        } else {
            new[param] = v.clone();
        }
    }

    for param in param_list {
        if record.get(param).is_none() {
            delete.push(param.to_string());
        }
    }

    new["delete"] = delete.into();

    new
}
