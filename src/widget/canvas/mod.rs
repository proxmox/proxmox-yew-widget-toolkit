mod circle;
pub use circle::Circle;

mod group;
pub use group:: Group;

mod hyperlink;
pub use hyperlink::Hyperlink;

mod line;
pub use line::Line;

mod polygon;
pub use polygon::Polygon;

mod path;
pub use path::Path;

mod polyline;
pub use polyline::Polyline;

mod rect;
pub use rect::Rect;

mod text;
pub use text::Text;

mod tspan;
pub use tspan::TSpan;

use std::fmt::Display;
use std::borrow::Cow;

use yew::prelude::*;
use yew::html::IntoPropValue;
use yew::virtual_dom::VTag;

use crate::props::WidgetBuilder;

use pwt_macros::widget;

#[derive(Copy, Clone, PartialEq)]
pub enum SvgLength {
    Px(f32),
    Em(f32),
    Fraction(f32),
}

impl Default for SvgLength {
    fn default() -> Self {
        SvgLength::Px(0.0)
    }
}

impl Display for SvgLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            SvgLength::Px(v) => write!(f, "{v}px"),
            SvgLength::Em(v) => write!(f, "{v}em"),
            SvgLength::Fraction(v) => write!(f, "{}%", v*100.0),
        }
    }
}

impl From<f32> for SvgLength {
    fn from(v: f32) -> SvgLength {
        SvgLength::Px(v)
    }
}

impl From<usize> for SvgLength {
    fn from(v: usize) -> SvgLength {
        SvgLength::Px(v as f32)
    }
}

impl From<i32> for SvgLength {
    fn from(v: i32) -> SvgLength {
        SvgLength::Px(v as f32)
    }
}

impl Into<AttrValue> for SvgLength {
    fn into(self) -> AttrValue {
        self.to_string().into()
    }
}

impl IntoPropValue<Option<AttrValue>> for SvgLength {
    fn into_prop_value(self) -> Option<AttrValue> {
        Some(self.into())
    }
}

/// SVG canvas (Html `<svg>` container).
#[widget(pwt=crate, @element, @container)]
#[derive(Properties, Clone, PartialEq)]
pub struct Canvas {}

impl Canvas {
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    pub fn width(mut self, w: impl Into<SvgLength>) -> Self {
        self.set_width(w);
        self
    }

    pub fn set_width(&mut self, w: impl Into<SvgLength>) {
        self.set_attribute("width", w.into().to_string());
    }

    pub fn height(mut self, h: impl Into<SvgLength>) -> Self {
        self.set_height(h);
        self
    }

    pub fn set_height(&mut self, h: impl Into<SvgLength>) {
        self.set_attribute("height", h.into().to_string());
    }
}

impl Into<VTag> for Canvas {
    fn into(self) -> VTag {
        self.std_props.into_vtag(Cow::Borrowed("svg"), Some(self.listeners), Some(self.children))
    }
}
