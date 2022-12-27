//! FormContext - shared form data.

use std::rc::Rc;
use std::cell::{Ref, RefMut, RefCell};
use std::ops::{Deref, DerefMut};
use std::mem::ManuallyDrop;
use std::collections::HashMap;

use derivative::Derivative;
use slab::Slab;
use serde_json::{json, Value};

use yew::prelude::*;
use yew::html::{IntoEventCallback, IntoPropValue};

use crate::state::optional_rc_ptr_eq;

use super::ValidateFn;

/// Basic field options used inside [FormContext].
///
/// This basically mirrors the (non-display) options from
/// [FieldStdProps](crate::props::FieldStdProps).
#[derive(Debug, PartialEq)]
pub struct FieldOptions {
    /// Include the field data in the submit request.
    pub submit: bool,
    /// Include the field data in the submit request even if its
    /// empty.
    pub submit_empty: bool,
    /// Field required flag.
    pub required: bool,
    /// Field disabled flag.
    pub disabled: bool,
}

#[derive(Debug, PartialEq)]
struct FieldRegistration {
    pub name: AttrValue,
    pub validate: Option<ValidateFn<Value>>,
    pub radio_group: bool,
    pub options: FieldOptions,
    pub value: Value,
    pub default: Value,
    pub valid: Result<(), String>,
}

/// Shared form data ([Rc]<[RefCell]<[FormContextState]>>)
///
/// This shared object can be used to control input fields. The
/// [Form](super::Form) widget uses a
/// [ContextProvider](yew::context::ContextProvider), so that fields
/// inside a form can automatically access the [FormContext] to store
/// data.
///
/// Fields listens to context changes, and are automatically updated
/// and validated when you modify the context.
///
/// The context is also the best place to gather data for a form
/// submit (see: [FormContext::get_submit_data]).
#[derive(Derivative)]
#[derivative(Clone, PartialEq)]
pub struct FormContext {
    // Allow to store one StoreObserver here (for convenience)
    #[derivative(PartialEq(compare_with="optional_rc_ptr_eq"))]
    on_change: Option<Rc<FormContextObserver>>,
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    inner: Rc<RefCell<FormContextState>>,
}

/// Owns the listener callback. When dropped, the
/// listener callback will be removed from the [FormContext].
pub struct FormContextObserver {
    key: usize,
    inner: Rc<RefCell<FormContextState>>,
}

impl Drop for FormContextObserver {
    fn drop(&mut self) {
        self.inner.borrow_mut().remove_listener(self.key);
    }
}

/// Owns the field registration. When dropped, the
/// field will be removed from the [FormContext].
pub struct FieldHandle {
    key: usize,
    form_ctx: FormContext,
}

impl FieldHandle {

    /// Lock the form context for read access.
    pub fn read(&self) -> FormContextReadGuard {
        self.form_ctx.read()
    }

    /// Lock the form context for write access.
    ///
    /// Automatically notifies listeners when the guard is dropped.
    pub fn write(&self) -> FormContextWriteGuard {
        self.form_ctx.write()
    }

    /// Returns the field value.
    pub fn get_value(&self) -> Value {
        self.get_data().0
    }

    /// Returns the field validation status.
    pub fn get_valid(&self) -> Result<(), String> {
        self.get_data().1
    }

    /// Returns the field value with the validation result.
    pub fn get_data(&self) -> (Value, Result<(), String>) {
        let key = self.key;
        self.read().get_field_data_by_slab_key(key)
    }

    /// Set the field value
    pub fn set_value(&mut self, value: Value) {
        let key = self.key;
        self.write().set_field_value_by_slab_key(key, value, false);
    }

    /// Set the field default value
    pub fn set_default(&mut self, default: Value) {
        let key = self.key;
        self.write().set_field_default_by_slab_key(key, default);
    }

    /// Trigger re-validation
    pub fn validate(&mut self) {
        let key = self.key;
        self.write().validate_field_by_slab_key(key);
    }

