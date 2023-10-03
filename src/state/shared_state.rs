use std::rc::Rc;
use std::cell::{Ref, RefCell, RefMut};
use std::ops::{Deref, DerefMut};
use std::mem::ManuallyDrop;

use derivative::Derivative;
use slab::Slab;

use yew::prelude::*;
use yew::html::IntoEventCallback;

use crate::state::optional_rc_ptr_eq;

/// Encapsulates the shared state inner data.
pub struct SharedStateInner<T> {
    data: T,
    listeners: Slab<Callback<SharedState<T>>>,
}

impl<T> SharedStateInner<T> {
    fn add_listener(&mut self, cb: Callback<SharedState<T>>) -> usize {
        self.listeners.insert(cb)
    }

    fn remove_listener(&mut self, key: usize) {
        self.listeners.remove(key);
    }
}

impl<T> Deref for SharedStateInner<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for SharedStateInner<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct SharedState<T> {
    // Allow to store one Observer here (for convenience)
    #[derivative(PartialEq(compare_with="optional_rc_ptr_eq"))]
    on_change: Option<Rc<SharedStateObserver<T>>>,
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    inner: Rc<RefCell<SharedStateInner<T>>>,
}

/// Owns the SharedState listener callback. When dropped, the
/// listener callback will be removed fron the SharedState.
pub struct SharedStateObserver<T> {
    key: usize,
    inner: Rc<RefCell<SharedStateInner<T>>>,
}

impl<T> Drop for SharedStateObserver<T> {
    fn drop(&mut self) {
        self.inner.borrow_mut().remove_listener(self.key);
    }
}

impl<T> SharedState<T> {
    /// Create a new instance.
    pub fn new(data: T) -> Self {
        Self {
            on_change: None,
            inner: Rc::new(RefCell::new(SharedStateInner {
                data,
                listeners: Slab::new(),
            }))
        }
    }

    /// Builder style method to set the on_change callback.
    ///
    /// This calls [Self::add_listener] to create a new
    /// [SharedStateObserver]. The observer is stored inside the
    /// [SharedState] object, so each clone can hold a single on_change
    /// callback.
    pub fn on_change(mut self, cb: impl IntoEventCallback<SharedState<T>>) -> Self {
        self.set_on_change(cb);
        self
    }

    pub fn set_on_change(&mut self, cb: impl IntoEventCallback<SharedState<T>>) {
        self.on_change = match cb.into_event_callback() {
            Some(cb) => Some(Rc::new(self.add_listener(cb))),
            None => None,
        };
    }

    /// Method to add an shared state observer.
    ///
    /// This is usually called by [Self::on_change], which stores the
    /// observer inside the [SharedState] object.
    pub fn add_listener(&self, cb: impl Into<Callback<SharedState<T>>>) -> SharedStateObserver<T> {
        let key = self.inner.borrow_mut()
            .add_listener(cb.into());
        SharedStateObserver { key, inner: self.inner.clone() }
    }

    fn notify_listeners(&self) {
        let listeners = self.inner.borrow().listeners.clone(); // clone to avoid borrow()
        for (_key, listener) in listeners.iter() {
            listener.emit(self.clone());
        }
    }

    /// Lock this shared state for write access.
    ///
    /// Please use a write lock if you do bulk operations. This
    /// notifies the listeners when you drop the lock only once.
    ///
    /// # Panics
    ///
    /// Panics if the store is already locked.
    pub fn write(&self) -> SharedStateWriteGuard<T> {
        let cloned_self = Self { on_change: None, inner: self.inner.clone() };
        let borrowed_state = ManuallyDrop::new(self.inner.borrow_mut());
        SharedStateWriteGuard {
            shared_state: cloned_self,
            //initial_version: state.version,
            borrowed_state,
            notify: true,
        }
    }

    /// Lock this shared state for read access.
    ///
    /// # Panics
    ///
    /// Panics if the shared state is currently mutably locked.
    pub fn read(&self) -> SharedStateReadGuard<T> {
        SharedStateReadGuard {
            borrowed_state: self.inner.borrow(),
        }
    }
}

/// A wrapper type for a mutably borrowed [SharedState]
pub struct SharedStateWriteGuard<'a, T> {
    shared_state: SharedState<T>,
    borrowed_state: ManuallyDrop<RefMut<'a, SharedStateInner<T>>>,
    pub notify: bool, // send notifications
    //initial_version: usize,
}

impl<'a, T> Deref for SharedStateWriteGuard<'a, T> {
    type Target = SharedStateInner<T>;

    fn deref(&self) -> &Self::Target {
        &self.borrowed_state
    }
}

impl<'a, T> DerefMut for SharedStateWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.borrowed_state
    }
}

impl<'a, T> Drop for SharedStateWriteGuard<'a, T> {
    fn drop(&mut self) {
        //let changed = self.state.version != self.initial_version;
        let changed = true; // TODO: impl change detection?
        unsafe { ManuallyDrop::drop(&mut self.borrowed_state); } // drop ref before calling notify listeners
        if self.notify && changed { self.shared_state.notify_listeners(); }
    }
}

// Wraps a borrowed reference to a [SharedState]
pub struct SharedStateReadGuard<'a, T> {
    borrowed_state: Ref<'a, SharedStateInner<T>>,
}

impl<'a, T> Deref for SharedStateReadGuard<'a, T> {
    type Target = SharedStateInner<T>;

    fn deref(&self) -> &Self::Target {
        &self.borrowed_state
    }
}
