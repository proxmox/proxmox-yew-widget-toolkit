use anyhow::{bail, format_err,  Error};
use serde_json::Value;
use web_sys::HtmlInputElement;

use yew::prelude::*;
use yew::html::{IntoEventCallback, IntoPropValue};

use proxmox_schema::Schema;

use crate::prelude::*;
use crate::widget::Tooltip;

use super::{FormContext, FieldOptions, Input, TextFieldStateHandle, ValidateFn};

use pwt_macros::widget;

#[widget(PwtField, @input, @element)]
#[derive(Clone, PartialEq, Properties)]
pub struct Field {
    /// Name of the form field.
    ///
    /// The field register itself with this `name` in the FormContext
    /// (if any).
    pub name: Option<AttrValue>,

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
    pub value: Option<AttrValue>,

    /// Default value.
    pub default: Option<AttrValue>,
    /// Validation function.
    pub validate: Option<ValidateFn<String>>,

    /// Change callback
    pub on_change: Option<Callback<String>>,
}

impl Field {

    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to set the field name.
    pub fn name(mut self, name: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_name(name);
        self
    }

    /// Method to set the field name.
    pub fn set_name(&mut self, name: impl IntoPropValue<Option<AttrValue>>) {
        self.name = name.into_prop_value();
    }

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

    pub fn input_type(mut self, input_type: impl IntoPropValue<AttrValue>) -> Self {
        self.set_input_type(input_type);
        self
    }

    pub fn set_input_type(&mut self, input_type: impl IntoPropValue<AttrValue>) {
        self.input_type = input_type.into_prop_value();
    }

    pub fn default(mut self, default: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_default(default);
        self
    }

    pub fn set_default(&mut self, default: impl IntoPropValue<Option<AttrValue>>) {
        self.default = default.into_prop_value();
    }

    /// Builder style method to set the validate callback
    pub fn validate(
        mut self,
        validate: impl 'static + Fn(&String) -> Result<(), Error>,
    ) -> Self {
        self.set_validate(validate);
        self
    }

    /// Method to set the validate callback
    pub fn set_validate(
        &mut self,
        validate: impl 'static + Fn(&String) -> Result<(), Error>,
    ) {
        self.validate = Some(ValidateFn::new(validate));
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
}

#[doc(hidden)]
pub struct PwtField {
    state: TextFieldStateHandle,
    real_validate: ValidateFn<Value>,
}

pub enum Msg {
    Update(String),
    FormCtxUpdate(FormContext),
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

impl Component for PwtField {
    type Message = Msg;
    type Properties = Field;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let value = props.value.as_deref()
            .or(props.default.as_deref())
            .unwrap_or("").to_string();

        let on_form_ctx_change = Callback::from({
            let link = ctx.link().clone();
            move |form_ctx: FormContext| link.send_message(Msg::FormCtxUpdate(form_ctx))
        });

        let real_validate = create_field_validation_cb(props.clone());

        let state = TextFieldStateHandle::new(
            ctx.link(),
            on_form_ctx_change,
            props.name.clone(),
            value,
            Some(real_validate.clone()),
            FieldOptions::from_field_props(&props.input_props),
            props.on_change.clone(),
        );

        Self { state, real_validate }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::FormCtxUpdate(form_ctx) => self.state.update(form_ctx),
            Msg::Update(value) => {
                if props.input_props.disabled { return true; }
                self.state.set_value(value);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let (value, valid) = match &props.value {
            Some(value) => {
                let value = value.to_string();
                let valid = self
                    .real_validate.validate(&value.clone().into())
                    .map_err(|err| err.to_string());
                (value, valid)
            }
            None => self.state.get_field_data()
        };

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