    pub fn update_field_options(&mut self, options: FieldOptions) {
        let key = self.key;
        self.write().update_field_options_by_slab_key(key, options);
    }
}

impl Drop for FieldHandle {
    fn drop(&mut self) {
        self.form_ctx.inner.borrow_mut().unregister_field(self.key);
    }
}

impl FormContext {

    pub fn new() -> Self {
        Self {
            on_change: None,
            inner: Rc::new(RefCell::new(FormContextState::new())),
        }
    }

    /// Builder style method to set the on_change callback.
    ///
    /// This calls [Self::add_listener] to create a new
    /// [FormContextObserver]. The observer is stored inside the
    /// [FormContext] object, so each clone can hold a single on_change
    /// callback.
    pub fn on_change(mut self, cb: impl IntoEventCallback<FormContext>) -> Self {
        self.on_change = match cb.into_event_callback() {
            Some(cb) => Some(Rc::new(self.add_listener(cb))),
            None => None,
        };
        self
    }

    /// Method to add an change observer.
    ///
    /// This is usually called by [Self::on_change], which stores the
    /// observer inside the [FormContext] object.
    pub fn add_listener(&self, cb: impl Into<Callback<FormContext>>) -> FormContextObserver {
        let key = self.inner.borrow_mut()
            .add_listener(cb.into());
        FormContextObserver { key, inner: self.inner.clone() }
    }

    fn notify_listeners(&self) {
        let listeners = self.inner.borrow().listeners.clone(); // clone to avoid borrow()
        for (_key, listener) in listeners.iter() {
            listener.emit(self.clone());
        }
    }

    /// Lock the form context for read access.
    pub fn read(&self) -> FormContextReadGuard {
        FormContextReadGuard {
            state: self.inner.borrow(),
        }
    }

    /// Lock the form context for write access.
    ///
    /// Automatically notifies listeners when the guard is dropped.
    pub fn write(&self) -> FormContextWriteGuard {
        let cloned_self = Self { on_change: None, inner: self.inner.clone() };
        let state = ManuallyDrop::new(self.inner.borrow_mut());
        FormContextWriteGuard {
            form_ctx: cloned_self,
            initial_version: state.version,
            state,
        }
    }

    /// Register a form field.
    ///
    /// The returned handle owns the registration. The registration is
    /// automatically removed when you drow the handle.
    pub fn register_field(
        &self,
        name: impl IntoPropValue<AttrValue>,
        value: Value,
        default: Value,
        radio_group: bool,
        validate: Option<ValidateFn<Value>>,
        options: FieldOptions,
    ) -> FieldHandle {
        let key = self.inner.borrow_mut()
            .register_field(name, value, default, radio_group, validate, options);

        FieldHandle { key, form_ctx: self.clone() }
    }

    /// Returns the show_advanced flag
    pub fn get_show_advanced(&self) -> bool {
        self.inner.borrow().show_advanced
    }

    /// Set the show_advanced flag
    pub fn set_show_advanced(&self, show_advanced: bool) {
        self.write().set_show_advanced(show_advanced);
    }

    /// Load form data.
    ///
    /// This sets the form data from the provided JSON object. Also
    /// clears the changed flag for all fields by setting the default
    /// to the provided data.
    pub fn load_form(&self, data: Value) {
        self.write().load_form(data);
    }

    /// Get form submit data.
    ///
    /// Returns a JSON object with the values of all registered fields
    /// that have [FieldOptions::submit] set. Empty strings are
    /// included when [FieldOptions::submit_empty] is set.
    pub fn get_submit_data(&self) -> Value {
        self.read().get_submit_data()
    }
}

/// A wrapper type for a mutably borrowed [FormContext]
pub struct FormContextWriteGuard<'a> {
    form_ctx: FormContext,
    state: ManuallyDrop<RefMut<'a, FormContextState>>,
    initial_version: usize,
}

