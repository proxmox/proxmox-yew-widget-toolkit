use std::borrow::Cow;

use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::VTag;

use pwt_macros::{builder, widget};

use crate::props::{
    ContainerBuilder, CssLength, IntoOptionalTextRenderFn, IntoVTag, TextRenderFn, WidgetBuilder,
    WidgetStyleBuilder,
};
use crate::widget::Container;

/// Wrapper for Html `<meter>`.
#[widget(pwt=crate, @element)]
#[builder]
#[derive(Default, Clone, PartialEq, Properties)]
pub struct Meter {
    /// Minimum value (default 0)
    ///
    /// Lower numeric bound. This must be less than the maximum value.
    #[prop_or(0.0)]
    #[builder(IntoPropValue, into_prop_value, 1.0)]
    pub min: f32,

    /// Maximum value (default 1)
    ///
    /// Upper numeric bound. This must be greater than the minimum
    /// value.
    #[prop_or(1.0)]
    #[builder(IntoPropValue, into_prop_value, 1.0)]
    pub max: f32,

    /// Define the low end range.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub low: Option<f32>,

    /// Define the high end range.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub high: Option<f32>,

    /// Optimal value.
    ///
    /// This gives an indication where along the range is considered
    /// preferable. For example, if it is between the min attribute
    /// and the low attribute, then the lower range is considered
    /// preferred. The meter's bar color depends on whether the value
    /// is less than or equal to the optimum value.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub optimum: Option<f32>,

    /// Current value (default 0).
    ///
    /// This must be between the minimum and maximum values. If
    /// specified, but not within the range given by the min attribute
    /// and max attribute, the value is equal to the nearest end of
    /// the range.
    #[prop_or(0.0)]
    #[builder(IntoPropValue, into_prop_value, 0.0)]
    pub value: f32,

    /// Show value as text.
    #[builder_cb(IntoOptionalTextRenderFn, into_optional_text_render_fn, f32)]
    #[prop_or_default]
    pub render_text: Option<TextRenderFn<f32>>,

    /// Determines if the meter value transitions are animated (via CSS) or not.
    /// It is equivalent to setting the class `pwt-animated`.
    #[builder]
    #[prop_or_default]
    pub animated: bool,
}

impl Meter {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props! { Self {}}
    }

    fn get_range_index(&self, value: f32) -> usize {
        if value < self.low.unwrap_or(self.min) {
            0
        } else if value < self.high.unwrap_or(self.max) {
            1
        } else {
            2
        }
    }
}

impl IntoVTag for Meter {
    fn into_vtag_with_ref(self, node_ref: NodeRef) -> VTag {
        let percentage = ((self.value - self.min).max(0.0) / (self.max - self.min)).clamp(0.0, 1.0);

        let distance_to_optimum = if let Some(optimum) = self.optimum {
            if optimum > self.value {
                self.get_range_index(optimum) - self.get_range_index(self.value)
            } else {
                self.get_range_index(self.value) - self.get_range_index(optimum)
            }
        } else {
            0
        };

        let mut children = Vec::new();
        let mut class = classes!("pwt-meter");

        if let Some(render_text) = &self.render_text {
            let text = render_text.apply(&self.value);
            children.push(
                Container::new()
                    .class("pwt-meter-text")
                    .with_child(text)
                    .into(),
            );
        } else {
            class.push("pwt-meter-small")
        }

        if self.animated {
            class.push("pwt-animated");
        }

        children.push(
            Container::new()
                .class("pwt-meter-bar")
                .class(format!("pwt-meter-distance-{}", distance_to_optimum))
                .width(CssLength::Fraction(percentage))
                .into(),
        );

        self.std_props.into_vtag(
            Cow::Borrowed("div"),
            node_ref,
            Some(class),
            None,
            Some(children),
        )
    }
}
