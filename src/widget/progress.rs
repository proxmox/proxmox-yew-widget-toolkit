use yew::prelude::*;
use yew::html::IntoPropValue;

use pwt_macros::widget;

use crate::widget::Container;
use crate::props::{ContainerBuilder, WidgetBuilder};


/// Wrapper for Html `<progress>`.
#[widget(pwt=crate, comp=PwtProgress, @element)]
#[derive(Default, Debug, Clone, PartialEq, Properties)]
pub struct Progress {
    /// Maximum value (default 1)
    ///
    /// This attribute describes how much work the task indicated by
    /// the progress element requires. The max attribute, if present,
    /// must have a value greater than 0.
    pub max: Option<f32>,

    /// Current value.
    ///
    /// Specifies how much of the task that has been completed. It
    /// must be a value between 0 and max, or between 0 and 1 if max
    /// is omitted. If there is no value attribute, the progress bar
    /// is indeterminate. This indicates that an activity is ongoing
    /// with no indication of how long it is expected to take.
    pub value: Option<f32>,

    /// Show percentage as text.
    #[prop_or(true)]
    pub show_text: bool,
}

impl Progress {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!{ Self {}}
    }

    /// Builder style method to set maximum value.
    pub fn max(mut self, value: impl IntoPropValue<Option<f32>>) -> Self {
        self.set_max(value);
        self
    }

    /// Method to set maximum value.
    pub fn set_max(&mut self, value: impl IntoPropValue<Option<f32>>) {
        self.max = value.into_prop_value();
    }

    /// Builder style method to set the current value.
    pub fn value(mut self, value: impl IntoPropValue<Option<f32>>) -> Self {
        self.set_value(value);
        self
    }

    /// Method to set the current value.
    pub fn set_value(&mut self, value: impl IntoPropValue<Option<f32>>) {
        self.value = value.into_prop_value();
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
}

#[doc(hidden)]
pub struct PwtProgress {}

impl Component for PwtProgress {
    type Message = ();
    type Properties = Progress;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let max = props.max.unwrap_or(1.0);


        let mut progress = Container::new()
            .with_std_props(&props.std_props)
            .class("pwt-progress");

        if let Some(value) = props.value {
            let percentage = ((value / max) * 100.0).min(100.0).max(0.0);

            if props.show_text {
                progress.add_child(
                    Container::new()
                        .class("pwt-progress-text")
                        .with_child(format!("{:.0}%", percentage))
                );
            }

            progress.add_child(
                Container::new()
                    .class("pwt-progress-bar")
                    .attribute("style", format!("width:{percentage}%"))
            );
        } else {
            progress.add_child(
                Container::new()
                    .class("pwt-progress-infinite")
            );
        }

        progress.into()
    }
}
