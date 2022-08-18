use anyhow::{bail, Error};
use serde_json::Value;
use web_sys::HtmlInputElement;

use yew::prelude::*;
use yew::html::{IntoEventCallback, IntoPropValue};

use crate::prelude::*;
use crate::widget::Tooltip;
use crate::widget::form::{Input, ValidateFn};

use super::{FormContext, FieldOptions};

use pwt_macros::widget;

#[widget(PwtField, @input, @element)]
#[derive(Clone, PartialEq, Properties)]
pub struct Field {
    pub name: AttrValue,
    
    #[prop_or(AttrValue::Static("text"))]
    pub input_type: AttrValue,
    pub default: Option<AttrValue>,
    pub validate: Option<ValidateFn<Value>>,

    pub on_change: Option<Callback<String>>,
}

impl Field {

    pub fn new(name: impl IntoPropValue<AttrValue>) -> Self {
        yew::props!(Self {
            name: name.into_prop_value(),
        })
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

    pub fn validate(
        mut self,
        validate: impl 'static + Fn(&Value) -> Result<(), Error>,
    ) -> Self {
        self.validate = Some(ValidateFn::new(validate));
        self
    }

    /// Builder style method to set the on_change callback
    pub fn on_change(mut self, cb: impl IntoEventCallback<String>) -> Self {
        self.on_change = cb.into_event_callback();
        self
    }
}

pub struct PwtField {
    value: String,

    real_validate: ValidateFn<Value>,
    
    form_ctx: Option<FormContext>,
    _form_ctx_handle: Option<ContextHandle<FormContext>>,
}

impl PwtField {

    fn get_field_data(&self, props: &Field) -> (String, Result<(), String>) {
        if let Some(form_ctx) = &self.form_ctx {
            (
                form_ctx.get_field_text(&props.name),
                form_ctx.get_field_valid(&props.name),
            )
        } else {
            (
                self.value.clone(),
                self.real_validate.validate(&self.value.clone().into())
                    .map_err(|e| e.to_string())
            )
        }
    }

    fn set_value(&mut self, ctx: &Context<Self>, value: String) {

        if self.value == value { return; }
        
        let props = ctx.props();
        
        self.value = value.clone();
 
        if let Some(form_ctx) = &self.form_ctx {
            form_ctx.set_value(&props.name, value.into());
        }

        if let Some(on_change) = &props.on_change {
            on_change.emit(self.value.clone());
        }
 
    }
}

pub enum Msg {
    Update(String),
    FormCtxUpdate(FormContext),
}
 
fn create_field_validation_cb(props: Field) -> ValidateFn<Value> {
    ValidateFn::new(move |value: &Value| {
        let value = match value {
            Value::Null => String::new(),
            Value::String(v) => v.clone(),
            _ => { // should not happen
                log::error!("PwtField: got wrong data type in validate (field '{}')!", props.name);
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

        /*
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
        */
        
        match &props.validate {
            Some(cb) => cb.validate(&value.into()),
            None => Ok(()),
    }
    })
}

impl Component for PwtField {
    type Message = Msg;
    type Properties = Field;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let value = props.default.as_deref().unwrap_or("").to_string();
     
        let on_form_ctx_change = Callback::from({
            let link = ctx.link().clone();
            move |form_ctx: FormContext| link.send_message(Msg::FormCtxUpdate(form_ctx))
        });
        
        let mut _form_ctx_handle = None;
        let mut form_ctx = None;

        let real_validate = create_field_validation_cb(props.clone());
        
        if let Some((form, handle)) = ctx.link().context::<FormContext>(on_form_ctx_change) {
            log::info!("INIT CTX {:?}", form);
            form.register_field(
                &props.name,
                value.clone().into(),
                Some(real_validate.clone()),
                FieldOptions::from_field_props(&props.input_props),
            );
  

            form_ctx = Some(form);
            _form_ctx_handle = Some(handle);
       }

        Self {
            _form_ctx_handle,
            form_ctx,
            value,
            real_validate,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::FormCtxUpdate(form_ctx) => {
                log::info!("FormCtxUpdate");
                self.value = form_ctx.get_field_text(&props.name);
                self.form_ctx = Some(form_ctx);
                true
            }
            Msg::Update(value) => {
                if props.input_props.disabled { return true; }
                log::info!("UPDATE {}", value);
                self.set_value(ctx, value);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let (value, valid) = self.get_field_data(props);
               
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
        //.attribute("min", props.min.map(|v| v.to_string()))
        //.attribute("max", props.max.map(|v| v.to_string()))
        //.attribute("step", props.step.map(|v| v.to_string()))
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


