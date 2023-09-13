use serde::{de::DeserializeOwned, Serialize};
use std::ops::Deref;

use crate::state::local_storage;

/// Helper to store data persitently using window local [Storage](web_sys::Storage)
///
/// Usage:
///
/// ```
/// # fn test() {
/// use pwt::state::PersistentState;
///
/// let mut state = PersistentState::<bool>::new("my-storage-key-name");
///
/// let cunnent_value: bool = *state; // acess value with Deref
///
/// state.update(true); // update the value
/// # }
/// ```
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
    /// Create a new instance, using 'name' as storage key.
    ///
    /// This automatically loads data from the storage.
    ///
    /// # Note
    ///
    /// Any errors are logged and ignored. Returns the default value
    /// in case of errors.
    pub fn new(name: &str) -> Self {
        let mut me = Self {
            name: name.into(),
            data: T::default(),
        };
        me.load();
        me
    }

    fn load(&mut self) {
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

    fn store(&self) {
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

    /// Update data and write the new value back to the storage.
    ///
    /// # Note
    ///
    /// Any errors are logged and ignored.
    pub fn update(&mut self, data: T) {
        self.data = data;
        self.store();
    }
}
