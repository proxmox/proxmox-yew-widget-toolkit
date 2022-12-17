use serde_json::Value;
use crate::props::FieldStdProps;
use crate::widget::form::ValidateFn; // fixme: move to props

use yew::AttrValue;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FieldOptions {
    pub submit: bool,
    pub submit_empty: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FieldState {
    pub validate: Option<ValidateFn<Value>>,
    pub initial_value: Value,
    pub initial_valid: Result<(), String>,
    pub value: Value,
    pub valid: Result<(), String>,
    options: FieldOptions,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FieldRegistration {
    pub name: AttrValue,
    pub validate: Option<ValidateFn<Value>>,
    pub submit: bool,
    pub submit_empty: bool,
}
