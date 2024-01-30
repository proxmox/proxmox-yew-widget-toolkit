use anyhow::Error;
use serde_json::Value;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;

use pwt_macros::{builder, widget};

use crate::props::{ContainerBuilder, EventSubscriber, WidgetBuilder};
use crate::widget::{Container, Tooltip};
use crate::tr;

use super::{
    IntoValidateFn, ManagedField, ManagedFieldContext, ManagedFieldMaster, ManagedFieldState,
    ValidateFn,
};

pub type PwtCheckbox = ManagedFieldMaster<CheckboxField>;

/// Checkbox input element.
#[widget(pwt=crate, comp=ManagedFieldMaster<CheckboxField>, @input, @element)]
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct Checkbox {
    /// Checkbox value (default is "on").
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub value: Option<AttrValue>,

    /// Force value (ignored by managed fields).
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub checked: Option<bool>,

    /// Default value.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub default: Option<bool>,

    /// Radio group flag
    #[prop_or_default]
    #[builder]
    pub radio_group: bool,

    /// Use switch style layout.
    #[prop_or_default]
    #[builder]
    pub switch: bool,

    /// The tooltip.
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub tip: Option<AttrValue>,

    /// Validation function.
    #[prop_or_default]
    pub validate: Option<ValidateFn<bool>>,

    /// Change callback.
    #[builder_cb(IntoEventCallback, into_event_callback, String)]
    #[prop_or_default]
    pub on_change: Option<Callback<String>>,

    /// Input callback.
    ///
    /// Called on user interaction:
    ///
    /// - Click on the checkbox.
    /// - Click on the associated input label.
    /// - Activation by keyboard (space press).
    #[builder_cb(IntoEventCallback, into_event_callback, String)]
    #[prop_or_default]
    pub on_input: Option<Callback<String>>,
}

impl Checkbox {
    /// Creates a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Creates a radio group member.
    pub fn radio() -> Self {
        yew::props!(Self { radio_group: true })
    }

    /// Builder style method to set the validate callback
    pub fn validate(mut self, validate: impl IntoValidateFn<bool>) -> Self {
        self.set_validate(validate);
        self
    }

    /// Method to set the validate callback
    pub fn set_validate(&mut self, validate: impl IntoValidateFn<bool>) {
        self.validate = validate.into_validate_fn();
    }
}

pub enum Msg {
    Toggle,
}

#[doc(hidden)]
pub struct CheckboxField {}

#[derive(PartialEq)]
pub struct ValidateClosure {
    validate: Option<ValidateFn<bool>>,
    on_value: AttrValue,
}

impl ManagedField for CheckboxField {
    type Message = Msg;
    type Properties = Checkbox;
    type ValidateClosure = ValidateClosure;

    fn validation_args(props: &Self::Properties) -> Self::ValidateClosure {
        ValidateClosure {
            validate: props.validate.clone(),
            on_value: props.value.clone().unwrap_or(AttrValue::Static("on")),
        }
    }

    fn validator(props: &Self::ValidateClosure, value: &Value) -> Result<Value, Error> {
        let value = match value {
            Value::String(value) => value == &props.on_value,
            _ => return Err(Error::msg(tr!("got wrong data type."))),
        };

        if let Some(validate) = &props.validate {
            validate.apply(&value)?;
        }

        Ok(if value {
            Value::String(props.on_value.to_string())
        } else {
            Value::String(String::new())
        })
    }

    fn setup(props: &Checkbox) -> ManagedFieldState {
        let on_value = props.value.as_deref().unwrap_or("on").to_string();

        let default = match props.default {
            Some(true) => on_value.clone(),
            _ => String::new(),
        };

        let value = match props.checked {
            Some(true) => on_value.clone(),
            Some(false) => String::new(),
            None => default.clone(),
        };

        ManagedFieldState {
            value: value.into(),
            valid: Ok(()),
            default: default.into(),
            radio_group: props.radio_group,
            unique: false,
        }
    }

    fn value_changed(&mut self, ctx: &super::ManagedFieldContext<Self>) {
        let props = ctx.props();
        let state = ctx.state();
        let value = state.value.as_str().unwrap_or("").to_string();
        if let Some(on_change) = &props.on_change {
            on_change.emit(value);
        }
    }

