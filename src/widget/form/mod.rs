//! Input Forms and Fields

mod context;
pub use context::{
    FieldHandle, FieldOptions, FormContext, FormContextObserver, FormContextReadGuard,
    FormContextState, FormContextWriteGuard,
};

mod boolean;
pub use boolean::{Boolean, PwtBoolean};

mod tristate_boolean;
pub use tristate_boolean::{PwtTristateBoolean, Tristate, TristateBoolean};

mod checkbox;
pub use checkbox::{Checkbox, PwtCheckbox};

mod combobox;
pub use combobox::{Combobox, PwtCombobox};

mod hidden;
pub use hidden::{Hidden, PwtHidden};

mod managed_field;
pub use managed_field::{
    ManagedField, ManagedFieldContext, ManagedFieldLink, ManagedFieldMaster, ManagedFieldState,
};

mod field;
pub use field::{Field, InputType, PwtField};

mod form;
pub use form::{Form, PwtForm};

mod number;
pub use number::{Number, PwtNumber};

mod reset_button;
pub use reset_button::{PwtResetButton, ResetButton};

mod selector;
pub use selector::{PwtSelector, Selector, SelectorRenderArgs};

mod submit_callback;
pub use submit_callback::{IntoSubmitCallback, SubmitCallback};

mod submit_button;
pub use submit_button::{PwtSubmitButton, SubmitButton};

mod textarea;
pub use textarea::{PwtTextArea, TextArea};

mod submit_validate;
pub use submit_validate::{IntoSubmitValidateFn, SubmitValidateFn};

mod validate;
pub use validate::{IntoValidateFn, ValidateFn};

// Propxmox API related helpers

use serde_json::{json, Value};

/// Proxmox API related helper: Delete empty values from the submit data.
///
/// And adds their names to the "delete" parameter.
///
/// By default only existing values are considered. if `delete_undefined` is
/// set, we also delete undefined values.
pub fn delete_existent_empty_values(record: &Value, param_list: &[&str], delete_undefined: bool) -> Value {
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

    if delete_undefined {
        for param in param_list {
            if record.get(param).is_none() {
                delete.push(param.to_string());
            }
        }
    }

    if !delete.is_empty() {
        new["delete"] = delete.into();
    }

    new
}
