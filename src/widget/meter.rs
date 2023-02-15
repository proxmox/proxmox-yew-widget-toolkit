use yew::prelude::*;
use yew::html::IntoPropValue;

use pwt_macros::widget;

use crate::widget::Container;
use crate::props::{ContainerBuilder, WidgetBuilder};


/// Wrapper for Html `<meter>`.
#[widget(pwt=crate, comp=PwtMeter, @element)]
#[derive(Default, Debug, Clone, PartialEq, Properties)]
pub struct Meter {
    /// Minimum value (default 0)
    ///
    /// Lower numeric bound. This must be less than the maximum value.
    #[prop_or(0.0)]
    pub min: f32,

    /// Maximum value (default 1)
    ///
    /// Upper numeric bound. This must be greater than the minimum
    /// value.
    #[prop_or(1.0)]
    pub max: f32,

    /// Define the low end range.
    pub low: Option<f32>,

    /// Define the high end range.
    pub high: Option<f32>,

    /// Optimal value.
    ///
    /// This gives an indication where along the range is considered
    /// preferable. For example, if it is between the min attribute
    /// and the low attribute, then the lower range is considered
    /// preferred. The meter's bar color depends on whether the value
    /// is less than or equal to the optimum value.
    pub optimum: Option<f32>,

    /// Current value (default 0).
    ///
    /// This must be between the minimum and maximum values. If
    /// specified, but not within the range given by the min attribute
    /// and max attribute, the value is equal to the nearest end of
    /// the range.
    #[prop_or(0.0)]
    pub value: f32,

    /// Show percentage as text.
    #[prop_or(true)]
    pub show_text: bool,
}

impl Meter {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!{ Self {}}
    }

    /// Builder style method to set minimum value.
    pub fn min(mut self, value: impl IntoPropValue<Option<f32>>) -> Self {
        self.set_min(value);
        self
    }

    /// Method to set minimum value.
    pub fn set_min(&mut self, value: impl IntoPropValue<Option<f32>>) {
        self.min = value.into_prop_value().unwrap_or(1.0);
    }

    /// Builder style method to set maximum value.
    pub fn max(mut self, value: impl IntoPropValue<Option<f32>>) -> Self {
        self.set_max(value);
        self
    }

    /// Method to set maximum value.
    pub fn set_max(&mut self, value: impl IntoPropValue<Option<f32>>) {
        self.max = value.into_prop_value().unwrap_or(1.0);
    }

    /// Builder style method to set the optimal value.
    pub fn optimum(mut self, value: impl IntoPropValue<Option<f32>>) -> Self {
        self.set_optimum(value);
        self
    }

    /// Method to set the optimal value.
    pub fn set_optimum(&mut self, value: impl IntoPropValue<Option<f32>>) {
        self.optimum = value.into_prop_value();
    }

    /// Builder style method to set the low value.
    pub fn low(mut self, value: impl IntoPropValue<Option<f32>>) -> Self {
        self.set_low(value);
        self
    }

    /// Method to set the low value.
    pub fn set_low(&mut self, value: impl IntoPropValue<Option<f32>>) {
        self.low = value.into_prop_value();
    }

    /// Builder style method to set the high value.
    pub fn high(mut self, value: impl IntoPropValue<Option<f32>>) -> Self {
        self.set_high(value);
        self
    }

    /// Method to set the high value.
    pub fn set_high(&mut self, value: impl IntoPropValue<Option<f32>>) {
        self.high = value.into_prop_value();
    }

    /// Builder style method to set the current value.
    pub fn value(mut self, value: impl IntoPropValue<Option<f32>>) -> Self {
        self.set_value(value);
        self
    }

    /// Method to set the current value.
    pub fn set_value(&mut self, value: impl IntoPropValue<Option<f32>>) {
        self.value = value.into_prop_value().unwrap_or(0.0);
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

#[doc(hidden)]
pub struct PwtMeter {}

impl Component for PwtMeter {
    type Message = ();
    type Properties = Meter;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let mut meter = Container::new()
            .class("pwt-meter");

        let percentage = (((props.value - props.min).max(0.0) / (props.max - props.min)) * 100.0).min(100.0).max(0.0);

        let distance_to_optimum = if let Some(optimum) = props.optimum {
            if optimum > props.value {
                props.get_range_index(optimum) - props.get_range_index(props.value)
            } else {
                props.get_range_index(props.value) - props.get_range_index(optimum)
            }
        } else {
            0
        };


        if props.show_text {
            meter.add_child(
                Container::new()
                    .class("pwt-meter-text")
                    .with_child(format!("{}", props.value))
            );
        }

        meter.add_child(
            Container::new()
                .class("pwt-meter-bar")
                .class(format!("pwt-meter-distance-{}", distance_to_optimum))
                .attribute("style", format!("width:{percentage}%"))
        );

        meter.into()
    }
}
