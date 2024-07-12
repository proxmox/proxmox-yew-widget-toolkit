use yew::{
    html::{IntoEventCallback, IntoPropValue},
    virtual_dom::Key,
    AttrValue, Callback, Component, Properties,
};

use pwt_macros::{builder, widget};

use crate::touch::prelude::{EventSubscriber, WidgetBuilder, WidgetStyleBuilder};

use super::{Container, Fa, Tooltip};

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
    #[prop_or_default]
    pub key: Option<Key>,

    /// The default icon to show
    pub icon: AttrValue,

    /// The callback when the trigger is clicked
    #[prop_or_default]
    #[builder_cb(IntoEventCallback, into_event_callback, yew::MouseEvent)]
    pub onclick: Option<Callback<yew::MouseEvent>>,

    /// An optional tooltip
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub tip: Option<AttrValue>,
}

impl Trigger {
    pub fn new(icon: impl IntoPropValue<AttrValue>) -> Self {
        yew::props!(Self {
            icon: icon.into_prop_value()
        })
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
        let props = ctx.props();
        let pointer_cls = props.onclick.as_ref().map(|_| "pwt-pointer");
        let icon = Container::from_tag("i")
            .with_std_props(&props.std_props)
            .tabindex(-1)
            .class(&props.icon)
            .class(pointer_cls)
            .onclick({
                let onclick = props.onclick.clone().into_event_callback();
                onclick.map(|onclick| {
                    move |value: yew::MouseEvent| {
                        onclick.emit(value.clone());
                        value.prevent_default();
                        value.stop_propagation();
                    }
                })
            });

        Tooltip::new(icon).tip(&props.tip).into()
    }
}

// convenience traits

impl From<Fa> for Trigger {
    fn from(value: Fa) -> Self {
        Trigger::new("").class(value.class).styles(value.style)
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
