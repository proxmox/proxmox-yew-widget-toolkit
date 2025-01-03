use std::borrow::Cow;

use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::VTag;

use pwt_macros::{builder, widget};

use crate::props::{CssLength, WidgetBuilder, WidgetStyleBuilder};
use crate::widget::Container;

/// Wrapper for Html `<progress>`.
#[widget(pwt=crate, @element)]
#[derive(Default, Debug, Clone, PartialEq, Properties)]
#[builder]
pub struct Progress {
    /// Maximum value (default 1)
    ///
    /// This attribute describes how much work the task indicated by
    /// the progress element requires. The max attribute, if present,
    /// must have a value greater than 0.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub max: Option<f32>,

    /// Current value.
    ///
    /// Specifies how much of the task that has been completed. It
    /// must be a value between 0 and max, or between 0 and 1 if max
    /// is omitted. If there is no value attribute, the progress bar
    /// is indeterminate. This indicates that an activity is ongoing
    /// with no indication of how long it is expected to take.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub value: Option<f32>,
}

impl Progress {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props! { Self {}}
    }
}

impl From<Progress> for VTag {
    fn from(val: Progress) -> Self {
        let max = val.max.unwrap_or(1.0);

        let bar = match val.value {
            Some(value) => {
                let percentage = (value / max).clamp(0.0, 1.0);
                Container::new()
                    .class("pwt-progress-bar")
                    .width(CssLength::Fraction(percentage))
                    .into()
            }
            None => Container::new().class("pwt-progress-infinite").into(),
        };

        val.std_props.into_vtag(
            Cow::Borrowed("div"),
            Some("pwt-progress"),
            None,
            Some(vec![bar]),
        )
    }
}
