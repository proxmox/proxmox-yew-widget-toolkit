use anyhow::Error;
use serde_json::Value;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;

use pwt_macros::{builder, widget};

use crate::css::AlignItems;
use crate::props::{ContainerBuilder, CssPaddingBuilder, EventSubscriber, IntoVTag, WidgetBuilder};
use crate::tr;
use crate::widget::{Container, Fa, FieldLabel, Row, Tooltip};

use super::{
    IntoValidateFn, ManagedField, ManagedFieldContext, ManagedFieldMaster, ManagedFieldState,
    ValidateFn,
};

pub type PwtRadioButton = ManagedFieldMaster<RadioButtonField>;

/// RadioButton input element.
#[widget(pwt=crate, comp=ManagedFieldMaster<RadioButtonField>, @input, @element)]
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct RadioButton {
    /// RadioButton value.
    #[builder(IntoPropValue, into_prop_value)]
    pub value: AttrValue,

    /// Force value (ignored by managed fields).
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub checked: Option<bool>,

    /// Default value.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub default: Option<bool>,

    /// The tooltip.
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub tip: Option<AttrValue>,

    /// Validation function.
    ///
    /// ```
    /// # use anyhow::bail;
    /// # use pwt::prelude::*;
    /// # use pwt::widget::form::RadioButton;
    /// RadioButton::new("value")
    ///   .submit(false)
    ///   .validate(|value: &bool| {
    ///      if !value {
    ///         bail!("Please select 'value'")
    ///      }
    ///      Ok(())
    ///    });
    /// ```
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

    /// A right side label for the checkbox, to display additional information
    #[prop_or_default]
    pub box_label: Option<FieldLabel>,
}

impl RadioButton {
    /// Creates a new instance.
    pub fn new(value: impl Into<AttrValue>) -> Self {
        yew::props!(Self {
            value: value.into()
        })
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

    /// Method to set the box label.
    ///
    /// A right side label for the checkbox to display additional information
    pub fn set_box_label(&mut self, box_label: impl Into<FieldLabel>) {
        self.box_label = Some(box_label.into());
    }

    /// Builder method to set the box label.
    ///
    ///A right side label for the checkbox to display additional information
    pub fn box_label(mut self, box_label: impl Into<FieldLabel>) -> Self {
        self.set_box_label(box_label);
        self
    }
}

pub enum Msg {
    Toggle,
}

#[doc(hidden)]
pub struct RadioButtonField {
    node_ref: NodeRef,
}

#[derive(PartialEq)]
pub struct ValidateClosure {
    validate: Option<ValidateFn<bool>>,
    on_value: AttrValue,
}

impl ManagedField for RadioButtonField {
    type Message = Msg;
    type Properties = RadioButton;
    type ValidateClosure = ValidateClosure;

    fn validation_args(props: &Self::Properties) -> Self::ValidateClosure {
        ValidateClosure {
            validate: props.validate.clone(),
            on_value: props.value.clone(),
        }
    }

    fn validator(props: &Self::ValidateClosure, value: &Value) -> Result<Value, Error> {
        let value = match value {
            Value::String(value) => value == props.on_value,
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

    fn setup(props: &RadioButton) -> ManagedFieldState {
        let on_value = props.value.to_string();

        let default = match props.default {
            Some(true) => on_value.clone(),
            _ => String::new(),
        };

        let value = match props.checked {
            Some(true) => on_value.clone(),
            Some(false) => String::new(),
            None => default.clone(),
        };

        let mut state = ManagedFieldState::new(value.into(), default.into());
        state.radio_group = true;
        state
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
        Self {
            node_ref: NodeRef::default(),
        }
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
                let on_value = props.value.to_string();
                let value = state.value.clone();

                let changes = value != on_value;

                if changes {
                    ctx.link().update_value(on_value.clone());

                    if let Some(on_input) = &props.on_input {
                        on_input.emit(on_value);
                    }
                }
                true
            }
        }
    }

    fn changed(&mut self, ctx: &ManagedFieldContext<Self>, _old_props: &Self::Properties) -> bool {
        let props = ctx.props();

        if let Some(checked) = props.checked {
            let on_value = props.value.to_string();
            let value = if checked { on_value } else { String::new() };
            ctx.link().force_value(Some(value), None);
        }

        true
    }

    fn view(&self, ctx: &ManagedFieldContext<Self>) -> Html {
        let props = ctx.props();
        let state = ctx.state();

        let disabled = props.input_props.disabled;

        let on_value = props.value.to_string();
        let (value, validation_result) = (&state.value, &state.result);
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

        let checkbox = Container::new().class("pwt-checkbox-state").with_child(
            Container::new()
                .class("pwt-radio-button")
                .with_child(
                    // invisible but necessary for correct baseline alignment
                    Container::from_tag("span")
                        .class("pwt-checkbox-icon")
                        .with_child(Fa::new("check")),
                )
                .class(checked.then_some("checked"))
                .class(disabled.then_some("disabled"))
                .class(if validation_result.is_ok() {
                    "is-valid"
                } else {
                    "is-invalid"
                })
                .attribute(
                    "tabindex",
                    props.input_props.tabindex.unwrap_or(0).to_string(),
                )
                .attribute("role", "checkbox")
                .attribute("aria-checked", checked.then_some("true"))
                .onkeyup(onkeyup)
                .into_html_with_ref(self.node_ref.clone()),
        );

        let box_label = props.box_label.clone().map(|label| label.padding_start(2));

        let checkbox = Row::new()
            .class(AlignItems::Center)
            .with_child(checkbox)
            .with_optional_child(box_label);

        // TODO: add other props.input_props

        let mut checkbox = Tooltip::new(checkbox)
            .with_std_props(&props.std_props)
            .listeners(&props.listeners)
            .onclick(onclick);

        if let Err(msg) = &validation_result {
            checkbox.set_tip(msg.clone())
        } else if let Some(tip) = &props.tip {
            if !disabled {
                checkbox.set_tip(tip.clone())
            }
        }

        checkbox.into()
    }

    fn rendered(&mut self, ctx: &ManagedFieldContext<Self>, first_render: bool) {
        if first_render {
            let props = ctx.props();
            if props.input_props.autofocus {
                if let Some(el) = self.node_ref.cast::<web_sys::HtmlElement>() {
                    let _ = el.focus();
                }
            }
        }
    }
}
