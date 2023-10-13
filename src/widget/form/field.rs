use anyhow::Error;
use serde_json::Value;

use web_sys::HtmlInputElement;
use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;

use pwt_macros::{builder, widget};

use super::{
    IntoValidateFn, ManagedField, ManagedFieldContext, ManagedFieldMaster, ManagedFieldState,
    ValidateFn,
};
use crate::props::{ContainerBuilder, EventSubscriber, WidgetBuilder};
use crate::widget::{Container, Input, Tooltip};

use crate::tr;

pub type PwtField = ManagedFieldMaster<StandardField>;

/// Checkbox input element, which stores values as boolean
#[widget(pwt=crate, comp=ManagedFieldMaster<StandardField>, @input, @element)]
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct Field {
    /// Input type (html input element attribute).
    #[prop_or(AttrValue::Static("text"))]
    #[builder(IntoPropValue, into_prop_value)]
    pub input_type: AttrValue,

    /// Minimum value for number fields.
    #[prop_or_default]
    pub min: Option<f64>,
    /// Maximum value for number fields.
    #[prop_or_default]
    pub max: Option<f64>,
    /// Step value for number fields.
    #[prop_or_default]
    pub step: Option<f64>,

    /// Force value.
    ///
    /// To implement controlled components (for use without a FormContext).
    /// This is ignored if the field has a name.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub value: Option<AttrValue>,

    /// Force validation result.
    ///
    /// To implement controlled components (for use without a FormContext).
    /// This is ignored if the field has a name.
    ///
    /// This is only used if you also force a value, and overwrites
    /// any result from the validation function (if any).
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub valid: Option<Result<(), String>>,

    /// Default value.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub default: Option<AttrValue>,

    /// Validation function.
    ///
    /// # Note
    ///
    /// It is currently not allowed to access the
    /// [FormContext](super::FormContext) inside a validation
    /// callback! If you need such functionality, do validation inside
    /// [FormContext::on_change](super::FormContext::on_change),
    /// then set the result with
    /// `form_ctx.write().set_field_valid(...)`.
    #[prop_or_default]
    pub validate: Option<ValidateFn<String>>,

    /// Change callback
    ///
    /// This callback is emited on any data change, i.e. if data
    /// inside the [FormContext](super::FormContext) changed.
    #[builder_cb(IntoEventCallback, into_event_callback, String)]
    #[prop_or_default]
    pub on_change: Option<Callback<String>>,

    /// Input callback
    ///
    /// This callback is emited when the user types in new data.
    #[builder_cb(IntoEventCallback, into_event_callback, String)]
    #[prop_or_default]
    pub on_input: Option<Callback<String>>,

    /// Show peek icon
    ///
    /// Whether to show a peek icon to reveal passwords. This won't have any
    /// effect if the input type of the field is not `password`.
    #[prop_or(true)]
    #[builder]
    pub show_peek_icon: bool,

    /// Icons to show on the left (false) or right(true) side of the input
    #[prop_or_default]
    #[builder]
    pub icons: Vec<(AttrValue, bool)>,

    /// The tooltip.
    #[prop_or_default]
    pub tip: Option<AttrValue>,
}

impl Field {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Create a new number field.
    pub fn number(
        mut self,
        min: impl IntoPropValue<Option<f64>>,
        max: impl IntoPropValue<Option<f64>>,
        step: impl IntoPropValue<Option<f64>>,
    ) -> Self {
        self.min = min.into_prop_value();
        self.max = max.into_prop_value();
        self.step = step.into_prop_value();
        self.input_type = AttrValue::Static("number");
        self
    }

    /// Builder style method to set the validate callback
    pub fn validate(mut self, validate: impl IntoValidateFn<String>) -> Self {
        self.set_validate(validate);
        self
    }

    /// Method to set the validate callback
    pub fn set_validate(&mut self, validate: impl IntoValidateFn<String>) {
        self.validate = validate.into_validate_fn();
    }

    /// Builder style method to add an icon
    pub fn with_icon(mut self, icon: impl IntoPropValue<AttrValue>, right: bool) -> Self {
        self.add_icon(icon, right);
        self
    }

