use std::rc::Rc;
use std::cell::RefCell;

use anyhow::Error;

use yew::prelude::*;
use yew::html::IntoPropValue;

use crate::prelude::*;
use crate::props::{LoadCallback, IntoLoadCallback};
use crate::widget::{error_message, Button, Fa};

pub struct LoaderState<T> {
    pub loading: u64,
    pub data: Option<Result<Rc<T>, Error>>,
}

pub struct Loader<T> {
    state: Rc<RefCell<LoaderState<T>>>,
    loader: Option<LoadCallback<T>>,
    onchange: Callback<()>,
}

impl<T> Clone for Loader<T> {
    fn clone(&self) -> Self {
        Self {
            state: Rc::clone(&self.state),
            loader: self.loader.clone(),
            onchange: self.onchange.clone(),
        }
    }
}

impl<T: 'static> Loader<T> {

    pub fn new(onchange: impl Into<Callback<()>>) -> Self {
        Self {
            state: Rc::new(RefCell::new(
                LoaderState {
                    loading: 0,
                    data: None,
                }
            )),
            loader: None,
            onchange: onchange.into(),
        }
    }

    /// Builder style method to set the load callback.
    pub fn loader(mut self, callback: impl IntoLoadCallback<T>) -> Self {
        self.set_loader(callback);
        self
    }

    /// Method to set the load callback.
    pub fn set_loader(&mut self, callback: impl IntoLoadCallback<T>) {
        self.loader = callback.into_load_callback();
    }

    pub fn data(mut self, data: impl IntoPropValue<Option<Rc<T>>>) -> Self {
        self.set_data(data);
        self
    }

    pub fn set_data(&mut self, data: impl IntoPropValue<Option<Rc<T>>>) {
        let data = match data.into_prop_value() {
            Some(data) => data,
            None => return, // do nothing
        };

        let mut state = self.state.borrow_mut();
        if let Some(Ok(old_data)) = &state.data {
            if Rc::ptr_eq(&old_data, &data) {
                return; // same data, do nothing
            }
        }
        state.data = Some(Ok(data));
        self.onchange.emit(());
    }

    pub fn loading(&self) -> bool {
        self.state.borrow().loading > 0
    }

    pub fn has_valid_data(&self) -> bool {
        match self.state.borrow().data {
            Some(Ok(_)) => true,
            _ => false,
        }
    }

    pub fn with_state<R>(&self, cb: impl Fn(&LoaderState<T>) -> R) -> R {
        cb(&self.state.borrow())
    }

    pub fn render<R: Into<Html>>(
        &self,
        render: impl Fn(Rc<T>) -> R,
    ) -> Html {
        let state = &self.state.borrow();
        match &state.data {
            None => html!{
                <div class="pwt-text-center pwt-p-4">
                {Fa::new("spinner").class("pwt-me-1").pulse()}
                {"Loading..."}
                </div>
            },
            Some(Ok(ref data)) => {
                render(Rc::clone(data)).into()
            }
            Some(Err(err)) => {
                error_message(&format!("Error: {}", err), "pwt-p-2")
            }
        }
    }

    pub fn load(&self) {
        if let Some(loader) = self.loader.clone() {

            let state = self.state.clone();
            let onchange = self.onchange.clone();

            state.borrow_mut().loading += 1;
            onchange.emit(());

            wasm_bindgen_futures::spawn_local(async move {
                let res = loader.apply().await;
                let mut state = state.borrow_mut();
                state.loading -= 1;
                state.data = Some(res.map(|data| Rc::new(data)));
                onchange.emit(());
            });
        }
    }

    pub fn reload_button(&self) -> Button {
        let loader = self.clone();
        Button::refresh(self.loading())
            .onclick(move |_| loader.load())
    }

}
