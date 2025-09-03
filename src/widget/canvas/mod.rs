//! Drawing Canvas (SVG wrapper)
//!
//! The [Canvas] component creates an Html `<svg>` element. SVG elements are
//! mapped into the DOM, so you can use the yew html marco to create
//! objects inside the canvas (as long as the are valid SVG elements like `<circle>`, `<line>`, ...).
//!
//! ```
//! use pwt::prelude::*;
//! use pwt::widget::canvas::Canvas;
//! # fn test() -> Canvas {
//!
//! Canvas::new()
//!     .with_child(html!{ <circle cx="50" cy="50" r="50" />})
//! # }
//! ```
//!
//! We also provide rust types to build common SVG element, so you can
//! rewite above example as:
//!
//! ```
//! # use pwt::prelude::*;
//! # use pwt::widget::canvas::{Canvas, Circle};
//! # fn test() -> Canvas {
//! Canvas::new()
//!     .with_child(
//!          Circle::new()
//!              .cx(50)
//!              .cy(50)
//!              .r(50)
//!     )
//! # }
//! ```
//!
//! SVG elements use the same event model as Html element, so you can
//! also attach event hanlers:
//!
//! ```
//! # use pwt::prelude::*;
//! # use pwt::widget::canvas::{Canvas, Circle};
//! # fn test() -> Canvas {
//! Canvas::new()
//!     .with_child(
//!          Circle::new()
//!              .position(50, 50)
//!              .r(50)
//!              .onclick(Callback::from(|_| log::info!("Circle clicked!")))
//!     )
//! # }

#[macro_use]
mod macros;

mod animate;
pub use animate::{Animate, IntoSvgAnimation};

mod animate_transform;
pub use animate_transform::AnimateTransform;

mod circle;
pub use circle::Circle;

mod ellipse;
pub use ellipse::Ellipse;

mod group;
pub use group::Group;

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

mod reference;
pub use reference::Reference;

mod rect;
pub use rect::Rect;

mod text;
pub use text::Text;

mod tspan;
pub use tspan::TSpan;

use std::borrow::Cow;
use std::fmt::Display;

use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::VTag;

use crate::props::{IntoVTag, WidgetBuilder};

use pwt_macros::widget;

/// SVG length in pixel, em or percentage.
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
            SvgLength::Fraction(v) => write!(f, "{}%", v * 100.0),
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

impl From<SvgLength> for AttrValue {
    fn from(val: SvgLength) -> Self {
        val.to_string().into()
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

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}

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

impl IntoVTag for Canvas {
    fn into_vtag_with_ref(self, node_ref: NodeRef) -> VTag {
        self.std_props.into_vtag(
            Cow::Borrowed("svg"),
            node_ref,
            None::<&str>,
            Some(self.listeners),
            Some(self.children),
        )
    }
}
