use std::rc::Rc;
use std::cell::{Ref, RefMut, RefCell};
use std::ops::Range;
use std::ops::{Deref, DerefMut};
use std::collections::HashMap;

use derivative::Derivative;
use slab::Slab;
use serde_json::Value;

use yew::prelude::*;
use yew::html::{IntoEventCallback, IntoPropValue};

use crate::state::optional_rc_ptr_eq;
use crate::widget::form::ValidateFn; // fixme: move to props

use super::{FieldState, FieldRegistration};

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
    inner: Rc<RefCell<FormState>>,
}

impl FieldHandle {

    // Lock the form context for read access.
    fn read(&self) -> FormContextReadGuard {
        FormContextReadGuard {
            state: self.inner.borrow(),
        }
    }

    // Lock the form context for write access.
    //
    // Automatically notifies listeners when the guard is dropped.
    fn write(&self) -> FormContextWriteGuard {
        let state = self.inner.borrow_mut();
        FormContextWriteGuard {
            initial_version: state.version,
            state,
        }
    }

    pub fn get_value(&mut self) -> Option<Value> {
        let key = self.key;
        let state = self.inner.borrow();
        let name = &state.fields.get(key).unwrap().name;
        state.get_value(name).map(Value::clone)
    }

    /// Get the field value as string.
    ///
    /// Return the empty string when the field value is not a string
    /// or number.
    pub fn get_text(&self) -> Option<String> {
        let key = self.key;
        let state = self.inner.borrow();
        let name = &state.fields.get(key).unwrap().name;
        state.get_value(name).map(|value| match value {
            Value::Number(n) => n.to_string(),
            Value::String(v) => v.clone(),
            _ => String::new(),
        })
    }

    pub fn set_value(&mut self, value: Value) {
        let key = self.key;
        let name = self.inner.borrow()
            .fields.get(key).unwrap()
            .name.clone();
        self.write().set_value(name, value);
    }
}

impl Drop for FieldHandle {
    fn drop(&mut self) {
        self.inner.borrow_mut().unregister_field(self.key);
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
    pub fn on_change(mut self, cb: impl IntoEventCallback<()>) -> Self {
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
    pub fn add_listener(&self, cb: impl Into<Callback<()>>) -> FormObserver {
        let key = self.inner.borrow_mut()
            .add_listener(cb.into());
        FormObserver { key, inner: self.inner.clone() }
    }

    // Lock the form context for read access.
    fn read(&self) -> FormContextReadGuard {
        FormContextReadGuard {
            state: self.inner.borrow(),
        }
    }

    // Lock the form context for write access.
    //
    // Automatically notifies listeners when the guard is dropped.
    fn write(&self) -> FormContextWriteGuard {
        let state = self.inner.borrow_mut();
        FormContextWriteGuard {
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
        validate: Option<ValidateFn<Value>>,
        submit: bool,
        submit_empty: bool,

    ) -> FieldHandle {
        let name = name.into_prop_value();
        let registration = FieldRegistration {
            name,
            validate,
            submit,
            submit_empty,
        };

        let key = self.inner.borrow_mut()
            .register_field(registration);

        FieldHandle { key, inner: self.inner.clone() }
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

// A wrapper type for a mutably borrowed [FormContext]
struct FormContextWriteGuard<'a> {
    state: RefMut<'a, FormState>,
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
        if self.state.version != self.initial_version {
            self.state.notify_listeners();
        }
    }
}

// Wraps a borrowed reference to a [FormContext]
struct FormContextReadGuard<'a> {
    state: Ref<'a, FormState>,
}

impl Deref for FormContextReadGuard<'_> {
    type Target = FormState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

struct FormState {
    version: usize,
    listeners: Slab<Callback<()>>,
    fields: Slab<FieldRegistration>,
    show_advanced: bool,
    data: HashMap<AttrValue, Value>
}

impl FormState {

    fn new() -> Self {
        Self {
            version: 0,
            listeners: Slab::new(),
            fields: Slab::new(),
            show_advanced: false,
            data: HashMap::new(),
        }
    }

    fn add_listener(&mut self, cb: Callback<()>) -> usize {
        self.listeners.insert(cb)
    }

    fn remove_listener(&mut self, key: usize) {
        self.listeners.remove(key);
    }

    fn notify_listeners(&self) {
        for (_key, listener) in self.listeners.iter() {
            listener.emit(());
        }
    }

    fn register_field(&mut self, field: FieldRegistration) -> usize {
        self.fields.insert(field)
    }

    fn unregister_field(&mut self, key: usize) {
        self.fields.remove(key);
    }

    fn set_show_advanced(&mut self, show_advanced: bool) {
        if self.show_advanced != show_advanced {
            self.show_advanced = show_advanced;
            self.version += 1;
        }
    }

    fn get_value(&self, name: &AttrValue) -> Option<&Value> {
        self.data.get(name)
    }

    fn set_value(&mut self, name: AttrValue, value: Value) {
        let current_value = self.data.get(&name);
        if current_value != Some(&value) {
            self.data.insert(name, value);
            self.version += 1;
        }
    }
}
