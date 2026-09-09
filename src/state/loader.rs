use std::future::Future;
use std::rc::Rc;

use anyhow::Error;
use derivative::Derivative;

use serde::{Serialize, de::DeserializeOwned};
use yew::html::IntoEventCallback;
use yew::prelude::*;

use crate::AsyncAbortGuard;
use crate::prelude::*;
use crate::props::{IntoLoadCallback, IntoStorageLocation, LoadCallback, StorageLocation};
use crate::state::{
    SharedState, SharedStateObserver, SharedStateReadGuard, SharedStateWriteGuard,
    optional_rc_ptr_eq,
};
use crate::widget::{Button, Container, Fa, error_message};

/// Shared HTTP load state
///
/// Stores the active request and the load result.
pub struct LoaderState<T> {
    storage_location: Option<StorageLocation>,
    active_load: Option<ActiveLoad>,
    pub loader: Option<LoadCallback<T>>,
    pub data: Option<Result<Rc<T>, Error>>,
}

struct ActiveLoad {
    // The future holds a clone, preventing address reuse while it can still compare the request ID.
    id: Rc<()>,
    _abort_guard: AsyncAbortGuard,
}

struct LoaderInner<T> {
    state: SharedState<LoaderState<T>>,
}

impl<T> Drop for LoaderInner<T> {
    fn drop(&mut self) {
        // Observer handles can outlive the Loader and keep its shared state allocated, but must not
        // keep a request running after the last Loader clone is dropped.
        let mut state = self.state.write();
        state.notify = false;
        state.active_load = None;
    }
}

impl<T: 'static + DeserializeOwned + Serialize> LoaderState<T> {
    fn load_from_cache(&mut self) {
        let storage_location = match &self.storage_location {
            Some(storage_location) => storage_location,
            None => return,
        };

        if let Some(data) = super::load_state(storage_location) {
            self.data = Some(Ok(Rc::new(data)));
        }
    }

    fn store_to_cache(&mut self) {
        let storage_location = match &self.storage_location {
            Some(storage_location) => storage_location,
            None => return,
        };

        match &self.data {
            Some(Ok(data)) => {
                super::store_state(data, storage_location);
            }
            _ => {
                super::delete_state(storage_location);
            }
        }
    }
}

/// Helper to share async loaded data.
///
/// - clnonable, shared state with change notifications.
/// - stores load result as `Option<Result<Rc<T>, Error>>`.
/// - tracks load state `self.loading()`.
/// - ability to cache result in local (default) or session storage by setting `state_id`.
/// - helper to simplify renderering `self.render`.
///
/// Starting a load replaces the previous request. Dropping the last Loader clone cancels the active
/// request; pending futures and listener registrations do not themselves keep the Loader alive.
/// The canceled future is dropped when the executor observes the abort. Aborting an underlying HTTP
/// request depends on the load callback's cancellation behavior.
#[derive(Derivative)]
#[derivative(Clone(bound = ""), PartialEq(bound = ""))]
pub struct Loader<T> {
    #[derivative(PartialEq(compare_with = "optional_rc_ptr_eq"))]
    on_change: Option<Rc<SharedStateObserver<LoaderState<T>>>>,
    #[derivative(PartialEq(compare_with = "Rc::ptr_eq"))]
    inner: Rc<LoaderInner<T>>,
}

impl<T: 'static + DeserializeOwned + Serialize> Default for Loader<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: 'static + DeserializeOwned + Serialize> Loader<T> {
    /// Create a new instance.
    pub fn new() -> Self {
        let state = LoaderState {
            data: None,
            loader: None,
            storage_location: None,
            active_load: None,
        };
        Self {
            on_change: None,
            inner: Rc::new(LoaderInner {
                state: SharedState::new(state),
            }),
        }
    }

    /// Builder style method to set the persistent state ID.
    pub fn state_id(mut self, state_id: impl IntoStorageLocation) -> Self {
        self.set_state_id(state_id);
        self
    }

    /// Method to set the persistent state ID.
    pub fn set_state_id(&mut self, state_id: impl IntoStorageLocation) {
        let mut me = self.write();
        me.storage_location = state_id.into_storage_location();
        me.load_from_cache();
    }

