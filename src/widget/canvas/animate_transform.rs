use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::{ApplyAttributeAs, Attributes, Listeners, VList, VNode, VTag};

use super::IntoSvgAnimation;

/// Wrapper for SVG `<animateTransform>`
#[derive(Default, Debug, Clone, PartialEq, Properties)]
pub struct AnimateTransform {
    /// Attributes.
    #[prop_or_default]
    pub attributes: Attributes,
}

impl IntoSvgAnimation for AnimateTransform {
    fn into_svg_animation(self) -> VNode {
        self.into()
    }
}

impl AnimateTransform {
    /// Creates a new instance without any attributes set.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Create a new `rotate` instance.
    ///
    /// This is a shortcut for:
    ///
    /// ```
    /// # use pwt::widget::canvas::AnimateTransform;
    /// AnimateTransform::new()
    ///     .attribute("attributeName", "transform")
    ///     .attribute("type", "rotate")
    /// # ;
    /// ```
    pub fn rotate() -> Self {
        yew::props!(Self {})
            .attribute("attributeName", "transform")
            .attribute("type", "rotate")
    }

    /// Create a new `scale` instance.
    ///
    /// This is a shortcut for:
    ///
    /// ```
    /// # use pwt::widget::canvas::AnimateTransform;
    /// AnimateTransform::new()
    ///     .attribute("attributeName", "transform")
    ///     .attribute("type", "scale")
    /// # ;
    /// ```
    pub fn scale() -> Self {
        yew::props!(Self {})
            .attribute("attributeName", "transform")
            .attribute("type", "scale")
    }

    /// Create a new `translate` instance.
    ///
    /// This is a shortcut for:
    ///
    /// ```
    /// # use pwt::widget::canvas::AnimateTransform;
    /// AnimateTransform::new()
    ///     .attribute("attributeName", "transform")
    ///     .attribute("type", "translate")
    /// # ;
    /// ```
    pub fn translate() -> Self {
        yew::props!(Self {})
            .attribute("attributeName", "transform")
            .attribute("type", "translate")
    }

    /// Create a new `skew` instance.
    ///
    /// This is a shortcut for:
    ///
    /// ```
    /// # use pwt::widget::canvas::AnimateTransform;
    /// AnimateTransform::new()
    ///     .attribute("attributeName", "transform")
    ///     .attribute("type", "skew")
    /// # ;
    /// ```
    pub fn skew() -> Self {
        yew::props!(Self {})
            .attribute("attributeName", "transform")
            .attribute("type", "skew")
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
            self.attributes
                .get_mut_index_map()
                .insert(key.into(), (value, ApplyAttributeAs::Attribute));
        } else {
            self.attributes.get_mut_index_map().swap_remove(&key.into());
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
        self.set_attribute(
            "repeatCount",
            if count.is_infinite() {
                AttrValue::Static("indefinite")
            } else {
                count.to_string().into()
            },
        );
    }

    /// Builder style method to set the `additive` property.
    pub fn additive(mut self, additive: bool) -> Self {
        self.set_additive(additive);
        self
    }

    /// Method to set the `additive` property.
    pub fn set_additive(&mut self, additive: bool) {
        self.set_attribute("additive", if additive { "sum" } else { "replace" });
    }

    /// Builder style method to set the `accumulate` property.
    pub fn accumulate(mut self, accumulate: bool) -> Self {
        self.set_accumulate(accumulate);
        self
    }

    /// Method to set the `accumulate` property.
    pub fn set_accumulate(&mut self, accumulate: bool) {
        self.set_attribute("accumulate", if accumulate { "sum" } else { "none" });
    }
}

impl Into<VTag> for AnimateTransform {
    fn into(self) -> VTag {
        VTag::__new_other(
            "animateTransform".into(),
            NodeRef::default(),
            None,
            self.attributes,
            Listeners::None,
            VList::new().into(),
        )
    }
}

impl Into<VNode> for AnimateTransform {
    fn into(self) -> VNode {
        let vtag: VTag = self.into();
        VNode::from(vtag)
    }
}
