use anyhow::{bail, format_err};
use web_sys::HtmlInputElement;
use serde_json::Value;

use yew::prelude::*;
use yew::html::{IntoEventCallback, IntoPropValue};

use proxmox_schema::Schema;

use crate::prelude::*;
use crate::widget::Tooltip;
use crate::widget::form::Input;

use super::{FieldState, FieldStateMsg, IntoValidateFn, ValidateFn};

use pwt_macros::widget;

/// Html input field.
#[widget(pwt=crate, comp=PwtField, @input, @element)]
#[derive(Clone, PartialEq, Properties)]
pub struct Field {
    /// Input type (html input element attribute).
    #[prop_or(AttrValue::Static("text"))]
    pub input_type: AttrValue,

    /// Minimum value for number fields.
    pub min: Option<f64>,
    /// Maximum value for number fields.
    pub max: Option<f64>,
    /// Step value for number fields.
    pub step: Option<f64>,

    /// Force value.
    ///
    /// To implement controlled components (for use without a FormContext).
    /// This is ignored if the field has a name.
    pub value: Option<AttrValue>,

    /// Force validation result.
    ///
    /// To implement controlled components (for use without a FormContext).
    /// This is ignored if the field has a name.
    ///
    /// This is only used if you also force a value, and overwrites
    /// any result from the validation function (if any).
    pub valid: Option<Result<(), String>>,

    /// Default value.
    pub default: Option<AttrValue>,

    /// Validation function.
    ///
    /// # Note
    ///
    /// It is currently not allowed to access the
    /// [FormContext](super::context::FormContext) inside a validation
    /// callback! If you need such functionality, do validation inside
    /// [FormContext::on_change](super::context::FormContext::on_change),
    /// then set the result with
    /// `form_ctx.write().set_field_valid(...)`.
    pub validate: Option<ValidateFn<String>>,

    /// Change callback
    ///
    /// This callback is emited on any data change, i.e. if data
    /// inside the [FormContext](super::context::FormContext) changed.
    pub on_change: Option<Callback<String>>,

    /// Input callback
    ///
    /// This callback is emited when the user types in new data.
    pub on_input: Option<Callback<String>>,
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
        step: impl IntoPropValue<Option<f64>>
    ) -> Self {
        self.min = min.into_prop_value();
        self.max = max.into_prop_value();
        self.step = step.into_prop_value();
        self.input_type = AttrValue::Static("number");
        self
    }

    /// Builder style method to set the `input-type` attribute.
    pub fn input_type(mut self, input_type: impl IntoPropValue<AttrValue>) -> Self {
        self.set_input_type(input_type);
        self
    }

    /// Method to set the `input-type` attribute.
    pub fn set_input_type(&mut self, input_type: impl IntoPropValue<AttrValue>) {
        self.input_type = input_type.into_prop_value();
    }

    /// Builder style method to set the `value` attribute.
    pub fn value(mut self, value: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_value(value);
        self
    }

    /// Method to set the `value` attribute.
    pub fn set_value(&mut self, value: impl IntoPropValue<Option<AttrValue>>) {
        self.value = value.into_prop_value();
    }

    /// Builder style method to set the validation result.
    pub fn valid(mut self, valid: impl IntoPropValue<Option<Result<(), String>>>) -> Self {
        self.set_valid(valid);
        self
    }

    /// Method to set the validation result.
    pub fn set_valid(&mut self, valid: impl IntoPropValue<Option<Result<(), String>>>) {
        self.valid = valid.into_prop_value();
    }

    /// Builder style method to set the default value.
    pub fn default(mut self, default: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_default(default);
        self
    }

    /// Method to set the default value.
    pub fn set_default(&mut self, default: impl IntoPropValue<Option<AttrValue>>) {
        self.default = default.into_prop_value();
    }

    /// Builder style method to set the validate callback
    pub fn validate(
        mut self,
        validate: impl IntoValidateFn<String>,
    ) -> Self {
        self.set_validate(validate);
        self
    }

    /// Method to set the validate callback
    pub fn set_validate(
        &mut self,
        validate: impl IntoValidateFn<String>,
    ) {
        self.validate = validate.into_validate_fn();
    }

    /// Builder style method to set the validation schema
    pub fn schema(mut self, schema: &'static Schema) -> Self {
        self.set_schema(schema);
        self
    }

    /// Method to set the validation schema
    pub fn set_schema(&mut self, schema: &'static Schema) {
        match schema {
            Schema::Integer(s) => {
                self.min = s.minimum.map(|v| v as f64);
                self.max = s.maximum.map(|v| v as f64);
                self.step = Some(1.0);
                self.input_type = AttrValue::Static("number");
            }
            Schema::Number(s) => {
                self.min = s.minimum;
                self.max = s.maximum;
                self.step = Some(1.0);
                self.input_type = AttrValue::Static("number");
            }
            _ => {}
        }
        self.set_validate(move |value: &String| {
            schema.parse_simple_value(value)?;
            Ok(())
        });
    }

    /// Builder style method to set the on_change callback
    pub fn on_change(mut self, cb: impl IntoEventCallback<String>) -> Self {
        self.on_change = cb.into_event_callback();
        self
    }

    /// Builder style method to set the on_input callback
    pub fn on_input(mut self, cb: impl IntoEventCallback<String>) -> Self {
        self.on_input = cb.into_event_callback();
        self
    }
}