    fn create(_ctx: &ManagedFieldContext<Self>) -> Self {
        Self {}
    }

    fn label_clicked(&mut self, ctx: &ManagedFieldContext<Self>) -> bool {
        ctx.link().send_message(Msg::Toggle);
        false
    }

    fn update(&mut self, ctx: &ManagedFieldContext<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        let state = ctx.state();

        match msg {
            Msg::Toggle => {
                if props.input_props.disabled {
                    return true;
                }
                let on_value = props.value.as_deref().unwrap_or("on").to_string();
                let value = state.value.clone();
                let new_value = if value == on_value {
                    if props.radio_group {
                        // do not allow to deselect radio buttons (same behaviour as browser).
                        on_value
                    } else {
                        String::new()
                    }
                } else {
                    on_value
                };

                let changes = value != new_value;

                if changes {
                    ctx.link().update_value(new_value.clone());

                    if let Some(on_input) = &props.on_input {
                        on_input.emit(new_value);
                    }
                }
                true
            }
        }
    }

    fn changed(&mut self, ctx: &ManagedFieldContext<Self>, _old_props: &Self::Properties) -> bool {
        let props = ctx.props();

        if let Some(checked) = props.checked {
            let on_value = props.value.as_deref().unwrap_or("on").to_string();
            let value = if checked { on_value } else { String::new() };
            ctx.link().force_value(Some(value), None);
        }

        true
    }

    fn view(&self, ctx: &ManagedFieldContext<Self>) -> Html {
        let props = ctx.props();
        let state = ctx.state();

        let disabled = props.input_props.disabled;

        let on_value = props.value.as_deref().unwrap_or("on").to_string();
        let (value, valid) = (&state.value, &state.valid);
        let checked = *value == on_value;

        let onclick = ctx.link().callback(|_| Msg::Toggle);
        let onkeyup = Callback::from({
            let link = ctx.link().clone();
            move |event: KeyboardEvent| {
                if event.key() == " " {
                    link.send_message(Msg::Toggle);
                }
            }
        });

        let (layout_class, inner) = match (props.switch, props.radio_group) {
            (true, _) => (
                "pwt-switch",
                html! {<span class="pwt-switch-slider"><i class="fa fa-check"/></span>},
            ),
            (false, true) => {
                // Note: icon is invisible, but necessary for correct baseline alignment
                (
                    "pwt-radio-button",
                    html! {<span class="pwt-checkbox-icon"><i class="fa fa-check"/></span>},
                )
            }
            (false, false) => (
                "pwt-checkbox",
                html! {<span class="pwt-checkbox-icon"><i class="fa fa-check"/></span>},
            ),
        };

        // TODO: add other props.input_props

        let mut checkbox = Tooltip::new(inner)
            .with_std_props(&props.std_props)
            .listeners(&props.listeners)
            .class(layout_class)
            .class(checked.then(|| "checked"))
            .class(disabled.then(|| "disabled"))
            .class(if valid.is_ok() {
                "is-valid"
            } else {
                "is-invalid"
            })
            .attribute(
                "tabindex",
                props.input_props.tabindex.unwrap_or(0).to_string(),
            )
            .attribute("role", "checkbox")
            .attribute("aria-checked", checked.then(|| "true"))
            .onkeyup(onkeyup)
            .onclick(onclick);

        if let Err(msg) = &valid {
            checkbox.set_tip(msg.clone())
        } else if let Some(tip) = &props.tip {
            if !disabled {
                checkbox.set_tip(tip.clone())
            }
        }

        if props.switch {
            checkbox.into()
        } else {
            Container::new()
                .class("pwt-checkbox-state")
                .with_child(checkbox)
                .into()
        }
    }

    fn rendered(&mut self, ctx: &ManagedFieldContext<Self>, first_render: bool) {
        if first_render {
            let props = ctx.props();
            if props.input_props.autofocus {
                if let Some(el) = props.std_props.node_ref.cast::<web_sys::HtmlElement>() {
                    let _ = el.focus();
                }
            }
        }
    }
}
