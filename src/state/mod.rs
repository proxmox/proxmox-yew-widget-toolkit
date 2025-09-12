//! State management helpers

use std::rc::Rc;

use serde::{de::DeserializeOwned, Serialize};

mod data_store;
pub use data_store::{DataNode, DataNodeDerefGuard, DataStore};

mod loader;
pub use loader::{Loader, LoaderState};

mod navigation_container;
pub use navigation_container::{
    NavigationContainer, NavigationContext, NavigationContextExt, PwtNavigationContainer,
};

mod persistent_state;
pub use persistent_state::PersistentState;

mod selection;
pub use selection::{use_selection, Selection, SelectionObserver};

mod shared_state;
pub use shared_state::{
    SharedState, SharedStateInner, SharedStateObserver, SharedStateReadGuard, SharedStateWriteGuard,
};

mod store;
pub use store::*;

mod tree_store;
pub use tree_store::*;

mod theme;
pub use theme::{
    get_available_themes, set_available_themes, Theme, ThemeDensity, ThemeMode, ThemeObserver,
};

mod language;
pub use language::{
    get_available_languages, get_language_info, set_available_languages, Language, LanguageInfo,
    LanguageObserver, TextDirection,
};

use crate::props::StorageLocation;

/// Helper function to get the window session [Storage](web_sys::Storage)
pub fn session_storage() -> Option<web_sys::Storage> {
    let window = match web_sys::window() {
        None => {
            log::error!("session_storage: no window");
            return None;
        }
        Some(window) => window,
    };

    let store = match window.session_storage() {
        Ok(Some(store)) => store,
        Ok(None) => {
            log::error!("session_storage: no session_storage");
            return None;
        }
        Err(_) => {
            log::error!("session_storage: security error");
            return None;
        }
    };

    Some(store)
}

/// Helper function to get the window local [Storage](web_sys::Storage)
pub fn local_storage() -> Option<web_sys::Storage> {
    let window = match web_sys::window() {
        None => {
            log::error!("local_storage: no window");
            return None;
        }
        Some(window) => window,
    };

    let store = match window.local_storage() {
        Ok(Some(store)) => store,
        Ok(None) => {
            log::error!("local_storage: no local_storage");
            return None;
        }
        Err(_) => {
            log::error!("local_storage: security error");
            return None;
        }
    };

    Some(store)
}

pub fn delete_state(storage: &StorageLocation) {
    let (store, state_id) = match storage {
        StorageLocation::Local(state_id) => (local_storage(), state_id),
        StorageLocation::Session(state_id) => (session_storage(), state_id),
    };
    if let Some(store) = store {
        let _ = store.delete(state_id);
    }
}

pub fn load_state<T: 'static + DeserializeOwned>(storage: &StorageLocation) -> Option<T> {
    let (store, state_id) = match storage {
        StorageLocation::Local(state_id) => (local_storage(), state_id),
        StorageLocation::Session(state_id) => (session_storage(), state_id),
    };

    if let Some(store) = store {
        if let Ok(Some(item_str)) = store.get_item(state_id) {
            if let Ok(data) = serde_json::from_str(&item_str) {
                return Some(data);
            }
        }
    }
    None
}

pub fn store_state<T: 'static + Serialize>(data: &T, storage: &StorageLocation) {
    let (store, state_id) = match storage {
        StorageLocation::Local(state_id) => (local_storage(), state_id),
        StorageLocation::Session(state_id) => (session_storage(), state_id),
    };
    if let Some(store) = store {
        let item_str = serde_json::to_string(data).unwrap();
        if let Err(err) = store.set_item(state_id, &item_str) {
            log::error!(
                "store persistent state {state_id} failed: {}",
                crate::convert_js_error(err)
            )
        }
    }
}

/// Helper to compare optional [Rc]s using [Rc::ptr_eq].
pub fn optional_rc_ptr_eq<T>(a: &Option<Rc<T>>, b: &Option<Rc<T>>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Rc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}
