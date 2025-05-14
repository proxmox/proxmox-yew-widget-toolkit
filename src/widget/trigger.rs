use yew::{
    html::{IntoEventCallback, IntoPropValue},
    AttrValue, Callback, Component, Properties,
};

use pwt_macros::{builder, widget};

use crate::touch::prelude::{EventSubscriber, WidgetBuilder};

use super::{Fa, Tooltip};

/// This represents a Trigger for a field, like the icon to toggle a dropdown,
/// show/hide password, etc.
///
/// There are convenience From<> traits implemented for [AttrValue], `&'static str`
/// and [crate::widget::Fa], so one can simply use the icon classes in place
/// of creating a trigger manually.
#[widget(pwt=crate, comp=PwtTrigger, @element)]
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct Trigger {
    /// The callback when the trigger is clicked
    #[prop_or_default]
    #[builder_cb(IntoEventCallback, into_event_callback, yew::MouseEvent)]
    pub on_activate: Option<Callback<yew::MouseEvent>>,

    /// An optional tooltip
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub tip: Option<AttrValue>,
}

impl Trigger {
    pub fn new(icon: impl IntoPropValue<AttrValue>) -> Self {
        yew::props!(Self {})
            .class(icon.into_prop_value())
            .tabindex(-1)
    }
}

pub struct PwtTrigger;

impl Component for PwtTrigger {
    type Message = ();
    type Properties = Trigger;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let mut props = ctx.props().clone();
        if let Some(on_activate) = props.on_activate.clone() {
            props.set_onclick(move |value: yew::MouseEvent| {
                on_activate.emit(value.clone());
                value.prevent_default();
                value.stop_propagation();
            });
        }

        let pointer_cls = props.on_activate.as_ref().map(|_| "pwt-pointer");

        let icon = props
            .std_props
            .into_vtag("i".into(), pointer_cls, Some(props.listeners), None);

        Tooltip::new(icon).tip(props.tip).into()
    }
}

// convenience traits

impl From<Fa> for Trigger {
    fn from(value: Fa) -> Self {
        let Fa {
            std_props,
            listeners,
        } = value;
        yew::props!(Self {
            std_props,
            listeners
        })
    }
}

impl From<&'static str> for Trigger {
    fn from(value: &'static str) -> Self {
        Trigger::new(value)
    }
}

impl From<AttrValue> for Trigger {
    fn from(value: AttrValue) -> Self {
        Trigger::new(value)
    }
}
