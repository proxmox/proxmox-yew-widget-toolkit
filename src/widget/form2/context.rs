use std::rc::Rc;
use std::cell::{Ref, RefMut, RefCell};
use std::ops::{Deref, DerefMut};
use std::mem::ManuallyDrop;

use derivative::Derivative;
use slab::Slab;
use serde_json::Value;

use yew::prelude::*;
use yew::html::{IntoEventCallback, IntoPropValue};

use crate::state::optional_rc_ptr_eq;
use crate::widget::form::ValidateFn; // fixme: move to props

#[derive(Clone, Debug, PartialEq)]
struct FieldRegistration {
    pub name: AttrValue,
    pub validate: Option<ValidateFn<Value>>,
    pub submit: bool,
    pub submit_empty: bool,

    pub value: Value,
    pub default: Value,
    pub valid: Result<(), String>,
}

/// Shared form data.
#[derive(Derivative)]
#[derivative(Clone, PartialEq)]
pub struct FormContext {
    // Allow to store one StoreObserver here (for convenience)
    #[derivative(PartialEq(compare_with="optional_rc_ptr_eq"))]
    on_change: Option<Rc<FormObserver>>,
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    inner: Rc<RefCell<FormState>>,
}

/// Owns the listener callback. When dropped, the
/// listener callback will be removed from the [FormContext].
pub struct FormObserver {
    key: usize,
    inner: Rc<RefCell<FormState>>,
}

impl Drop for FormObserver {
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

    pub fn set_value(&mut self, value: Value) {
        let key = self.key;
        self.write().set_field_value_by_slab_key(key, value);
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
            inner: Rc::new(RefCell::new(FormState::new())),
        }
    }

    /// Builder style method to set the on_change callback.
    ///
    /// This calls [Self::add_listener] to create a new
    /// [FormObserver]. The observer is stored inside the
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
    pub fn add_listener(&self, cb: impl Into<Callback<FormContext>>) -> FormObserver {
        let key = self.inner.borrow_mut()
            .add_listener(cb.into());
        FormObserver { key, inner: self.inner.clone() }
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
        validate: Option<ValidateFn<Value>>,
        submit: bool,
        submit_empty: bool,
    ) -> FieldHandle {
        let key = self.inner.borrow_mut()
            .register_field(name, value, default, validate, submit, submit_empty);

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
}

/// A wrapper type for a mutably borrowed [FormContext]
pub struct FormContextWriteGuard<'a> {
    form_ctx: FormContext,
    state: ManuallyDrop<RefMut<'a, FormState>>,
    initial_version: usize,
}

impl Deref for FormContextWriteGuard<'_> {
    type Target = FormState;

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
    state: Ref<'a, FormState>,
}

impl Deref for FormContextReadGuard<'_> {
    type Target = FormState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

/// Form state.
pub struct FormState {
    version: usize,
    listeners: Slab<Callback<FormContext>>,
    fields: Slab<FieldRegistration>,
    show_advanced: bool,
}

impl FormState {

    fn new() -> Self {
        Self {
            version: 0,
            listeners: Slab::new(),
            fields: Slab::new(),
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
        validate: Option<ValidateFn<Value>>,
        submit: bool,
        submit_empty: bool,
    ) -> usize {
        let name = name.into_prop_value();

        let mut valid = Ok(());
        if let Some(validate) = &validate {
            valid = validate.validate(&value)
                .map_err(|e| e.to_string());
        }

        let field = FieldRegistration {
            name,
            validate,
            submit,
            submit_empty,
            value,
            default,
            valid,
        };

        self.version += 1;
        self.fields.insert(field)
    }

    fn unregister_field(&mut self, key: usize) {
        self.version += 1;
        self.fields.remove(key);
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
        (field.value.clone(), field.valid.clone())
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

    pub fn get_field_value(&self, name: impl IntoPropValue<AttrValue>) -> Option<Value> {
        self.get_field_data(name).map(|data| data.0)
    }

    pub fn get_field_valid(&self, name: impl IntoPropValue<AttrValue>) -> Option<Result<(), String>> {
        self.get_field_data(name).map(|data| data.1)
    }

    fn set_field_value_by_slab_key(
        &mut self,
        slab_key: usize,
        value: Value,
    ) {
        let field = &mut self.fields[slab_key];
        let current_value = &field.value;
        if current_value != &value {

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

    pub fn set_field_value(
        &mut self,
        name: impl IntoPropValue<AttrValue>,
        value: Value,
    ) {
        let name = name.into_prop_value();
        if let Some(slab_key) = self.find_field_slab_id(&name) {
            self.set_field_value_by_slab_key(slab_key, value);
        }
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

}
