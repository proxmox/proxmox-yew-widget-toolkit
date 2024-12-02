use yew::{AttrValue, Properties};

use pwt_macros::{builder, widget};

use crate::{
    touch::prelude::{ContainerBuilder, WidgetBuilder},
    widget::{
        form::{ManagedField, ManagedFieldMaster, ManagedFieldState},
        Container, Tooltip,
    },
};

pub type PwtDisplayField = ManagedFieldMaster<DisplayFieldImpl>;

#[widget(pwt=crate, comp=ManagedFieldMaster<DisplayFieldImpl>, @input, @element)]
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct DisplayField {
    /// The value to display.
    pub value: AttrValue,

    /// The tooltip to display.
    #[prop_or_default]
    #[builder]
    pub tip: Option<AttrValue>,
}

impl DisplayField {
    pub fn new(value: impl Into<AttrValue>) -> Self {
        yew::props!(Self {
            value: value.into(),
        })
    }
}

#[doc(hidden)]
pub struct DisplayFieldImpl {}

impl ManagedField for DisplayFieldImpl {
    type Properties = DisplayField;
    type Message = ();
    type ValidateClosure = ();

    fn validation_args(_props: &Self::Properties) -> Self::ValidateClosure {}

    fn setup(props: &Self::Properties) -> super::ManagedFieldState {
        ManagedFieldState {
            value: props.value.to_string().into(),
            valid: Ok(()),
            default: props.value.to_string().into(),
            radio_group: false,
            unique: false,
        }
    }

    fn create(_ctx: &super::ManagedFieldContext<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &super::ManagedFieldContext<Self>) -> yew::Html {
        let props = ctx.props();
        let state = ctx.state();
        let value = state.value.as_str().unwrap_or(&props.value);
        Tooltip::new(Container::from_tag("span").with_child(value))
            .with_std_props(&props.std_props)
            .class("pwt-input-display")
            .tip(&props.tip)
            .attribute("tabindex", "0")
            .into()
    }
}
