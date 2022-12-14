//! State management helpers

use std::rc::Rc;


mod data_store;
pub use data_store::{DataStore, DataNode, DataNodeDerefGuard};

//mod data_filter;
//pub use data_filter::{optional_rc_ptr_eq, DataFilter};

mod loader;
pub use loader::{Loader, LoaderState};

mod navigation_container;
pub use navigation_container::{
    NavigationContainer, NavigationContext, NavigationContextExt, PwtNavigationContainer
};

mod selection;
pub use selection::{use_selection, Selection, SelectionObserver};

mod store;
pub use store::*;

mod tree_store;
pub use tree_store::*;

mod theme;
pub use theme::Theme;

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

pub fn optional_rc_ptr_eq<T>(a: &Option<Rc<T>>, b: &Option<Rc<T>>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Rc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}
