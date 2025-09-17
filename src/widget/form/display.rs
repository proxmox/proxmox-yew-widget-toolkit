use serde_json::Value;

use yew::html::IntoPropValue;
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

/// A display only text field which is not validated
#[widget(pwt=crate, comp=ManagedFieldMaster<DisplayFieldImpl>, @input, @element)]
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct DisplayField {
    /// Force value.
    ///
    /// To implement controlled components (for use without a FormContext).
    /// This is ignored if the field has a name.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub value: Option<AttrValue>,

    /// Default value.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub default: Option<AttrValue>,

    /// The tooltip to display.
    #[prop_or_default]
    #[builder]
    pub tip: Option<AttrValue>,
}

impl DisplayField {
    pub fn new() -> Self {
        yew::props!(Self {})
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
        let value: Value = match &props.value {
            Some(value) => value.to_string().into(),
            None => Value::Null,
        };

        let default: Value = props.default.as_deref().unwrap_or("").into();
        let result = Ok(default.clone());
        let last_valid = result.clone().ok();

        ManagedFieldState {
            value,
            result,
            last_valid,
            default,
            radio_group: false,
            unique: false,
        }
    }

    fn create(_ctx: &super::ManagedFieldContext<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &super::ManagedFieldContext<Self>) -> yew::Html {
        let props = ctx.props();
        let input_props = &props.input_props;

        let state = ctx.state();
        let value = state
            .value
            .as_str()
            .unwrap_or(props.default.as_deref().unwrap_or(""));

        let tabindex = input_props.tabindex.unwrap_or(0).to_string();

        Tooltip::new(Container::from_tag("span").with_child(value))
            .with_std_props(&props.std_props)
            .class("pwt-input-display")
            .tip(&props.tip)
            .attribute("tabindex", tabindex)
            .attribute("aria-label", input_props.aria_label.clone())
            .attribute("aria-labelledby", input_props.label_id.clone())
            .into()
    }
}
