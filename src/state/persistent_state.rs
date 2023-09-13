use serde::{de::DeserializeOwned, Serialize};
use std::ops::Deref;

use crate::state::local_storage;

pub struct PersistentState<T> {
    name: String,
    data: T,
}

impl<T> Deref for PersistentState<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.data
    }
}

impl<T: Default + Serialize + DeserializeOwned> PersistentState<T> {
    pub fn new(name: &str) -> Self {
        let mut me = Self {
            name: name.into(),
            data: T::default(),
        };
        me.load();
        me
    }

    pub fn load(&mut self) {
        let store = match local_storage() {
            Some(store) => store,
            None => {
                log::error!(
                    "load persistent state {} failed - cannot access local storage",
                    self.name
                );
                return;
            }
        };

        if let Ok(Some(item_str)) = store.get_item(&self.name) {
            if let Ok(data) = serde_json::from_str(&item_str) {
                self.data = data;
            }
        }
    }

    pub fn store(&self) {
        let store = match local_storage() {
            Some(store) => store,
            None => {
                log::error!(
                    "store persistent state {} failed - cannot access local storage",
                    self.name
                );
                return;
            }
        };

        let item_str = serde_json::to_string(&self.data).unwrap();
        match store.set_item(&self.name, &item_str) {
            Err(err) => log::error!(
                "store persistent state {} failed: {}",
                self.name,
                crate::convert_js_error(err)
            ),
            Ok(_) => {}
        }
    }

    pub fn update(&mut self, data: T) {
        self.data = data;
        self.store();
    }
}
