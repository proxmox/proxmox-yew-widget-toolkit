use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::VNode;

use super::{IntoOptionalKey, WidgetStdProps};

/// Defines common builder methods for widgets.
pub trait WidgetBuilder: Into<VNode> {
    /// Acces to the widget [properties](WidgetStdProps).
    fn as_std_props(&self) -> &WidgetStdProps;

    /// Mutable acces to the widget [properties](WidgetStdProps).
    fn as_std_props_mut(&mut self) -> &mut WidgetStdProps;

    /// Copy properties from another [WidgetStdProps]
    fn with_std_props(mut self, props: &WidgetStdProps) -> Self {
        *self.as_std_props_mut() = props.clone();
        self
    }

    /// Builder style method to set the yew `key` property
    fn key(mut self, key: impl IntoOptionalKey) -> Self {
        self.set_key(key);
        self
    }

    /// Method to set the yew `key` property
    fn set_key(&mut self, key: impl IntoOptionalKey) {
        self.as_std_props_mut().key = key.into_optional_key();
    }

    /// Builder style method to add a html class
    fn class(mut self, class: impl Into<Classes>) -> Self {
        self.add_class(class);
        self
    }

    /// Method to add a html class.
    fn add_class(&mut self, class: impl Into<Classes>) {
        self.as_std_props_mut().class.push(class);
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
        self.as_std_props_mut().set_attribute(key, value);
    }

    /// Builder style method to set the element ID.
    fn id(mut self, id: impl Into<AttrValue>) -> Self {
        self.set_id(id);
        self
    }

    /// Method to set the element ID.
    fn set_id(&mut self, id: impl Into<AttrValue>) {
        self.set_attribute("id", id.into());
    }

    /// Builder style method to set the element tabindex.
    fn tabindex(mut self, tabindex: i32) -> Self {
        self.set_tabindex(tabindex);
        self
    }

    /// Method to set the element tabinde.
    fn set_tabindex(&mut self, tabindex: i32) {
        self.set_attribute("tabindex", tabindex.to_string());
    }
}
