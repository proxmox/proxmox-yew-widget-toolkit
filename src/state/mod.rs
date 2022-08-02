//! State management helpers

mod loader;
pub use loader::{Loader, LoaderState};

mod form_state;
pub use form_state::{
    delete_empty_values, ButtonFormRef, FieldState, FieldFormRef,
    FieldOptions, FormState, 
};

mod selection;
pub use selection::Selection;

mod navigation_container;
pub use navigation_container::*; // fixme
