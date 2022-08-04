use anyhow::Error;
use serde_json::Value;

use proxmox_schema::Schema;

use web_sys::HtmlInputElement;

use yew::prelude::*;
use yew::html::{IntoEventCallback, IntoPropValue};

use crate::prelude::*;
use crate::widget::Tooltip;
use crate::widget::form::Input;

use super::ValidateFn;

use pwt_macros::widget;

#[widget(PwtField, @input, @element)]
#[derive(Clone, PartialEq, Properties)]
pub struct Field {

    #[prop_or(String::from("text"))]
    pub input_type: String,

    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,

    pub default: Option<AttrValue>,

    pub validate: Option<ValidateFn<Value>>,

    pub on_change: Option<Callback<String>>,
}

impl Field {

    pub fn new() -> Self {
        yew::props!(Field {})
    }

    pub fn default(mut self, default: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_default(default);
        self
    }

    pub fn set_default(&mut self, default: impl IntoPropValue<Option<AttrValue>>) {
        self.default = default.into_prop_value();
    }

    pub fn number(
        min: impl IntoPropValue<Option<f64>>,
        max: impl IntoPropValue<Option<f64>>,
        step: impl IntoPropValue<Option<f64>>
    ) -> Self {
        let mut me = Self::new();
        me.min = min.into_prop_value();
        me.max = max.into_prop_value();
        me.step = step.into_prop_value();
        me.input_type = String::from("number");
        me
    }

    pub fn validate(
        mut self,
        validate: impl 'static + Fn(&Value) -> Result<(), Error>,
    ) -> Self {
        self.validate = Some(ValidateFn::new(validate));
        self
    }

    pub fn schema(mut self, schema: &'static Schema) -> Self {
        match schema {
            Schema::Integer(s) => {
                self.min = s.minimum.map(|v| v as f64);
                self.max = s.maximum.map(|v| v as f64);
                self.step = Some(1.0);
                self.input_type = String::from("number");
            }
            Schema::Number(s) => {
                self.min = s.minimum;
                self.max = s.maximum;
                self.step = Some(1.0);
                self.input_type = String::from("number");
            }
            _ => {}
        }

        self.validate = Some(ValidateFn::new(move |value: &Value| {
            let value = value.as_str().unwrap_or("");
            schema.parse_simple_value(value)?;
            Ok(())
        }));
        self
    }

    pub fn input_type(mut self, input_type: impl Into<String>) -> Self {
        self.input_type = input_type.into();
        self
    }

    /// Builder style method to set the on_change callback
    pub fn on_change(mut self, cb: impl IntoEventCallback<String>) -> Self {
        self.on_change = cb.into_event_callback();
        self
    }
}

pub enum Msg {
    Update(String),
}

pub struct PwtField {
    value: String,
    valid: Result<(), String>,
    last_version: usize,
}

impl PwtField {

    fn get_value(&self, ctx: &Context<Self>) -> String {
        // Fixme: improve type handling: Integer, Number, ...
        match  &ctx.props().input_props.form_ref {
            Some(form_ref) => {
                match form_ref.get_value() {
                    Value::Null => String::new(),
                    Value::Bool(v) => String::from(if v { "true" } else { "false" }), //???
                    Value::Number(v) => v.to_string(),
                    Value::String(v) => v.clone(),
                    _ => { // should not happen
                        log::error!("PwtField: ignoring complex datatypes!");
                        String::new()
                    }
                }
            }
            None => self.value.clone()
        }
    }

    fn get_valid(&self, ctx: &Context<Self>) -> Result<(), String> {
        match &ctx.props().input_props.form_ref {
            Some(form_ref) => form_ref.get_valid(),
            None => self.valid.clone()
        }
    }

    fn set_value(&mut self, ctx: &Context<Self>, value: String, default: Option<String>) {
        let props = ctx.props();

        self.value = value.clone();
        let valid = Self::validate_field_value(props, &self.value);
        self.valid = valid.clone();

        if let Some(form_ref) = &props.input_props.form_ref {
            form_ref.form.with_field_state_mut(&form_ref.field_name, move |field| {
                field.value = value.clone().into();
                field.valid = valid.clone();
                if let Some(default) = &default {
                    field.initial_value = default.as_str().into();
                }
            });
        }

        if let Some(on_change) = &props.on_change {
            on_change.emit(self.value.clone());
        }
    }

    fn validate_field_value(props: &Field, value: &str) -> Result<(), String> {
        if value.is_empty() {
            if props.input_props.required {
                return Err(String::from("Field may not be empty."));
            } else {
                return Ok(())
            }
        }

        if props.input_type == "number" {
            let value_f64 = match value.parse::<f64>() {
                Ok(v) => v,
                Err(err) => return Err(err.to_string()),
            };
            if let Some(min) = props.min {
                if value_f64 < min {
                    return Err(format!("value must be greate than or equal to '{}'", min));
                }
            }
            if let Some(max) = props.max {
                if value_f64 > max {
                    return Err(format!("value must be less than or equal to '{}'", max));
                }
            }
        }
        match &props.validate {
            Some(cb) => {
                cb.validate(&value.into()).map_err(|e| e.to_string())
            }
            None => Ok(()),
        }
    }
}

impl Component for PwtField {
    type Message = Msg;
    type Properties = Field;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let value = props.default.as_deref().unwrap_or("").to_string();
        let valid = Self::validate_field_value(props, &value);

        props.input_props.register_form_field(value.clone().into(), valid.clone());

        Self {
            value: value,
            valid: valid,
            last_version: 0,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::Update(value) => {
                if props.input_props.disabled { return true; }
                self.set_value(ctx, value, None);
                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        let props = ctx.props();

        if let Some(form_ref) = &props.input_props.form_ref {
            if form_ref.version != self.last_version {
                self.last_version = form_ref.version;

                let value = self.get_value(ctx);
                let valid = self.get_valid(ctx);

                // try to keep data in sync
                if value != self.value || valid != self.valid {
                    self.set_value(ctx, value, None);
                }
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {

        let props = ctx.props();

        let valid = self.get_valid(ctx);
        let value = self.get_value(ctx);

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
            .attribute("value", value.clone())
            .attribute("min", props.min.map(|v| v.to_string()))
            .attribute("max", props.max.map(|v| v.to_string()))
            .attribute("step", props.step.map(|v| v.to_string()))
            .oninput(oninput.clone())
            .into();

        let mut tooltip = Tooltip::new()
            .with_child(input);

        if let Err(msg) = &valid {
            tooltip.set_tip(Some(html!{msg}))
        }

        tooltip.into()
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
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
