use std::borrow::Cow;

use yew::prelude::*;
use yew::html::IntoPropValue;
use yew::virtual_dom::VTag;

use pwt_macros::{widget, builder};

use crate::widget::Container;
use crate::props::{ContainerBuilder, WidgetBuilder};


/// Wrapper for Html `<meter>`.
#[widget(pwt=crate, @element)]
#[builder]
#[derive(Default, Debug, Clone, PartialEq, Properties)]
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
    pub low: Option<f32>,

    /// Define the high end range.
    #[builder(IntoPropValue, into_prop_value)]
    pub high: Option<f32>,

    /// Optimal value.
    ///
    /// This gives an indication where along the range is considered
    /// preferable. For example, if it is between the min attribute
    /// and the low attribute, then the lower range is considered
    /// preferred. The meter's bar color depends on whether the value
    /// is less than or equal to the optimum value.
    #[builder(IntoPropValue, into_prop_value)]
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

    /// Show percentage as text.
    #[prop_or(true)]
    pub show_text: bool,
}

impl Meter {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props! { Self {}}
    }

    /// Builder style method to set the `show_text` flag.
    pub fn show_text(mut self, value: bool) -> Self {
        self.set_show_text(value);
        self
    }

    /// Method to set the `show_text` flag.
    pub fn set_show_text(&mut self, value: bool) {
        self.show_text = value;
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


impl Into<VTag> for Meter {
    fn into(self) -> VTag {
        let percentage = (((self.value - self.min).max(0.0) / (self.max - self.min)) * 100.0).min(100.0).max(0.0);

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

        if self.show_text {
            children.push(
                Container::new()
                    .class("pwt-meter-text")
                    .with_child(format!("{}", self.value))
                    .into()
            );
        }

        children.push(
            Container::new()
                .class("pwt-meter-bar")
                .class(format!("pwt-meter-distance-{}", distance_to_optimum))
                .attribute("style", format!("width:{percentage}%"))
                .into()
        );


        self.std_props.into_vtag(Cow::Borrowed("div"), Some("pwt-meter"), None, Some(children))
    }
}
