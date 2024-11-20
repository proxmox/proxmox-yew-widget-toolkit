use serde::{de::DeserializeOwned, Serialize};
use std::ops::Deref;

use crate::props::StorageLocation;

/// Helper to store data persistently using window local [Storage](web_sys::Storage)
///
/// Usage:
///
/// ```
/// # fn test() {
/// use pwt::state::PersistentState;
/// use pwt::state::StorageLocation;
///
/// let mut state = PersistentState::<bool>::new("my-storage-key-name");
///
/// let cunnent_value: bool = *state; // acess value with Deref
///
/// state.update(true); // update the value
/// # }
/// ```
///
/// in session storage instead of local storage:
/// ```
/// # fn test() {
/// use pwt::state::PersistentState;
/// use pwt::state::StorageLocation;
///
/// let mut state = PersistentState::<bool>::with_location("my-storage-key-name", StorageLocation::Session);
///
/// let cunnent_value: bool = *state; // acess value with Deref
///
/// state.update(true); // update the value
/// # }
/// ```
pub struct PersistentState<T> {
    storage: StorageLocation,
    data: T,
}

impl<T> Deref for PersistentState<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.data
    }
}

impl<T: 'static + Default + Serialize + DeserializeOwned> PersistentState<T> {
    /// Create a new instance, using 'state_id' as storage location.
    ///
    /// See [Self::with_location] for details.
    pub fn new(storage: impl Into<StorageLocation>) -> Self {
        let mut me = Self {
            data: T::default(),
            storage: storage.into(),
        };
        me.load();
        me
    }

    fn load(&mut self) {
        if let Some(data) = super::load_state(&self.storage) {
            self.data = data;
        }
    }

    fn store(&self) {
        super::store_state(&self.data, &self.storage)
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

    /// Consumes self and returns the inner data
    pub fn into_inner(self) -> T {
        self.data
    }
}