    /// Method to add an icon
    pub fn add_icon(&mut self, icon: impl IntoPropValue<AttrValue>, right: bool) {
        self.icons.push((icon.into_prop_value(), right));
    }

    /// Builder style method to set the tooltip
    pub fn tip(mut self, tip: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_tip(tip);
        self
    }

    /// Method to set the tooltip
    pub fn set_tip(&mut self, tip: impl IntoPropValue<Option<AttrValue>>) {
        self.tip = tip.into_prop_value();
    }
}

pub enum Msg {
    Update(String),
    RevealPassword,
    HidePassword,
}

#[derive(Clone, Copy, PartialEq)]
enum PasswordState {
    NotAPassword,
    Revealed,
    Hidden,
}

#[doc(hidden)]
pub struct StandardField {
    password_state: PasswordState,
    input_ref: NodeRef,
}

// Field are type Value::String(), but we also allow Value::Number ..
fn value_to_text(value: &Value) -> String {
    match value {
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.to_string(),
        _ => String::new(),
    }
}

#[derive(PartialEq)]
pub struct ValidateClosure {
    required: bool,
    input_type: AttrValue,
    min: Option<f64>,
    max: Option<f64>,
    validate: Option<ValidateFn<String>>,
}

impl ManagedField for StandardField {
    type Properties = Field;
    type Message = Msg;
    type ValidateClosure = ValidateClosure;

    fn validation_args(props: &Self::Properties) -> Self::ValidateClosure {
        ValidateClosure {
            required: props.input_props.required,
            input_type: props.input_type.clone(),
            min: props.min,
            max: props.max,
            validate: props.validate.clone(),
        }
    }

    fn validator(props: &Self::ValidateClosure, value: &Value) -> Result<(), Error> {
        let value = match value {
            Value::Null => String::new(),
            Value::Number(n) => n.to_string(),
            Value::String(v) => v.clone(),
            _ => {
                // should not happen
                log::error!("PwtField: got wrong data type in validate!");
                String::new()
            }
        };

        if value.is_empty() {
            if props.required {
                return Err(Error::msg(tr!("Field may not be empty.")));
            } else {
                return Ok(());
            }
        }

        if props.input_type == "number" {
            let value_f64 = match value.parse::<f64>() {
                Ok(v) => v,
                Err(err) => return Err(Error::msg(tr!("unable to parse number: {}", &err))),
            };
            if let Some(min) = props.min {
                if value_f64 < min {
                    return Err(Error::msg(tr!(
                        "value must be greater than or equal to '{}'",
                        min
                    )));
                }
            }
            if let Some(max) = props.max {
                if value_f64 > max {
                    return Err(Error::msg(tr!(
                        "value must be less than or equal to '{}'",
                        max
                    )));
                }
            }
        }

        match &props.validate {
            Some(cb) => cb.validate(&value),
            None => Ok(()),
        }
    }

    fn setup(props: &Self::Properties) -> ManagedFieldState {
        let mut value = String::new();

        if let Some(default) = &props.default {
            value = default.to_string();
        }
        if let Some(force_value) = &props.value {
            value = force_value.to_string();
        }

        let value: Value = value.clone().into();

        let default = props.default.as_deref().unwrap_or("").into();

        ManagedFieldState {
            value,
            valid: Ok(()),
            default,
            radio_group: false,
            unique: false,
            submit_converter: None,
        }
    }

    fn value_changed(&mut self, ctx: &super::ManagedFieldContext<Self>) {
        let props = ctx.props();
        let state = ctx.state();
        let text_value = value_to_text(&state.value);
        if let Some(on_change) = &props.on_change {
            on_change.emit(text_value);
        }
    }

    fn changed(&mut self, ctx: &ManagedFieldContext<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        if props.value != old_props.value || props.valid != old_props.valid {
            if let Some(forced_value) = &props.value {
                ctx.link()
                    .force_value(forced_value.to_string(), props.valid.clone());
            }
        }
        true
    }

