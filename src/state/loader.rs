use std::rc::Rc;

use anyhow::Error;
use derivative::Derivative;

use yew::html::IntoEventCallback;
use yew::prelude::*;

use crate::prelude::*;
use crate::props::{IntoLoadCallback, LoadCallback};
use crate::state::{SharedState, SharedStateObserver, SharedStateReadGuard, SharedStateWriteGuard};
use crate::widget::{error_message, Button, Fa};

/// Shared HTTP load state
///
/// This struct stores the state (loading) and the result of the load.
pub struct LoaderState<T> {
    loading: u64,
    pub loader: Option<LoadCallback<T>>,
    pub data: Option<Result<Rc<T>, Error>>,
}

/// Share HTTP loaded data.
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct Loader<T>(SharedState<LoaderState<T>>);

impl<T: 'static> Loader<T> {
    /// Create a new instance.
    pub fn new() -> Self {
        let state = LoaderState {
            loading: 0,
            data: None,
            loader: None,
        };
        Self(SharedState::new(state))
    }

    pub fn on_change(mut self, cb: impl IntoEventCallback<Loader<T>>) -> Self {
        let me = self.clone();
        match cb.into_event_callback() {
            Some(cb) => self.0.set_on_change(move |_| cb.emit(me.clone())),
            _ => self.0.set_on_change(None::<Callback<SharedState<LoaderState<T>>>>),
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
        });
    }

    pub fn reload_button(&self) -> Button {
        let loader = self.clone();
        Button::refresh(self.loading()).onclick(move |_| loader.load())
    }
}
