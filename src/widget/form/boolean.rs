use anyhow::Error;
use serde_json::Value;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;

use pwt_macros::{builder, widget};

use super::{
    IntoValidateFn, ManagedField, ManagedFieldContext, ManagedFieldMaster, ManagedFieldState,
    ValidateFn,
};
use crate::props::{ContainerBuilder, EventSubscriber, WidgetBuilder};
use crate::tr;
use crate::widget::{Container, Tooltip};

pub type PwtBoolean = ManagedFieldMaster<BooleanField>;

/// Checkbox input element, which stores values as boolean
#[widget(pwt=crate, comp=ManagedFieldMaster<BooleanField>, @input, @element)]
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct Boolean {
    /// Force value (ignored by managed fields)
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub checked: Option<bool>,

    /// Default value.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub default: Option<bool>,

    /// Use switch style layout.
    #[prop_or_default]
    #[builder]
    pub switch: bool,

    /// The tooltip.
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub tip: Option<AttrValue>,

    /// Validation function.
    ///
    /// ```
    /// # use anyhow::bail;
    /// # use pwt::prelude::*;
    /// # use pwt::widget::form::Boolean;
    /// Boolean::new()
    ///   .submit(false)
    ///   .validate(|value: &bool| {
    ///      if !value {
    ///         bail!("Please accept the Terms Of Service")
    ///      }
    ///      Ok(())
    ///    });
    /// ```
    #[prop_or_default]
    pub validate: Option<ValidateFn<bool>>,

    /// Change callback
    #[builder_cb(IntoEventCallback, into_event_callback, bool)]
    #[prop_or_default]
    pub on_change: Option<Callback<bool>>,

    /// Input callback.
    ///
    /// Called on user interaction:
    ///
    /// - Click on the checkbox.
    /// - Click on the associated input label.
    /// - Activation by keyboard (space press).
    #[builder_cb(IntoEventCallback, into_event_callback, bool)]
    #[prop_or_default]
    pub on_input: Option<Callback<bool>>,
}

impl Boolean {
    /// Creates a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
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
pub struct BooleanField {}

#[derive(PartialEq)]
pub struct ValidateClosure {
    validate: Option<ValidateFn<bool>>,
}

impl ManagedField for BooleanField {
    type Properties = Boolean;
    type Message = Msg;
    type ValidateClosure = ValidateClosure;

    fn validation_args(props: &Self::Properties) -> Self::ValidateClosure {
        ValidateClosure {
            validate: props.validate.clone(),
        }
    }

    fn validator(props: &Self::ValidateClosure, value: &Value) -> Result<Value, Error> {
        let value = match value {
            Value::Bool(value) => *value,
            _ => return Err(Error::msg(tr!("got wrong data type."))),
        };

        if let Some(validate) = &props.validate {
            validate.apply(&value)?;
        }

        Ok(Value::Bool(value))
    }

    fn setup(props: &Boolean) -> ManagedFieldState {
        let mut value = false;
        if let Some(default) = &props.default {
            value = *default;
        }
        if let Some(checked) = &props.checked {
            value = *checked;
        }

        let valid = Ok(());

        let default = props.default.unwrap_or(false).into();

        ManagedFieldState {
            value: value.into(),
            valid,
            default,
            radio_group: false,
            unique: false,
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
                let checked = state.value.as_bool().unwrap_or(false);
                let new_checked = !checked;
                ctx.link().update_value(new_checked);

                if let Some(on_input) = &props.on_input {
                    on_input.emit(new_checked);
                }

                false
            }
        }
    }

    fn value_changed(&mut self, ctx: &ManagedFieldContext<Self>) {
        let props = ctx.props();
        let state = ctx.state();
        let checked = state.value.as_bool().unwrap_or(false);
        if let Some(on_change) = &props.on_change {
            on_change.emit(checked);
        }
    }

    fn changed(&mut self, ctx: &ManagedFieldContext<Self>, _old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        if let Some(checked) = props.checked {
            ctx.link().force_value(Some(checked), None)
        }
        true
    }

    fn view(&self, ctx: &ManagedFieldContext<Self>) -> Html {
        let props = ctx.props();
        let link = ctx.link();
        let disabled = props.input_props.disabled;

        let state = ctx.state();

        let (value, valid) = (&state.value, &state.valid);
        let checked = value.as_bool().unwrap_or(false);

        let onclick = link.callback(|_| Msg::Toggle);
        let onkeyup = Callback::from({
            let link = link.clone();
            move |event: KeyboardEvent| {
                if event.key() == " " {
                    link.send_message(Msg::Toggle);
                }
            }
        });

        let (layout_class, inner) = match props.switch {
            true => (
                "pwt-switch",
                html! {<span class="pwt-switch-slider"><i class="fa fa-check"/></span>},
            ),
            false => (
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
