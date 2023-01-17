use yew::prelude::*;
use yew::html::IntoPropValue;
use yew::virtual_dom::{ApplyAttributeAs, Attributes, Key};

/// Standard widget properties.
#[derive(PartialEq, Debug, Default, Clone)]
pub struct WidgetStdProps {

    /// The yew node ref.
    pub node_ref: NodeRef,

    /// The yew component key.
    pub key: Option<Key>,

    /// CSS class.
    pub class: Classes,

    /// Additional Html attributes.
    pub attributes: Attributes,
}

impl WidgetStdProps {

    pub fn cumulate_attributes(&self, additional_class: Option<impl Into<Classes>>) -> Attributes {
        let mut class = self.class.clone();

        if let Some(additional_class) = additional_class {
            class.push(additional_class);
        }

        let mut attributes = self.attributes.clone();
        let attr_map = attributes.get_mut_index_map();
        attr_map.insert(AttrValue::Static("class"), (class.into_prop_value(), ApplyAttributeAs::Attribute));

        attributes
    }

}
