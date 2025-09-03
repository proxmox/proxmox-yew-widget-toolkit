use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::{VNode, VTag};

use pwt_macros::widget;

use crate::props::{IntoVTag, WidgetBuilder};

use super::SvgLength;

/// Use another SVG object referenced by ID (SVG `<use>` element).
///
/// Attributes x, y, width, height always overwrite values from reference object.
///
/// Presentation attributes are ignored if the corresponding attribute
/// is already defined on the referenced element.
#[widget(pwt=crate, @element, @svg)]
#[derive(Properties, Clone, PartialEq)]
pub struct Reference {
    #[prop_or_default]
    children: Option<Vec<VNode>>,
}

impl Reference {
    /// Create a new instance.
    pub fn new(href: impl Into<AttrValue>) -> Self {
        yew::props!(Self {}).attribute("href", href.into())
    }

    impl_svg_position_attributes!();

    /// Builder style method to set the reference width.
    pub fn width(mut self, width: impl Into<SvgLength>) -> Self {
        self.set_width(width);
        self
    }

    /// Method to set the reference width.
    pub fn set_width(&mut self, width: impl Into<SvgLength>) {
        self.set_attribute("width", width.into());
    }

    /// Builder style method to set the reference height.
    pub fn height(mut self, height: impl Into<SvgLength>) -> Self {
        self.set_height(height);
        self
    }

    /// Method to set the reference height.
    pub fn set_height(&mut self, height: impl Into<SvgLength>) {
        self.set_attribute("height", height.into());
    }

    impl_svg_animation_attributes!();
    impl_svg_presentation_attributes!();
}

impl IntoVTag for Reference {
    fn into_vtag_with_ref(self, node_ref: NodeRef) -> VTag {
        self.std_props.into_vtag(
            Cow::Borrowed("use"),
            node_ref,
            None::<&str>,
            Some(self.listeners),
            self.children,
        )
    }
}
