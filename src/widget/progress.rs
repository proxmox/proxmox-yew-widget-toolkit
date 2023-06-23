use std::borrow::Cow;

use yew::prelude::*;
use yew::html::IntoPropValue;
use yew::virtual_dom::{VList, VTag};

use pwt_macros::{builder, widget};
use yew::virtual_dom::Listeners;

use crate::props::WidgetBuilder;
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
    pub max: Option<f32>,

    /// Current value.
    ///
    /// Specifies how much of the task that has been completed. It
    /// must be a value between 0 and max, or between 0 and 1 if max
    /// is omitted. If there is no value attribute, the progress bar
    /// is indeterminate. This indicates that an activity is ongoing
    /// with no indication of how long it is expected to take.
    #[builder(IntoPropValue, into_prop_value)]
    pub value: Option<f32>,
}

impl Progress {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!{ Self {}}
    }
}

impl Into<VTag> for Progress {
    fn into(self) -> VTag {
        let attributes = self.std_props.cumulate_attributes(Some("pwt-progress"));

        let max = self.max.unwrap_or(1.0);

        let bar = match self.value {
            Some(value) => {
                let percentage = ((value / max) * 100.0).min(100.0).max(0.0);
                Container::new()
                    .class("pwt-progress-bar")
                    .attribute("style", format!("width:{percentage}%"))
                    .into()
            }
            None => {
                Container::new()
                    .class("pwt-progress-infinite")
                    .into()
            }
        };

        let children = VList::with_children(vec![bar], None);

        VTag::__new_other(
            Cow::Borrowed("div"),
            self.std_props.node_ref,
            self.std_props.key,
            attributes,
            Listeners::None,
            children,
        )
    }
}