impl Deref for FormContextWriteGuard<'_> {
    type Target = FormContextState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<'a> DerefMut for FormContextWriteGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl<'a> Drop for FormContextWriteGuard<'a> {
    fn drop(&mut self) {
        let changed = self.state.version != self.initial_version;
        unsafe { ManuallyDrop::drop(&mut self.state); } // drop ref before calling notify listeners
        if changed { self.form_ctx.notify_listeners(); }
    }
}

/// Wraps a borrowed reference to a [FormContext]
pub struct FormContextReadGuard<'a> {
    state: Ref<'a, FormContextState>,
}

impl Deref for FormContextReadGuard<'_> {
    type Target = FormContextState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

#[derive(Clone)]
struct GroupState {
    value: Option<Value>,
    members: Vec<usize>,
    radio_count: usize,
}

/// The inner state of a [FormContext].
///
/// A Form contains named fields. Field names are not required to be
/// unique. Fields using the same name are called a "field group".
pub struct FormContextState {
    version: usize,
    listeners: Slab<Callback<FormContext>>,
    fields: Slab<FieldRegistration>,
    groups: HashMap<AttrValue, GroupState>,
    show_advanced: bool,
}

impl FormContextState {

    fn new() -> Self {
        Self {
            version: 0,
            listeners: Slab::new(),
            fields: Slab::new(),
            groups: HashMap::new(),
            show_advanced: false,
         }
    }

    fn add_listener(&mut self, cb: Callback<FormContext>) -> usize {
        self.listeners.insert(cb)
    }

    fn remove_listener(&mut self, key: usize) {
        self.listeners.remove(key);
    }

    fn register_field(
        &mut self,
        name: impl IntoPropValue<AttrValue>,
        value: Value,
        default: Value,
        radio_group: bool,
        validate: Option<ValidateFn<Value>>,
        options: FieldOptions,
    ) -> usize {
        let name = name.into_prop_value();

        let mut valid = Ok(());
        if let Some(validate) = &validate {
            valid = validate.validate(&value)
                .map_err(|e| e.to_string());
        }

        let field = FieldRegistration {
            name: name.clone(),
            validate,
            radio_group,
            options,
            value,
            default: default.clone(),
            valid,
        };

        self.version += 1;
        let slab_key = self.fields.insert(field);

        let group = self.groups.entry(name).or_insert(GroupState {
            value: None,
            members: Vec::new(),
            radio_count: 0,
        });

        group.members.push(slab_key);
        if radio_group {
            if let Some(default) = default.as_str() {
                if !default.is_empty() && group.value.is_none() {
                    group.value = Some(default.clone().into());
                }
            }
            group.radio_count += 1;
        }

        slab_key
    }

    fn unregister_field(&mut self, key: usize) {
        self.version += 1;
        let field = self.fields.remove(key);
        let group = self.groups.get_mut(&field.name).unwrap();
        group.members.retain(|k| k != &key);
        if field.radio_group {
            group.radio_count = group.radio_count.saturating_sub(1);
        }
    }

    pub fn set_show_advanced(&mut self, show_advanced: bool) {
        if self.show_advanced != show_advanced {
            self.show_advanced = show_advanced;
            self.version += 1;
        }
    }

    fn find_field_slab_id(&self, name: &AttrValue) -> Option<usize> {
        self.fields.iter().find(|(_key, f)| &f.name == name).map(|(key, _)| key)
    }

    fn get_field_data_by_slab_key(
        &self,
        slab_key: usize,
    ) ->  (Value, Result<(), String>) {
        let field = &self.fields[slab_key];

        if field.radio_group {
            let group = &self.groups[&field.name];
            let value = group.value.clone().unwrap_or("".into());
            let valid = Ok(()); // fixme
            (value, valid)
        } else {
            (field.value.clone(), field.valid.clone())
        }
    }

    pub fn get_field_data(
        &self,
        name: impl IntoPropValue<AttrValue>,
    ) -> Option<(Value, Result<(), String>)> {
        let name = name.into_prop_value();
        match self.find_field_slab_id(&name) {
            Some(key) => Some(self.get_field_data_by_slab_key(key)),
            None => None,
        }
    }

