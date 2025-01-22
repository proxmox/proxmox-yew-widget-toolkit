use std::rc::Rc;

use anyhow::Error;
use derivative::Derivative;

use serde::{de::DeserializeOwned, Serialize};
use yew::html::IntoEventCallback;
use yew::prelude::*;

use crate::prelude::*;
use crate::props::{IntoLoadCallback, IntoStorageLocation, LoadCallback, StorageLocation};
use crate::state::{SharedState, SharedStateObserver, SharedStateReadGuard, SharedStateWriteGuard};
use crate::widget::{error_message, Button, Container, Fa};
use crate::AsyncAbortGuard;

/// Shared HTTP load state
///
/// This struct stores the state (loading) and the result of the load.
pub struct LoaderState<T> {
    loading: u64,
    storage_location: Option<StorageLocation>,
    async_abort_guard: Option<AsyncAbortGuard>,
    pub loader: Option<LoadCallback<T>>,
    pub data: Option<Result<Rc<T>, Error>>,
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
#[derive(Derivative)]
#[derivative(Clone(bound = ""), PartialEq(bound = ""))]
pub struct Loader<T>(SharedState<LoaderState<T>>);

impl<T: 'static + DeserializeOwned + Serialize> Default for Loader<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: 'static + DeserializeOwned + Serialize> Loader<T> {
    /// Create a new instance.
    pub fn new() -> Self {
        let state = LoaderState {
            loading: 0,
            data: None,
            loader: None,
            storage_location: None,
            async_abort_guard: None,
        };
        Self(SharedState::new(state))
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

    pub fn on_change(mut self, cb: impl IntoEventCallback<Loader<T>>) -> Self {
        let me = self.clone();
        match cb.into_event_callback() {
            Some(cb) => self.0.set_on_change(move |_| cb.emit(me.clone())),
            _ => self
                .0
                .set_on_change(None::<Callback<SharedState<LoaderState<T>>>>),
        };
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

    pub fn add_listener(
        &self,
        cb: impl Into<Callback<Loader<T>>>,
    ) -> SharedStateObserver<LoaderState<T>> {
        let me = self.clone();
        let cb = cb.into();
        self.0.add_listener(move |_| cb.emit(me.clone()))
    }

    pub fn read(&self) -> SharedStateReadGuard<LoaderState<T>> {
        self.0.read()
    }
    pub fn write(&self) -> SharedStateWriteGuard<LoaderState<T>> {
        self.0.write()
    }

    pub fn loading(&self) -> bool {
        self.read().loading > 0
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
            Some(Ok(ref data)) => render(Rc::clone(data)).into(),
            Some(Err(err)) => error_message(&format!("Error: {}", err)).padding(2).into(),
        }
    }

    pub fn load(&self) {
        let loader = match &self.read().loader {
            Some(loader) => loader.clone(),
            None => return, // do nothing
        };

        let me = self.clone();

        let mut state = self.write();
        state.loading += 1;
        state.async_abort_guard = Some(AsyncAbortGuard::spawn(async move {
            let res = loader.apply().await;
            let mut me = me.write();
            me.async_abort_guard = None;
            me.loading -= 1;
            me.data = Some(res.map(|data| Rc::new(data)));
            me.store_to_cache();
        }));

        // we don't want to notify listeners here, because we just set up the loading
        state.notify = false;
        drop(state);
    }

    pub fn reload_button(&self) -> Button {
        let loader = self.clone();
        Button::refresh(self.loading()).onclick(move |_| loader.load())
    }
}
