//! Input Forms and Fields

mod context;
pub use context::{
    FieldHandle, FieldOptions, FormContext, FormContextObserver, FormContextReadGuard,
    FormContextState, FormContextWriteGuard,
};

mod tristate_boolean;
#[doc(hidden)]
pub use tristate_boolean::PwtTristateBoolean;
pub use tristate_boolean::{Tristate, TristateBoolean};

mod checkbox;
pub use checkbox::Checkbox;
#[doc(hidden)]
pub use checkbox::PwtCheckbox;

mod radio_button;
#[doc(hidden)]
pub use radio_button::PwtRadioButton;
pub use radio_button::RadioButton;

mod combobox;
pub use combobox::Combobox;
#[doc(hidden)]
pub use combobox::PwtCombobox;

mod display;
pub use display::DisplayField;
#[doc(hidden)]
pub use display::PwtDisplayField;

mod hidden;
pub use hidden::Hidden;
#[doc(hidden)]
pub use hidden::PwtHidden;

mod managed_field;
pub use managed_field::{
    ManagedField, ManagedFieldContext, ManagedFieldLink, ManagedFieldMaster, ManagedFieldState,
};

mod field;
#[doc(hidden)]
pub use field::PwtField;
pub use field::{Field, InputType};

#[allow(clippy::module_inception)]
mod form;
pub use form::Form;
#[doc(hidden)]
pub use form::PwtForm;

mod number;
pub use number::{Number, NumberTypeInfo};
#[doc(hidden)]
pub use number::PwtNumber;

mod reset_button;
#[doc(hidden)]
pub use reset_button::PwtResetButton;
pub use reset_button::ResetButton;

mod selector;
pub use selector::{PwtSelector, Selector, SelectorRenderArgs};

mod submit_button;
#[doc(hidden)]
pub use submit_button::PwtSubmitButton;
pub use submit_button::SubmitButton;

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
pub fn delete_empty_values(record: &Value, param_list: &[&str], delete_undefined: bool) -> Value {
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
