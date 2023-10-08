use serde::{de::DeserializeOwned, Serialize};
use std::ops::Deref;

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
    /// Create a new instance, using 'state_id' as storage key.
    ///
    /// This automatically loads data from the storage.
    ///
    /// # Note
    ///
    /// Any errors are logged and ignored. Returns the default value
    /// in case of errors.
    pub fn new(state_id: &str) -> Self {
        let mut me = Self {
            state_id: state_id.into(),
            data: T::default(),
        };
        me.load();
        me
    }

    fn load(&mut self) {
        if let Some(data) = super::load_state(&self.state_id) {
            self.data = data;
        }
    }

    fn store(&self) {
        super::store_state(&self.state_id, &self.data)
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
