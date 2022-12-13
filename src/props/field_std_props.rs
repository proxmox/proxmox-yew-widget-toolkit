use indexmap::IndexMap;

use yew::prelude::*;
use yew::virtual_dom::ApplyAttributeAs;

#[derive(PartialEq, Clone, Properties)]
pub struct FieldStdProps {
    /// The field register itself with this `name` in the FormContext
    /// (if any).
    pub name: Option<AttrValue>,

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

    pub fn cumulate_attributes(&self, attr_map: &mut IndexMap<AttrValue, (AttrValue, ApplyAttributeAs)>) {
        if self.disabled {
            attr_map.insert(AttrValue::Static("disabled"), (AttrValue::Static(""), ApplyAttributeAs::Attribute));
        }

         if self.required {
            attr_map.insert(AttrValue::Static("required"), (AttrValue::Static(""), ApplyAttributeAs::Attribute));
        }

        if self.autofocus {
            attr_map.insert(AttrValue::Static("autofocus"), (AttrValue::Static(""), ApplyAttributeAs::Attribute));
        }

        if let Some(ref aria_label) = self.aria_label {
            attr_map.insert(AttrValue::Static("aria-label"), (aria_label.clone(), ApplyAttributeAs::Attribute));
        }

        if let Some(ref label_id) = self.label_id {
            attr_map.insert(AttrValue::Static("aria-labelledby"), (label_id.clone(), ApplyAttributeAs::Attribute));
        }

        if let Some(ref tabindex) = self.tabindex {
            attr_map.insert(AttrValue::Static("tabindex"), (tabindex.to_string().into(), ApplyAttributeAs::Attribute));
        }

        if let Some(ref placeholder) = self.placeholder {
            attr_map.insert(AttrValue::Static("placeholder"), (placeholder.clone(), ApplyAttributeAs::Attribute));
        }
    }
}
