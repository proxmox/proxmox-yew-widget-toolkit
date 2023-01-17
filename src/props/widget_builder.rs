use yew::prelude::*;
use yew::html::IntoPropValue;
use yew::virtual_dom::{ApplyAttributeAs, Key, VNode};

use super::{WidgetStdProps, Border, Margin, Padding}; 

/// Defines common builder methods for widgets.
pub trait WidgetBuilder: Into<VNode> {
    /// Mutable acces to the widget [properties](WidgetStdProps).
    fn as_std_props_mut(&mut self) -> &mut WidgetStdProps;

    /// Copy properties from another [WidgetStdProps]
    fn with_std_props(mut self, props: &WidgetStdProps) -> Self {
        *self.as_std_props_mut() = props.clone();
        self
    }

    /// Builder style method to set the yew `node_ref`
    fn node_ref(mut self, node_ref: NodeRef) -> Self {
        self.set_node_ref(node_ref);
        self
    }

    /// Method to set the yew `node_ref`
    fn set_node_ref(&mut self, node_ref: NodeRef) {
        self.as_std_props_mut().node_ref = node_ref;
    }

    /// Builder style method to set the yew `key` property
    fn key(mut self, key: impl IntoPropValue<Option<Key>>) -> Self {
        self.set_key(key);
        self
    }

    /// Method to set the yew `key` property
    fn set_key(&mut self, key: impl IntoPropValue<Option<Key>>) {
        self.as_std_props_mut().key = key.into_prop_value();
    }

    /// Builder style method to add a html class
    fn class(mut self, class: impl Into<Classes>) -> Self {
        self.add_class(class);
        self
    }

    /// Method to add a html class
    fn add_class(&mut self, class: impl Into<Classes>) {
        self.as_std_props_mut().class.push(class);
    }

    /// Builder style method to set the box border
    fn border(mut self, border: impl Into<Border>) -> Self {
        self.set_border(border);
        self
    }

    /// Method to set the box border
    fn set_border(&mut self, border: impl Into<Border>) {
        let border: Border = border.into();
        self.add_class(border.to_class());
    }

    /// Builder style method to set the box padding
    fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.set_padding(padding);
        self
    }

    /// Method to set the box padding
    fn set_padding(&mut self, padding: impl Into<Padding>) {
        let padding: Padding = padding.into();
        self.add_class(padding.to_class());
    }

    /// Builder style method to set the box margin
    fn margin(mut self, margin: impl Into<Margin>) -> Self {
        self.set_margin(margin);
        self
    }

    /// Method to set the box margin
    fn set_margin(&mut self, margin: impl Into<Margin>) {
        let margin: Margin = margin.into();
        self.add_class(margin.to_class());
    }

    /// Builder style method to set additional html attributes
    ///
    /// Note: Value 'None' removes the attribute.
    fn attribute(
        mut self,
        key: impl Into<AttrValue>,
        value: impl IntoPropValue<Option<AttrValue>>,
    ) -> Self {
        self.set_attribute(key, value);
        self
    }

    /// Method to set additional html attributes
    ///
    /// Note: Value 'None' removes the attribute.
    fn set_attribute(
        &mut self,
        key: impl Into<AttrValue>,
        value: impl IntoPropValue<Option<AttrValue>>,
    ) {
        if let Some(value) = value.into_prop_value() {
            self.as_std_props_mut().attributes.get_mut_index_map()
                .insert(key.into(), (value, ApplyAttributeAs::Attribute));
        } else {
            self.as_std_props_mut().attributes.get_mut_index_map()
                .remove(&key.into());
        }
    }
}