    /// Get the field value.
    ///
    /// Returns `None` for non-existent fields.
    pub fn get_field_value(&self, name: impl IntoPropValue<AttrValue>) -> Option<Value> {
        self.get_field_data(name).map(|data| data.0)
    }

    /// Get the field value as string.
    ///
    /// Return the empty string for non-existent fields, or
    /// when the field value is not a string or number.
    pub fn get_field_text(&self, name: impl IntoPropValue<AttrValue>) -> String {
        match self.get_field_data(name).map(|data| data.0)  {
            Some(Value::Number(n)) => n.to_string(),
            Some(Value::String(v)) => v.clone(),
            _ => String::new(),
        }
    }

    pub fn get_field_valid(&self, name: impl IntoPropValue<AttrValue>) -> Option<Result<(), String>> {
        self.get_field_data(name).map(|data| data.1)
    }

    fn set_field_value_by_slab_key(
        &mut self,
        slab_key: usize,
        value: Value,
        set_default: bool,
    ) {
        let field = &mut self.fields[slab_key];

        if field.radio_group {
            let group = self.groups.get_mut(&field.name).unwrap();
            let changed = match &group.value {
                Some(current_value) => current_value != &value,
                None => true,
            };
            if changed {
                group.value = Some(value.clone());
                self.version += 1;
            }
            if set_default {
                for member in group.members.iter() {
                    let group_field = &mut self.fields[*member];
                    if group_field.value == value {
                        group_field.default = value.clone();
                    } else {
                        group_field.default = String::new().into();
                    }
                }
            }
        } else {
            if set_default && field.default != value {
                field.default = value.clone();
                self.version += 1;
            }
            if value != field.value {

                let mut valid = Ok(());
                if let Some(validate) = &field.validate {
                    valid = validate.validate(&value)
                        .map_err(|e| e.to_string());
                }

                field.value = value;
                field.valid = valid;

                self.version += 1;
            }
        }
    }

    pub fn set_field_value(
        &mut self,
        name: impl IntoPropValue<AttrValue>,
        value: Value,
    ) {
        let name = name.into_prop_value();
        if let Some(slab_key) = self.find_field_slab_id(&name) {
            self.set_field_value_by_slab_key(slab_key, value, false);
        }
    }

    fn update_field_options_by_slab_key(
        &mut self,
        slab_key: usize,
        options: FieldOptions,
    ) {
        let field = &mut self.fields[slab_key];
        field.options = options;
    }

    pub fn is_dirty(&self) -> bool {
        for (_key, field) in self.fields.iter() {
            if field.value != field.default {
                return true;
            }
        }
        false
    }

    /// Reset all form fields to their default value.
    pub fn reset_form(&mut self) {
        let mut changes = false;
        for (_key, field) in self.fields.iter_mut() {
            if field.value != field.default {
                changes = true;
                field.value = field.default.clone();
                let mut valid = Ok(());
                if let Some(validate) = &field.validate {
                    valid = validate.validate(&field.value)
                        .map_err(|e| e.to_string());
                }
                field.valid = valid;
            }
        }
        if changes {
            self.version += 1;
        }
    }

    pub fn is_valid(&self) -> bool {
        for (_key, field) in self.fields.iter() {
            if field.valid.is_err() {
                return false;
            }
        }
        true
    }

    fn set_field_default_by_slab_key(&mut self, slab_key: usize, default: Value) {
        let field = &mut self.fields[slab_key];
        field.default = default;
    }

    fn validate_field_by_slab_key(&mut self, slab_key: usize) {
        let field = &mut self.fields[slab_key];

        if field.radio_group {
            // fixme: do something ?
        } else {
            let mut valid = Ok(());
            if let Some(validate) = &field.validate {
                valid = validate.validate(&field.value)
                    .map_err(|e| e.to_string());
            }
            if valid != field.valid {
                self.version += 1;
                field.valid = valid;

            }
        }
    }

