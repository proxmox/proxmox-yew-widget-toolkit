use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;

use serde_json::{json, Value};

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

/// Form Context
///
/// This struct is used by form like widgets ([crate::widget::form2::Form],
/// [crate::component::EditWindow]) to provide a shared state for all
/// input widgets.
#[derive(Clone, Debug, PartialEq)]
pub struct FormContext {
    ctr: usize, // property change trigger
    on_change: Callback<()>,
    inner: Rc<RefCell<FormContextInner>>,
}

#[derive(Debug, PartialEq)]
pub struct FormContextInner {
    change_trackers: HashMap<AttrValue, HashSet<AttrValue>>,
    field_state: HashMap<AttrValue, FieldState>,
    loaded: bool,
}

impl FormContextInner {
    pub fn new() -> Self {
        Self {
            change_trackers: HashMap::new(),
            field_state: HashMap::new(),
            loaded: false,
        }
    }

    fn set_field_changed(
        &mut self,
        name: &AttrValue,
    ) {
        for (_tracker_id, set) in &mut self.change_trackers {
            set.insert(name.clone());
        }
    }

    fn clear_change_trackers(&mut self) {
        for (_tracker_id, set) in &mut self.change_trackers {
            set.clear();
        }
    }
}

impl FormContext {

    /// Create a new instance.
    ///
    /// The `on_change` callback should call [Self::context_change_trigger]
    /// to notify the ContextProvider.
    pub fn new(on_change: Callback<()>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(FormContextInner::new())),
            on_change,
            ctr: 0,
        }
    }

    /// Trigger a context change.
    ///
    /// This simply increases an internal counter, so that the
    /// [ContextProvider] gets aware of the state change.
    pub fn context_change_trigger(&mut self) {
        self.ctr += 1;
    }

    /// Register a form field.
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

    /// Get the result from the validation function.
    ///
    /// Returns `Ok(())` for non-existent fields.
    pub fn get_field_valid(&self, name: impl IntoPropValue<AttrValue>) -> Result<(), String> {
        let name = name.into_prop_value();
        self.inner.borrow()
            .field_state
            .get(&name)
            .map(|state| state.valid.clone())
            .unwrap_or(Ok(()))
    }

    /// Get the field value.
    ///
    /// Returns [Value::Null] for non-existent fields.
    pub fn get_field_value(&self, name: impl IntoPropValue<AttrValue>) -> Value {
        let name = name.into_prop_value();
        self.inner.borrow()
            .field_state
            .get(&name)
            .map(|state| state.value.clone())
            .unwrap_or(Value::Null)

    }

    /// Get the field value as string.
    ///
    /// Return the empty string for non-existent fields, or
    /// when the field value is not a string.
    pub fn get_field_text(&self, name: impl IntoPropValue<AttrValue>) -> String {
        self.get_field_value(name)
            .as_str()
            .unwrap_or("")
            .to_string()
    }

    /// Get form submit data.
    ///
    /// Returns a JSON object with the values of all registered fields
    /// that have [FieldOptions::submit] set. Empty strings are
    /// included when [FieldOptions::submit_empty] is set.
    pub fn get_submit_data(&self) -> Value {
        let mut data = json!({});

        for (name, state) in self.inner.borrow().field_state.iter() {
            if !state.options.submit { continue; }
            let value = state.value.clone();
            match value {
                Value::Null => { /* do not include */ },
                Value::String(v) => {
                    if !v.is_empty() || state.options.submit_empty {
                        data[name.deref()] = Value::String(v);
                    }
                }
                // Bool/Number/Array/Object
                v => data[name.deref()] = v,
            }
        }

        data
    }


    /// Set a field value.
    ///
    /// This calls the field validate callback.
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

    /// Set a field validation callback.
    ///
    /// This automatically re-validates the field value.
    pub fn set_validate(
        &self,
        name: impl IntoPropValue<AttrValue>,
        validate: Option<ValidateFn<Value>>,
    ) {
        let name = name.into_prop_value();
        self.with_field_state_mut(&name, move |state| {
            state.validate = validate;
        });

        self.validate_field(&name);
    }

    /// Trigger re-validation of some field.
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

    /// Reset all fields to their initial value.
    pub fn reset_form(&self) {
        let mut changes = HashSet::new();
        let mut form = self.inner.borrow_mut();
        {
            for (name, field) in form.field_state.iter_mut() {
                if field.value != field.initial_value {
                    field.value = field.initial_value.clone();
                    field.valid = field.initial_valid.clone();
                    changes.insert(name.clone());
                    //field.version += 1;
                    //field.changed = true;
                }
            }
        }

        if !changes.is_empty() {
            for name in changes.into_iter() {
                form.set_field_changed(&name);
            }
            self.on_change.emit(());
        }
    }

    /// Reset a single field back to its initial value.
    pub fn reset_field(&self, name: impl IntoPropValue<AttrValue>) {
        let name = name.into_prop_value();
        let mut changes = false;
        let mut form = self.inner.borrow_mut();
        if let Some(field) = form.field_state.get_mut(&name) {
           if field.value != field.initial_value {
               field.value = field.initial_value.clone();
               field.valid = field.initial_valid.clone();
               changes = true;
               //field.version += 1;
               //field.changed = true;
            }
        }
        if changes {
            form.set_field_changed(&name);
            self.on_change.emit(());
        }
    }

    /// Register a new change tracker.
    ///
    /// After registering, use [Self::get_field_changed] to check if a
    /// field value has changed.
    pub fn register_change_tracker(&self, tracker_id: impl IntoPropValue<AttrValue>) {
        let tracker_id = tracker_id.into_prop_value();
         let mut form = self.inner.borrow_mut();
        form.change_trackers.entry(tracker_id).or_insert(HashSet::new());
    }

    /// Set form data and 'loaded' flag
    ///
    /// This sets the form data from the provided JSON object.
    /// This also clears the changed flag for all fields (in all change trackers).
    pub fn load_form(&self, data: Value) {
        let mut form = self.inner.borrow_mut();
        for (name, field) in form.field_state.iter_mut() {
            field.initial_value = data[name.deref()].clone();
            field.initial_valid = if let Some(validate) = &field.validate {
                validate.validate(&field.initial_value)
                    .map_err(|e| e.to_string())
            } else {
                Ok(())
            };
            field.value = field.initial_value.clone();
            field.valid = field.initial_valid.clone();
            //field.version += 1;
            //field.changed = false;
        }
        form.loaded = true;
        form.clear_change_trackers();
        self.on_change.emit(());
    }

    /// Returns the changed flag, and reset it back to false
    ///
    /// It is necessary to call [Self::register_change_tracker] before
    /// using this function (to start change tracking).
    pub fn get_field_changed(
        &self,
        tracker_id: impl IntoPropValue<AttrValue>,
        name: impl IntoPropValue<AttrValue>,
    ) -> bool {
        let tracker_id = tracker_id.into_prop_value();
        let name = name.into_prop_value();
        let mut form = self.inner.borrow_mut();
        let map = form.change_trackers.entry(tracker_id).or_insert(HashSet::new());
        map.remove(&name)
    }

    /// Returns the loaded flag (see [Self::load_form])
    pub fn loaded(&self) -> bool {
        self.inner.borrow().loaded
    }

    /// Returns true if a field value differs from its initial value.
    pub fn dirty(&self) -> bool {
        for (_name, state) in &self.inner.borrow().field_state {
            if state.value != state.initial_value {
                return true;
            }
        }
        false
    }

    /// Returns true if all fields values are valid.
    pub fn valid(&self) -> bool {
        for state in self.inner.borrow().field_state.values() {
            if state.valid.is_err() {
                return false;
            }
        }
        true
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
        if changed {
            form.set_field_changed(&name);
        }

        drop(form); // release borrow_mut

        if changed {
            self.on_change.emit(());
        }
    }

}