pub enum Msg {
    Update(String),
    StateUpdate(FieldStateMsg),
}

fn create_field_validation_cb(props: Field) -> ValidateFn<Value> {
    ValidateFn::new(move |value: &Value| {
        let value = match value {
            Value::Null => String::new(),
            Value::Number(n) => n.to_string(),
            Value::String(v) => v.clone(),
            _ => { // should not happen
                log::error!("PwtField: got wrong data type in validate!");
                String::new()
            }
        };

        if value.is_empty() {
            if props.input_props.required {
                bail!("Field may not be empty.");
            } else {
                return Ok(());
            }
        }

        if props.input_type == "number" {
            let value_f64 = match value.parse::<f64>() {
                Ok(v) => v,
                Err(err) => return Err(format_err!("unable to parse number: {}", err)),
            };
            if let Some(min) = props.min {
                if value_f64 < min {
                    return Err(format_err!("value must be greate than or equal to '{}'", min));
                }
            }
            if let Some(max) = props.max {
                if value_f64 > max {
                    return Err(format_err!("value must be less than or equal to '{}'", max));
                }
            }
        }

        match &props.validate {
            Some(cb) => cb.validate(&value),
            None => Ok(()),
        }
    })
}

#[doc(hidden)]
pub struct PwtField {
    state: FieldState,
}

impl Component for PwtField {
    type Message = Msg;
    type Properties = Field;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let real_validate = create_field_validation_cb(props.clone());

        let on_change = match &props.on_change {
            Some(on_change) => Some(Callback::from({
                let on_change = on_change.clone();
                move |value: Value| {
                    on_change.emit(value.as_str().unwrap_or("").to_string());
                }
            })),
            None => None,
        };

        let state = FieldState::create(
            ctx,
            &props.input_props,
            ctx.link().callback(Msg::StateUpdate),
            on_change,
            real_validate.clone(),
        );

        let value = props.default.as_deref().unwrap_or("").to_string();
        let default = props.default.as_deref().unwrap_or("").to_string();

        let mut me = Self { state };

        if let Some(name) = &props.input_props.name {
            me.state.register_field(&props.input_props, value, default, false);
            if props.value.is_some() || props.valid.is_some() {
                log::error!("Field '{name}' is named - unable to force value/valid");
            }
        } else {
            if let Some(value) = &props.value { // force value
                me.state.force_value(value.to_string(), props.valid.clone());
            }
        }

        me
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::StateUpdate(state_msg) => {
                let default = props.default.as_deref().unwrap_or("").to_string();
                self.state.update_hook(&props.input_props, state_msg, default, false)
            }
            Msg::Update(value) => {
                if props.input_props.disabled { return true; }
                self.state.set_value(value.clone());
                if let Some(on_input) = &props.on_input {
                    on_input.emit(value);
                }
                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();

        if let Some(name) = &props.input_props.name {
            if props.value.is_some() || props.valid.is_some() {
                log::error!("Field '{name}' is named - unable to force value/valid");
            }
            self.state.update_field_options(&props.input_props);
        } else {
            if props.value != old_props.value || props.valid != old_props.valid {
                if let Some(value) = &props.value { // force value
                    self.state.force_value(value.to_string(), props.valid.clone());
                }
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let (value, valid) = self.state.get_field_data();
        let value = value.as_str().unwrap_or("").to_owned();

        let oninput = ctx.link().callback(move |event: InputEvent| {
            let input: HtmlInputElement = event.target_unchecked_into();
            Msg::Update(input.value())
        });

        let input: Html = Input::new()
            .with_std_props(&props.std_props)
            .with_input_props(&props.input_props)
            .class("pwt-input")
            .class("pwt-w-100")
            .class(if valid.is_ok() { "is-valid" } else { "is-invalid" })
            .attribute("type", props.input_type.clone())
            .attribute("value", value)
            .attribute("min", props.min.map(|v| v.to_string()))
            .attribute("max", props.max.map(|v| v.to_string()))
            .attribute("step", props.step.map(|v| v.to_string()))
            .oninput(oninput)
            .into();

        let mut tooltip = Tooltip::new()
            .with_child(input);

        if let Err(msg) = &valid {
            tooltip.set_tip(Some(html!{msg}))
        }

        tooltip.into()
    }
}
