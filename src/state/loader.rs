use std::rc::Rc;

use anyhow::Error;
use derivative::Derivative;

use serde::{de::DeserializeOwned, Serialize};
use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;

use crate::prelude::*;
use crate::props::{IntoLoadCallback, LoadCallback};
use crate::state::{local_storage, SharedState, SharedStateObserver, SharedStateReadGuard, SharedStateWriteGuard};
use crate::widget::{error_message, Button, Fa};

/// Shared HTTP load state
///
/// This struct stores the state (loading) and the result of the load.
pub struct LoaderState<T> {
    loading: u64,
    state_id: Option<AttrValue>,
    pub loader: Option<LoadCallback<T>>,
    pub data: Option<Result<Rc<T>, Error>>,
}

impl<T: 'static + DeserializeOwned + Serialize> LoaderState<T> {
    fn load_from_cache(&mut self) {
        let state_id = match &self.state_id {
            Some(state_id) => state_id,
            None => return,
        };
        let store = match local_storage() {
            Some(store) => store,
            None => return,
        };
        if let Ok(Some(item_str)) = store.get_item(state_id) {
            if let Ok(data) = serde_json::from_str(&item_str) {
                self.data = Some(Ok(Rc::new(data)));
            }
        }
    }

    fn store_to_cache(&mut self) {
        let state_id = match &self.state_id {
            Some(state_id) => state_id,
            None => return,
        };
        let store = match local_storage() {
            Some(store) => store,
            None => return,
        };
        match &self.data {
            Some(Ok(data)) => {
                let item_str = serde_json::to_string(data).unwrap();
                match store.set_item(state_id, &item_str) {
                     Err(err) => log::error!(
                        "store loader state {} failed: {}",
                        state_id,
                        crate::convert_js_error(err)
                    ),
                    Ok(_) => {}
                }
            }
            _ =>{
                let _ = store.delete(state_id);
            }
        }

    }
}

/// Helper to share async loaded data.
///
/// - clnonable, shared state with change notifications.
/// - stores load result as `Option<Result<Rc<T>, Error>>`.
/// - tracks load state `self.loading()`.
/// - ability to cache result in local storage by setting `state_id`.
/// - helper to simplify renderering `self.render`.
///
#[derive(Derivative)]
#[derivative(Clone(bound = ""), PartialEq(bound = ""))]
pub struct Loader<T>(SharedState<LoaderState<T>>);

impl<T: 'static + DeserializeOwned + Serialize> Loader<T> {
    /// Create a new instance.
    pub fn new() -> Self {
        let state = LoaderState {
            loading: 0,
            state_id: None,
            data: None,
            loader: None,
        };
        Self(SharedState::new(state))
    }

    /// Builder style method to set the persistent state ID.
    pub fn state_id(mut self, state_id: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_state_id(state_id);
        self
    }

    /// Method to set the persistent state ID.
    pub fn set_state_id(&mut self, state_id: impl IntoPropValue<Option<AttrValue>>) {
    let mut me = self.write();
        me.state_id = state_id.into_prop_value();
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
        match self.read().data {
            Some(Ok(_)) => true,
            _ => false,
        }
    }

    pub fn render<R: Into<Html>>(&self, render: impl Fn(Rc<T>) -> R) -> Html {
        match &self.read().data {
            None => html! {
                <div class="pwt-text-center pwt-p-4">
                {Fa::new("spinner").class("pwt-me-1").pulse()}
                {"Loading..."}
                </div>
            },
            Some(Ok(ref data)) => render(Rc::clone(data)).into(),
            Some(Err(err)) => error_message(&format!("Error: {}", err), "pwt-p-2"),
        }
    }

    pub fn load(&self) {
        let loader = match &self.read().loader {
            Some(loader) => loader.clone(),
            None => return, // do nothing
        };

        self.write().loading += 1;
        let me = self.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let res = loader.apply().await;
            let mut me = me.write();
            me.loading -= 1;
            me.data = Some(res.map(|data| Rc::new(data)));
            me.store_to_cache();
        });
    }

    pub fn reload_button(&self) -> Button {
        let loader = self.clone();
        Button::refresh(self.loading()).onclick(move |_| loader.load())
    }
}
