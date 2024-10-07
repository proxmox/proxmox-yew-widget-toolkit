use crate::prelude::*;

use crate::props::{EventSubscriber, WidgetBuilder};
use crate::widget::Container;

use pwt_macros::widget;

/// Represents a Field Label.
#[widget(pwt=crate, comp=PwtFieldLabel, @element)]
#[derive(Clone, PartialEq, Properties)]
pub struct FieldLabel {
    pub label: AttrValue,
}

impl FieldLabel {
    pub fn new(label: impl Into<AttrValue>) -> Self {
        let label = label.into();
        yew::props! { Self {label} }
    }
}

pub struct PwtFieldLabel {}

impl Component for PwtFieldLabel {
    type Message = ();
    type Properties = FieldLabel;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        Container::from_tag("label")
            .with_std_props(&props.std_props)
            .listeners(&props.listeners)
            .with_child(props.label.clone())
            .into()
    }
}

impl From<&'static str> for FieldLabel {
    fn from(value: &'static str) -> Self {
        FieldLabel::new(value)
    }
}

impl From<AttrValue> for FieldLabel {
    fn from(value: AttrValue) -> Self {
        FieldLabel::new(value)
    }
}

impl From<String> for FieldLabel {
    fn from(value: String) -> Self {
        FieldLabel::new(value)
    }
}
