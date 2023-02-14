use yew::prelude::*;
use yew::virtual_dom::{Attributes, ApplyAttributeAs, Listeners, VList, VNode, VTag};
use yew::html::IntoPropValue;

pub trait IntoSvgAnimation {
    fn into_svg_animation(self) -> VNode;
}

/// Wrapper for SVG `<animate>`
#[derive(Default, Debug, Clone, PartialEq, Properties)]
pub struct Animate {
    /// Attributes.
    #[prop_or_default]
    pub attributes: Attributes,
}

impl IntoSvgAnimation for Animate {
    fn into_svg_animation(self) -> VNode {
        self.into()
    }
}

impl Animate {

    /// Creates a new instance to animate the named attribute.
    pub fn new(attribute_name: impl Into<AttrValue>) -> Self {
        yew::props!(Self {})
            .attribute("attributeName", attribute_name.into())
    }

    /// Builder style method to set attributes
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

    /// Method to set attributes.
    ///
    /// Note: Value 'None' removes the attribute.
    pub fn set_attribute(
        &mut self,
        key: impl Into<AttrValue>,
        value: impl IntoPropValue<Option<AttrValue>>,
    ) {
        if let Some(value) = value.into_prop_value() {
            self.attributes.get_mut_index_map()
                .insert(key.into(), (value, ApplyAttributeAs::Attribute));
        } else {
            self.attributes.get_mut_index_map()
                .remove(&key.into());
        }
    }

    /// Builder style method to set the repeat count.
    pub fn repeat_count(mut self, count: f32) -> Self {
        self.set_repeat_count(count);
        self
    }

    /// Method to set the repeat count.
    ///
    /// You can use [f32::INFINITY] for indefinite repeat.
    pub fn set_repeat_count(&mut self, count: f32) {
        self.set_attribute("repeatCount", if count.is_infinite() {
            AttrValue::Static("indefinite")
        } else {
            count.to_string().into()
        });
    }

    /// Builder style method to set the `additive` property.
    pub fn additive(mut self, additive: bool) -> Self {
        self.set_additive(additive);
        self
    }

    /// Method to set the `additive` property.
    pub fn set_additive(&mut self, additive: bool){
        self.set_attribute("additive", if additive { "sum" } else { "replace" });
    }

    /// Builder style method to set the `accumulate` property.
    pub fn accumulate (mut self, accumulate: bool) -> Self {
        self.set_accumulate(accumulate);
        self
    }

    /// Method to set the `accumulate` property.
    pub fn set_accumulate(&mut self, accumulate: bool){
        self.set_attribute("accumulate", if accumulate { "sum" } else { "none" });
    }
}

impl Into<VTag> for Animate {
    fn into(self) -> VTag {
        VTag::__new_other(
            "animate".into(),
            NodeRef::default(),
            None,
            self.attributes,
            Listeners::None,
            VList::new(),
        )
    }
}

impl Into<VNode> for Animate {
    fn into(self) -> VNode {
        let vtag: VTag = self.into();
        VNode::from(vtag)
    }
}
