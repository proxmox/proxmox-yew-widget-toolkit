use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::VTag;

use pwt_macros::widget;

use crate::props::WidgetBuilder;

use super::SvgLength;

/// SVG `<rect>` element.
#[widget(pwt=crate, @element, @svg)]
#[derive(Properties, Clone, PartialEq)]
pub struct Rect {}

impl Rect {

    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    impl_svg_position_attributes!();

    /// Builder style method to set the rect width.
    pub fn width(mut self, width: impl Into<SvgLength>) -> Self {
        self.set_width(width);
        self
    }

    /// Method to set the rect width.
    pub fn set_width(&mut self, width: impl Into<SvgLength>) {
        self.set_attribute("width", width.into());
    }

    /// Builder style method to set the rect height.
    pub fn height(mut self, height: impl Into<SvgLength>) -> Self {
        self.set_height(height);
        self
    }

    /// Method to set the rect height.
    pub fn set_height(&mut self, height: impl Into<SvgLength>) {
        self.set_attribute("height", height.into());
    }

    /// Builder style method to set the horizontal corner radius of the rect.
    pub fn rx(mut self, r: impl Into<SvgLength>) -> Self {
        self.set_rx(r);
        self
    }

    /// Method to set the horizontal corner radius of the rect.
    pub fn set_rx(&mut self, r: impl Into<SvgLength>) {
        self.set_attribute("rx", r.into());
    }

    /// Builder style method to set the vertical corner radius of the rect.
    pub fn ry(mut self, r: impl Into<SvgLength>) -> Self {
        self.set_ry(r);
        self
    }

    /// Method to set the vertical corner radius of the rect.
    pub fn set_ry(&mut self, r: impl Into<SvgLength>) {
        self.set_attribute("ry", r.into());
    }

    impl_svg_presentation_attributes!();
}

impl Into<VTag> for Rect {
    fn into(self) -> VTag {
        self.std_props.into_vtag(Cow::Borrowed("rect"), Some(self.listeners), None)
    }
}