    pub fn validate_field(&mut self, name: impl IntoPropValue<AttrValue>) {
        let name = name.into_prop_value();
        if let Some(slab_key) = self.find_field_slab_id(&name) {
            self.validate_field_by_slab_key(slab_key);
        }
    }

    fn set_field_valid_by_slab_key(&mut self, slab_key: usize, valid: Result<(), String>) {
        let field = &mut self.fields[slab_key];
        if valid != field.valid {
            self.version += 1;
            field.valid = valid;
        }
    }

    pub fn set_field_valid(&mut self, name: impl IntoPropValue<AttrValue>, valid: Result<(), String>) {
        let name = name.into_prop_value();
        if let Some(slab_key) = self.find_field_slab_id(&name) {
            self.set_field_valid_by_slab_key(slab_key, valid);
        }
    }


    /// Load form data.
    pub fn load_form(&mut self, data: Value) {
        self.version += 1;

        // Note: We clone self.groups here, so that we can still modify fields
        for (name, group) in self.groups.clone().iter() {
            if group.members.is_empty() { continue; }

            let value = match data.get(name.deref()) {
                None => continue,
                Some(value) => value.clone(),
            };

            // Are there radio group fields?
            let radio_group_key = group.members.iter()
                .find(|k| self.fields[**k].radio_group == true);

            if let Some(radio_group_key) = radio_group_key {
                // Note: we only call set_value for one radio_group member
                self.set_field_value_by_slab_key(*radio_group_key, value, true);
                continue;
            }

            if group.members.len() == 1 {
                let key = group.members[0];
                self.set_field_value_by_slab_key(key, value, true);
                continue;
            }

            // there are several group members, restore data as array
            let list = match value.as_array() {
                Some(list) => list.clone(),
                None => vec![value],
            };

            for (i, key) in group.members.iter().enumerate() {
                let value = match list.get(i) {
                    Some(v) => v.clone(),
                    None => break,
                };
                self.set_field_value_by_slab_key(*key, value, true);
            }
        }

    }

    /// Get form submit data.
    pub fn get_submit_data(&self) -> Value {
        let mut data = json!({});

        for (name, group) in self.groups.iter() {
            if group.members.is_empty() { continue; }

            let field_keys: Vec<usize> = group.members.iter()
                .filter(|k| !self.fields[**k].radio_group)
                .map(|k| *k)
                .collect();

            let radio_keys: Vec<usize> = group.members.iter()
                .filter(|k| self.fields[**k].radio_group)
                .map(|k| *k)
                .collect();

            if !radio_keys.is_empty() {
                let mut submit = false;
                let mut submit_empty = false;
                for key in radio_keys {
                    if self.fields[key].options.submit { submit = true; }
                    if self.fields[key].options.submit_empty { submit_empty = true; }
                }
                if submit {
                    let value = match &group.value {
                        Some(value) => value.as_str().unwrap_or("").to_string(),
                        None => String::new(),
                    };
                    if !value.is_empty() || submit_empty {
                        data[name.deref()] = value.into();
                    }
                }
            }

            if field_keys.is_empty() { continue; }

            if field_keys.len() == 1 {
                let key = field_keys[0];
                let field = &self.fields[key];
                if field.options.submit {
                    let value = field.value.clone();
                    match value {
                        Value::String(v) => {
                            if !v.is_empty() || field.options.submit_empty {
                                data[name.deref()] = Value::String(v);
                            }
                        }
                        // Bool/Number/Array/Object/Null
                        v => data[name.deref()] = v,
                    }
                }
                continue;
            }

            if field_keys.len() > 1 { // include as array
                let mut list = Vec::new();
                for key in field_keys {
                    let field = &self.fields[key];
                    if field.options.submit {
                        let value = field.value.clone();
                        match value {
                            Value::String(v) => {
                                if !v.is_empty() || field.options.submit_empty {
                                    list.push(Value::String(v));
                                }
                            }
                            // Bool/Number/Array/Object/Null
                            v => list.push(v),
                        }
                    }
                }
                if !list.is_empty() {
                    data[name.deref()] = list.into();
                }
            }
        }

        data
    }
}
