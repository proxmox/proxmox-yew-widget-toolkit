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
#[doc(hidden)]
pub use number::PwtNumber;
pub use number::{Number, NumberTypeInfo};

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
