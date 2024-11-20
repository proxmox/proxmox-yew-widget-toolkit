//! Storage location for Application State

use yew::AttrValue;

/// Where the state should be saved
#[derive(Clone, PartialEq)]
pub enum StorageLocation {
    /// saved in the browser local storage
    /// <https://developer.mozilla.org/en-US/docs/Web/API/Window/localStorage>
    Local(AttrValue),
    /// saved in the browser session storage
    /// <https://developer.mozilla.org/en-US/docs/Web/API/Window/sessionStorage>
    Session(AttrValue),
}

impl StorageLocation {
    pub fn local(state_id: impl Into<AttrValue>) -> Self {
        Self::Local(state_id.into())
    }
    pub fn session(state_id: impl Into<AttrValue>) -> Self {
        Self::Session(state_id.into())
    }
}

pub trait IntoStorageLocation {
    fn into_storage_location(self) -> Option<StorageLocation>;
}

impl<I: Into<StorageLocation>> IntoStorageLocation for I {
    fn into_storage_location(self) -> Option<StorageLocation> {
        Some(self.into())
    }
}

impl<I: Into<StorageLocation>> IntoStorageLocation for Option<I> {
    fn into_storage_location(self) -> Option<StorageLocation> {
        self.map(|me| me.into())
    }
}

impl From<&'static str> for StorageLocation {
    fn from(state_id: &'static str) -> Self {
        StorageLocation::Local(AttrValue::Static(state_id))
    }
}

impl From<AttrValue> for StorageLocation {
    fn from(state_id: AttrValue) -> Self {
        StorageLocation::Local(state_id)
    }
}

impl From<String> for StorageLocation {
    fn from(state_id: String) -> Self {
        StorageLocation::Local(AttrValue::from(state_id))
    }
}