    /// Register a listener kept alive by this handle and its clones.
    ///
    /// Starting a load does not notify listeners. Accepted completion, explicit abort, and writes
    /// through [Self::write] do. The Loader passed to the callback does not retain registrations.
    pub fn on_change(mut self, cb: impl IntoEventCallback<Loader<T>>) -> Self {
        self.on_change = cb
            .into_event_callback()
            .map(|cb| Rc::new(self.add_listener(cb)));
        self
    }

    /// Builder style method to set the load callback.
    pub fn loader(mut self, callback: impl IntoLoadCallback<T>) -> Self {
        self.set_loader(callback);
        self
    }

    /// Method to set the load callback.
    pub fn set_loader(&mut self, callback: impl IntoLoadCallback<T>) {
        let mut me = self.write();
        me.notify = false;
        me.loader = callback.into_load_callback();
    }

    /// Register a listener until the returned observer is dropped, without retaining the Loader.
    pub fn add_listener(
        &self,
        cb: impl Into<Callback<Loader<T>>>,
    ) -> SharedStateObserver<LoaderState<T>> {
        let inner = Rc::downgrade(&self.inner);
        let cb = cb.into();
        self.inner.state.add_listener(move |_| {
            if let Some(inner) = inner.upgrade() {
                cb.emit(Self {
                    on_change: None,
                    inner,
                });
            }
        })
    }

    pub fn read(&self) -> SharedStateReadGuard<'_, LoaderState<T>> {
        self.inner.state.read()
    }
    pub fn write(&self) -> SharedStateWriteGuard<'_, LoaderState<T>> {
        self.inner.state.write()
    }

    /// Whether a current request is pending, excluding canceled and superseded requests.
    pub fn loading(&self) -> bool {
        self.read().active_load.is_some()
    }

    pub fn has_valid_data(&self) -> bool {
        matches!(self.read().data, Some(Ok(_)))
    }

    pub fn render<R: Into<Html>>(&self, render: impl Fn(Rc<T>) -> R) -> Html {
        match &self.read().data {
            None => Container::new()
                .class("pwt-text-center")
                .padding(4)
                .with_child(Fa::new("spinner").margin_end(1).pulse())
                .with_child(tr!("Loading..."))
                .into(),
            Some(Ok(data)) => render(Rc::clone(data)).into(),
            Some(Err(err)) => error_message(&format!("Error: {}", err)).padding(2).into(),
        }
    }

    /// Start a request, replacing any pending request without notifying listeners on entry.
    pub fn load(&self) {
        if let Some(future) = self.start_load() {
            wasm_bindgen_futures::spawn_local(future);
        }
    }

    fn start_load(&self) -> Option<impl Future<Output = ()> + use<T>> {
        let loader = self.read().loader.clone()?;
        let inner = Rc::downgrade(&self.inner);
        let id = Rc::new(());
        let request_id = Rc::clone(&id);
        let (abort_guard, future) = AsyncAbortGuard::new(async move {
            let res = loader.apply().await;
            let Some(inner) = inner.upgrade() else {
                return;
            };
            let mut state = inner.state.write();
            // A callback can replace or abort its own request during its final poll. Aborting the
            // future alone does not prevent that poll from completing.
            if !state
                .active_load
                .as_ref()
                .is_some_and(|active| Rc::ptr_eq(&active.id, &request_id))
            {
                state.notify = false;
                return;
            }
            state.active_load = None;
            state.data = Some(res.map(Rc::new));
            state.store_to_cache();
        });

        let mut state = self.write();
        state.notify = false;
        state.active_load = Some(ActiveLoad {
            id,
            _abort_guard: abort_guard,
        });
        Some(future)
    }

    /// Abort the current request and notify listeners, without changing data or persistent cache.
    pub fn abort(&mut self) {
        self.write().active_load = None;
    }

    pub fn reload_button(&self) -> Button {
        let loader = self.clone();
        Button::refresh(self.loading()).onclick(move |_| loader.load())
    }
}

#[cfg(test)]
mod tests;