    fn create(ctx: &ManagedFieldContext<Self>) -> Self {
        let props = ctx.props();
        let password_state = if props.input_type == "password" {
            PasswordState::Hidden
        } else {
            PasswordState::NotAPassword
        };

        Self {
            password_state,
            input_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &ManagedFieldContext<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::Update(input) => {
                ctx.link().update_value(input.clone());
                if let Some(on_input) = &props.on_input {
                    on_input.emit(input);
                }
                true
            }
            Msg::RevealPassword => {
                self.password_state = PasswordState::Revealed;
                true
            }
            Msg::HidePassword => {
                self.password_state = PasswordState::Hidden;
                true
            }
        }
    }

    fn view(&self, ctx: &ManagedFieldContext<Self>) -> Html {
        let props = ctx.props();
        let state = ctx.state();

        let (value, valid) = (&state.value, &state.valid);
        let value = value_to_text(value);

        let input_type = match self.password_state {
            PasswordState::Hidden => AttrValue::Static("password"),
            PasswordState::Revealed => AttrValue::Static("text"),
            PasswordState::NotAPassword => props.input_type.clone(),
        };
        let oninput = ctx.link().callback(move |event: InputEvent| {
            let input: HtmlInputElement = event.target_unchecked_into();
            Msg::Update(input.value())
        });

        let disabled = props.input_props.disabled;
        let input: Html = Input::new()
            .node_ref(self.input_ref.clone())
            .with_input_props(&props.input_props)
            .class("pwt-flex-fill")
            .attribute("type", input_type)
            .attribute("readonly", disabled.then_some(""))
            .attribute("value", value)
            .attribute("min", props.min.map(|v| v.to_string()))
            .attribute("max", props.max.map(|v| v.to_string()))
            .attribute("step", props.step.map(|v| v.to_string()))
            .oninput((!disabled).then_some(oninput))
            .into();

        let peek_icon =
            if self.password_state != PasswordState::NotAPassword && props.show_peek_icon {
                let is_hidden = matches!(self.password_state, PasswordState::Hidden);
                let onclick = ctx.link().callback(move |_| {
                    if is_hidden {
                        Msg::RevealPassword
                    } else {
                        Msg::HidePassword
                    }
                });
                let class = classes!(
                    "fa",
                    if is_hidden { "fa-eye" } else { "fa-eye-slash" },
                    "pwt-pointer"
                );
                let tooltip_text = if is_hidden {
                    tr!("Show Password")
                } else {
                    tr!("Hide Password")
                };
                Some(Tooltip::new(html! { <i {class} {onclick}/> }).tip(tooltip_text))
            } else {
                None
            };
        let mut input_container = Container::new()
            .with_std_props(&props.std_props)
            .listeners(&props.listeners)
            .class("pwt-input")
            .class(format!("pwt-input-type-{}", props.input_type))
            .class(disabled.then_some("disabled"))
            .class("pwt-w-100")
            .class(if valid.is_ok() {
                "is-valid"
            } else {
                "is-invalid"
            });

        for (class, right) in &props.icons {
            if !right {
                let class = class.to_string();
                let outer_class = "pwt-flex-fill-first-child pwt-d-flex pwt-align-self-center";
                input_container.add_child(html! {<div class={outer_class}><i {class} /></div>});
            }
        }

        input_container.add_child(input);

        for (class, right) in &props.icons {
            if *right {
                let class = class.to_string();
                let outer_class = "pwt-flex-fill-first-child pwt-d-flex pwt-align-self-center";
                input_container.add_child(html! {<div class={outer_class}><i {class} /></div>});
            }
        }
        input_container.add_optional_child(peek_icon);

        let mut tooltip = Tooltip::new(input_container);

        if let Err(msg) = &valid {
            tooltip.set_tip(msg.clone())
        } else if let Some(tip) = &props.tip {
            if !disabled {
                tooltip.set_tip(tip.clone())
            }
        }

        tooltip.into()
    }

    fn rendered(&mut self, ctx: &ManagedFieldContext<Self>, first_render: bool) {
        if first_render {
            let props = ctx.props();
            if props.input_props.autofocus {
                if let Some(el) = self.input_ref.cast::<web_sys::HtmlElement>() {
                    let _ = el.focus();
                }
            }
        }
    }
}
