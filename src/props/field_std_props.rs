use serde_json::Value;
use indexmap::IndexMap;

use yew::prelude::*;

use crate::state::{FieldFormRef, FieldOptions};

#[derive(PartialEq, Clone, Properties)]
pub struct FieldStdProps {
    pub form_ref: Option<FieldFormRef>,
    pub label_id: Option<AttrValue>,

    pub tabindex: Option<i32>,
    pub aria_label: Option<AttrValue>,
    pub placeholder: Option<AttrValue>,

    #[prop_or_default]
    pub autofocus: bool,

    #[prop_or_default]
    pub disabled: bool,

    #[prop_or_default]
    pub required: bool,

    #[prop_or(true)]
    pub submit: bool,

    #[prop_or_default]
    pub submit_empty: bool,
}

impl Default for FieldStdProps {
    fn default() -> Self { Self::new() }
}

impl FieldStdProps {
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    pub fn cumulate_attributes(&self, attr_map: &mut IndexMap<AttrValue, AttrValue>) {
        if self.disabled {
            attr_map.insert(AttrValue::Static("disabled"), AttrValue::Static(""));
        }

         if self.required {
            attr_map.insert(AttrValue::Static("required"), AttrValue::Static(""));
        }

        if self.autofocus {
            attr_map.insert(AttrValue::Static("autofocus"), AttrValue::Static(""));
        }

        if let Some(ref aria_label) = self.aria_label {
            attr_map.insert(AttrValue::Static("aria-label"), aria_label.clone());
        }

        if let Some(ref label_id) = self.label_id {
            attr_map.insert(AttrValue::Static("aria-labelledby"), label_id.clone());
        }

        if let Some(ref tabindex) = self.tabindex {
            attr_map.insert(AttrValue::Static("tabindex"), tabindex.to_string().into());
        }

        if let Some(ref placeholder) = self.placeholder {
            attr_map.insert(AttrValue::Static("placeholder"), placeholder.clone());
        }
    }

    pub fn register_form_field(&self, value: Value, valid: Result<(), String>) {
        if let Some(form_ref) = &self.form_ref {
            let mut options = FieldOptions::new();
            options.submit = self.submit;
            options.submit_empty = self.submit_empty;
            form_ref.register_field(value, valid, options);
        }
    }
}
