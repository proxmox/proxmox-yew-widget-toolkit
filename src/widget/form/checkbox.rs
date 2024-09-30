use anyhow::Error;
use serde_json::Value;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;

use pwt_macros::{builder, widget};

use super::{
    IntoValidateFn, ManagedField, ManagedFieldContext, ManagedFieldMaster, ManagedFieldState,
    ValidateFn,
};
use crate::css::AlignItems;
use crate::props::{ContainerBuilder, CssPaddingBuilder, EventSubscriber, WidgetBuilder};
use crate::tr;
use crate::widget::{Container, Fa, FieldLabel, Row, Tooltip};

pub type PwtCheckbox = ManagedFieldMaster<CheckboxField>;

/// Checkbox input element, which stores values as boolean
#[widget(pwt=crate, comp=ManagedFieldMaster<CheckboxField>, @input, @element)]
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct Checkbox {
    /// The value that will be submitted when the field is checked/on.
    ///
    /// Note: must differ from the `submit_off_value` to be working correctly.
    #[builder(Into, into)]
    #[prop_or(Value::Bool(true))]
    pub submit_on_value: Value,

    /// The value that will be submitted when the field is not checked/off.
    ///
    /// Note: must differ from the `submit_on_value` to be working correctly.
    #[builder(Into, into)]
    #[prop_or(Value::Bool(false))]
    pub submit_off_value: Value,

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
    /// # use pwt::widget::form::Checkbox;
    /// Checkbox::new()
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

    /// A right side label for the checkbox, to display additional information
    #[prop_or_default]
    pub box_label: Option<FieldLabel>,
}

impl Checkbox {
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
pub struct CheckboxField {}

#[derive(PartialEq)]
pub struct ValidateClosure {
    validate: Option<ValidateFn<bool>>,
    submit_on_value: Value,
    submit_off_value: Value,
}

impl ManagedField for CheckboxField {
    type Properties = Checkbox;
    type Message = Msg;
    type ValidateClosure = ValidateClosure;

    fn validation_args(props: &Self::Properties) -> Self::ValidateClosure {
        ValidateClosure {
            validate: props.validate.clone(),
            submit_on_value: props.submit_on_value.clone(),
            submit_off_value: props.submit_off_value.clone(),
        }
    }

    fn validator(props: &Self::ValidateClosure, value: &Value) -> Result<Value, Error> {
        let value = match value {
            Value::Bool(value) => *value,
            other => {
                if *other == props.submit_on_value {
                    true
                } else if *other == props.submit_off_value {
                    false
                } else {
                    return Err(Error::msg(tr!("got invalid value.")));
                }
            }
        };

        if let Some(validate) = &props.validate {
            validate.apply(&value)?;
        }

        Ok(Value::Bool(value))
    }

    fn setup(props: &Checkbox) -> ManagedFieldState {
        let mut value = Value::Bool(false);
        if let Some(default) = &props.default {
            value = if *default {
                props.submit_on_value.clone()
            } else {
                props.submit_off_value.clone()
            };
        }
        if let Some(checked) = &props.checked {
            value = if *checked {
                props.submit_on_value.clone()
            } else {
                props.submit_off_value.clone()
            };
        }

        let valid = Ok(());

        let default = if props.default.unwrap_or(false) {
            props.submit_on_value.clone()
        } else {
            props.submit_off_value.clone()
        };

        ManagedFieldState {
            value,
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
                let checked = state.value == props.submit_on_value;
                let new_value = if checked {
                    props.submit_off_value.clone()
                } else {
                    props.submit_on_value.clone()
                };
                ctx.link().update_value(new_value);

                if let Some(on_input) = &props.on_input {
                    on_input.emit(!checked);
                }

                false
            }
        }
    }

    fn value_changed(&mut self, ctx: &ManagedFieldContext<Self>) {
        let props = ctx.props();
        let state = ctx.state();
        let checked = state.value == props.submit_on_value;
        if let Some(on_change) = &props.on_change {
            on_change.emit(checked);
        }
    }

    fn changed(&mut self, ctx: &ManagedFieldContext<Self>, _old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        if let Some(checked) = props.checked {
            let value = if checked {
                props.submit_on_value.clone()
            } else {
                props.submit_off_value.clone()
            };
            ctx.link().force_value(Some(value), None)
        }
        true
    }

    fn view(&self, ctx: &ManagedFieldContext<Self>) -> Html {
        let props = ctx.props();
        let link = ctx.link();
        let disabled = props.input_props.disabled;

        let state = ctx.state();

        let (value, valid) = (&state.value, &state.valid);
        let checked = {
            let this = &props;
            *value == this.submit_on_value
        };

        let onclick = link.callback(|_| Msg::Toggle);
        let onkeyup = Callback::from({
            let link = link.clone();
            move |event: KeyboardEvent| {
                if event.key() == " " {
                    link.send_message(Msg::Toggle);
                }
            }
        });

        let (layout_class, inner_class) = match props.switch {
            true => ("pwt-switch", "pwt-switch-slider"),
            false => ("pwt-checkbox", "pwt-checkbox-icon"),
        };

        let checkbox = Container::new()
            .class((!props.switch).then_some("pwt-checkbox-state"))
            .with_child(
                Container::new()
                    .class(layout_class)
                    .with_child(
                        Container::from_tag("span")
                            .class(inner_class)
                            .with_child(Fa::new("check")),
                    )
                    .class(checked.then_some("checked"))
                    .class(disabled.then_some("disabled"))
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
                    .attribute("aria-checked", checked.then_some("true"))
                    .onkeyup(onkeyup),
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

        if let Err(msg) = &valid {
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
                if let Some(el) = props.std_props.node_ref.cast::<web_sys::HtmlElement>() {
                    let _ = el.focus();
                }
            }
        }
    }
}
