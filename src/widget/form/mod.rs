//! Input Forms and Fields

mod context;
pub use context::{
    FormContext, FormContextState, FormContextObserver,
    FormContextReadGuard, FormContextWriteGuard,
    FieldHandle, FieldOptions,
};

//mod boolean;
//pub use boolean::{Boolean, PwtBoolean};

mod boolean2;
pub use boolean2::{Boolean, PwtBoolean};

mod checkbox2;
pub use checkbox2::{Checkbox, PwtCheckbox};

mod combobox;
pub use combobox::{Combobox, PwtCombobox};

mod field_state;
pub use field_state::{FieldState, FieldStateMsg};

mod managed_field;
pub use managed_field::{ManagedField, ManagedFieldContext, ManagedFieldLink, ManagedFieldMaster, ManagedFieldState};

//mod field;
//pub use field::{Field, PwtField};
mod field2;
pub use field2::{Field, PwtField};

mod form;
pub use form::{Form, PwtForm};

mod reset_button;
pub use reset_button::{ResetButton, PwtResetButton};

mod selector2;
pub use selector2::{Selector, SelectorRenderArgs, PwtSelector};

mod submit_callback;
pub use submit_callback::{SubmitCallback, IntoSubmitCallback};

mod submit_button;
pub use submit_button::{SubmitButton, PwtSubmitButton};

mod validate;
pub use validate::{IntoValidateFn, ValidateFn};

// Propxmox API related helpers

use serde_json::{json, Value};

/// Proxmox API related helper: Delete empty values from the submit data.
///
/// And adds their names to the "delete" parameter.
pub fn delete_empty_values(record: &Value, param_list: &[&str]) -> Value {
    let mut new = json!({});
    let mut delete: Vec<String> = Vec::new();

    for (param, v) in record.as_object().unwrap().iter() {
        if !param_list.contains(&param.as_str()) {
            new[param] = v.clone();
            continue;
        }
        if v.is_null() || (v.is_string() && v.as_str().unwrap().is_empty()) {
            delete.push(param.to_string());
        } else {
            new[param] = v.clone();
        }
    }

    for param in param_list {
        if record.get(param).is_none() {
            delete.push(param.to_string());
        }
    }

    new["delete"] = delete.into();

    new
}
