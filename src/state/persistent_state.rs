use serde::{de::DeserializeOwned, Serialize};
use std::ops::Deref;

use super::StorageLocation;

/// Helper to store data persitently using window local [Storage](web_sys::Storage)
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
    state_id: String,
    data: T,
}

impl<T> Deref for PersistentState<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.data
    }
}

impl<T: 'static + Default + Serialize + DeserializeOwned> PersistentState<T> {
    /// Create a new instance, using 'state_id' as storage key in the
    /// local storage.
    ///
    /// See [Self::with_location] for details.
    pub fn new(state_id: &str) -> Self {
        Self::with_location(state_id, StorageLocation::Local)
    }

    /// Create a new instance, using 'state_id' as storage key from the given
    /// [StorageLocation]
    ///
    /// This automatically loads data from the storage.
    ///
    /// # Note
    ///
    /// Any errors are logged and ignored. Returns the default value
    /// in case of errors.
    pub fn with_location(state_id: &str, storage: StorageLocation) -> Self {
        let mut me = Self {
            state_id: state_id.into(),
            data: T::default(),
            storage,
        };
        me.load();
        me
    }

    fn load(&mut self) {
        if let Some(data) = super::load_state(&self.state_id, self.storage) {
            self.data = data;
        }
    }

    fn store(&self) {
        super::store_state(&self.state_id, &self.data, self.storage)
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
