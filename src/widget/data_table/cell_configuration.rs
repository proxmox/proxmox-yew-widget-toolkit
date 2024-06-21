use yew::{
    html::IntoPropValue,
    virtual_dom::{ApplyAttributeAs, Attributes},
    AttrValue, Classes,
};

use crate::props::{
    AsClassesMut, AsCssStylesMut, CssBorderBuilder, CssMarginBuilder, CssPaddingBuilder, CssStyles,
    WidgetStyleBuilder,
};

#[derive(Clone, Default, PartialEq)]
pub struct CellConfiguration {
    pub class: Classes,
    pub style: CssStyles,
    pub attributes: Attributes,
}

impl AsClassesMut for CellConfiguration {
    fn as_classes_mut(&mut self) -> &mut yew::Classes {
        &mut self.class
    }
}

impl AsCssStylesMut for CellConfiguration {
    fn as_css_styles_mut(&mut self) -> &mut CssStyles {
        &mut self.style
    }
}

impl CssBorderBuilder for CellConfiguration {}
impl CssMarginBuilder for CellConfiguration {}
impl CssPaddingBuilder for CellConfiguration {}
impl WidgetStyleBuilder for CellConfiguration {}

impl CellConfiguration {
    pub fn new() -> Self {
        Default::default()
    }

    /// Method to set attributes.
    ///
    /// Note: Value 'None' removes the attribute.
    pub fn set_attribute(
        &mut self,
        key: impl Into<AttrValue>,
        value: impl IntoPropValue<Option<AttrValue>>,
    ) {
        if let Some(value) = value.into_prop_value() {
            self.attributes
                .get_mut_index_map()
                .insert(key.into(), (value, ApplyAttributeAs::Attribute));
        } else {
            self.attributes.get_mut_index_map().swap_remove(&key.into());
        }
    }

    /// Builder style method to set attributes.
    ///
    /// Note: Value 'None' removes the attribute.
    pub fn attribute(
        mut self,
        key: impl Into<AttrValue>,
        value: impl IntoPropValue<Option<AttrValue>>,
    ) -> Self {
        self.set_attribute(key, value);
        self
    }

    pub fn add_class(&mut self, class: impl Into<Classes>) {
        self.class.push(class)
    }

    pub fn class(mut self, class: impl Into<Classes>) -> Self {
        self.add_class(class);
        self
    }
}
