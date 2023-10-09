use std::fmt::Display;
use std::marker::PhantomData;

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

pub type PwtNumber<T> = ManagedFieldMaster<NumberField<T>>;

#[doc(hidden)]
pub trait NumberTypeInfo:
    PartialEq + PartialOrd + Display + Copy + Clone + Sized + 'static
{
    fn value_to_number(value: &Value) -> Result<Self, Error>;
    fn number_to_value(&self) -> Value;
}

impl NumberTypeInfo for f64 {
    fn value_to_number(value: &Value) -> Result<f64, Error> {
        match value {
            Value::Number(n) => match n.as_f64() {
                Some(n) => Ok(n),
                None => return Err(Error::msg(tr!("cannot represent number as f64"))),
            },
            Value::String(s) => Ok(s.parse()?),
            _ => return Err(Error::msg(tr!("got wrong data type"))),
        }
    }
    fn number_to_value(&self) -> Value {
        (*self).into()
    }
}

// Note: Error message from rust parse() are not gettext translated, so try to do all
// integer range checks ourselves,

macro_rules! signed_number_impl {
    ($T:ty) => {
        impl NumberTypeInfo for $T {
            fn value_to_number(value: &Value) -> Result<$T, Error> {
                match value {
                    Value::Number(n) => match n.as_i64() {
                        Some(n) => {
                            if n > (<$T>::MAX as i64) {
                                return Err(Error::msg(tr!(
                                    "number too large (n > {})",
                                    <$T>::MAX
                                )));
                            }
                            if n < (<$T>::MIN as i64) {
                                return Err(Error::msg(tr!(
                                    "number too small (n < {})",
                                    <$T>::MIN
                                )));
                            }
                            Ok(n as $T)
                        }
                        None => {
                            return Err(Error::msg(tr!(
                                "cannot represent number as signed integer"
                            )))
                        }
                    },
                    Value::String(s) => {
                        let n: i128 = match s.parse() {
                            Ok(n) => n,
                            Err(_) => {
                                return Err(Error::msg(tr!(
                                    "cannot represent number as signed integer"
                                )))
                            }
                        };
                        if n > (<$T>::MAX as i128) {
                            return Err(Error::msg(tr!("number too large (n > {})", <$T>::MAX)));
                        }
                        if n < (<$T>::MIN as i128) {
                            return Err(Error::msg(tr!("number too small (n < {})", <$T>::MIN)));
                        }
                        Ok(s.parse()?)
                    }
                    _ => return Err(Error::msg(tr!("got wrong data type"))),
                }
            }
            fn number_to_value(&self) -> Value {
                (*self).into()
            }
        }
    };
}

macro_rules! unsigned_number_impl {
    ($T:ty) => {
        impl NumberTypeInfo for $T {
            fn value_to_number(value: &Value) -> Result<$T, Error> {
                match value {
                    Value::Number(n) => match n.as_u64() {
                        Some(n) => {
                            if n > (<$T>::MAX as u64) {
                                return Err(Error::msg(tr!(
                                    "number too large (n > {})",
                                    <$T>::MAX
                                )));
                            }
                            if n < (<$T>::MIN as u64) {
                                return Err(Error::msg(tr!(
                                    "number too small (n < {})",
                                    <$T>::MIN
                                )));
                            }
                            Ok(n as $T)
                        }
                        None => {
                            return Err(Error::msg(tr!(
                                "cannot represent number as unsigned integer"
                            )))
                        }
                    },
                    Value::String(s) => {
                        let n: u128 = match s.parse() {
                            Ok(n) => n,
                            Err(_) => {
                                return Err(Error::msg(tr!(
                                    "cannot represent number as unsigned integer"
                                )))
                            }
                        };
                        if n > (<$T>::MAX as u128) {
                            return Err(Error::msg(tr!("number too large (n > {})", <$T>::MAX)));
                        }
                        if n < (<$T>::MIN as u128) {
                            return Err(Error::msg(tr!("number too small (n < {})", <$T>::MIN)));
                        }
                        Ok(s.parse()?)
                    }
                    _ => return Err(Error::msg(tr!("got wrong data type"))),
                }
            }
            fn number_to_value(&self) -> Value {
                (*self).into()
            }
        }
    };
}

signed_number_impl!(i64);
signed_number_impl!(i32);
signed_number_impl!(i16);
signed_number_impl!(i8);

unsigned_number_impl!(u64);
unsigned_number_impl!(u32);
unsigned_number_impl!(u16);
unsigned_number_impl!(u8);

// Note: We need to store numbers as strings while editing, because
// the conversion String/Number is not bijective, and would lead to strange effects
// especially for floating point numbers.
//
// Instead, we use a hook which tranlates values when the user
// calls [get_submit_data](crate::widget::form::FormContext::get_submit_data).


/// Number input element for common Rust types (f64, u8, u16, u32, u64, i8, i16, i32, i64)
///
/// When used inside a [FormContext](crate::widget::form::FormContext), values are submitted a json numbers (not strings).
///
/// Usage examples:
/// ```
/// # use pwt::widget::form::Number;
/// # fn test() {
/// let f64_input = Number::<f64>::new();
/// let u8_input = Number::<u8>::new();
/// # }
/// ```
#[widget(pwt=crate, comp=ManagedFieldMaster<NumberField<T>>, @input, @element)]
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct Number<T: NumberTypeInfo> {
    /// Minimum value for number fields.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub min: Option<T>,

    /// Maximum value for number fields.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub max: Option<T>,

    /// Step value for number fields.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub step: Option<T>,

    /// Force value.
    ///
    /// To implement controlled components (for use without a FormContext).
    /// This is ignored if the field has a name.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub value: Option<T>,

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
    pub default: Option<T>,

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
    pub validate: Option<ValidateFn<T>>,

    /// Change callback
    ///
    /// This callback is emited on any data change, i.e. if data
    /// inside the [FormContext](super::FormContext) changed.
    #[builder_cb(IntoEventCallback, into_event_callback, Option<Result<T, String>>)]
    #[prop_or_default]
    pub on_change: Option<Callback<Option<Result<T, String>>>>,

    /// Input callback
    ///
    /// This callback is emited when the user types in new data.
    #[builder_cb(IntoEventCallback, into_event_callback, String)]
    #[prop_or_default]
    pub on_input: Option<Callback<String>>,
}

impl<T: NumberTypeInfo> Number<T> {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to set the validate callback
    pub fn validate(mut self, validate: impl IntoValidateFn<T>) -> Self {
        self.set_validate(validate);
        self
    }

    /// Method to set the validate callback
    pub fn set_validate(&mut self, validate: impl IntoValidateFn<T>) {
        self.validate = validate.into_validate_fn();
    }
}

pub enum Msg {
    Update(String),
}

#[doc(hidden)]
pub struct NumberField<T> {
    input_ref: NodeRef,
    _phantom_data: PhantomData<T>,
}

// Note: This is called on submit, but only for valid fields
fn value_to_number(value: Value) -> Value {
    match &value {
        Value::Number(_) | Value::Null => value,
        Value::String(text) => {
            if text.is_empty() {
                return Value::Null;
            } // fixme: howto handle submit_empty?
            let number: serde_json::Number = text.parse().unwrap();
            Value::Number(number)
        }
        _ => unreachable!(),
    }
}

impl<T: NumberTypeInfo> ManagedField for NumberField<T> {
    type Properties = Number<T>;
    type Message = Msg;

    fn create_validation_fn(props: &Number<T>) -> ValidateFn<Value> {
        let props = props.clone();
        ValidateFn::new(move |value: &Value| {
            let is_empty = match value {
                Value::Null => true,
                Value::Number(_) => false,
                Value::String(v) => v.is_empty(),
                _ => return Err(Error::msg(tr!("Got wrong data type!"))),
            };

            if is_empty {
                if props.input_props.required {
                    return Err(Error::msg(tr!("Field may not be empty.")));
                } else {
                    return Ok(());
                }
            }

            let number = match T::value_to_number(value) {
                Ok(number) => number,
                Err(err) => return Err(Error::msg(tr!("Parse number failed: {}", err.to_string()))),
            };

            if let Some(min) = props.min {
                if number < min {
                    return Err(Error::msg(tr!(
                        "value must be greater than or equal to '{0}'",
                        min
                    )));
                }
            }
            if let Some(max) = props.max {
                if number > max {
                    return Err(Error::msg(tr!(
                        "value must be less than or equal to '{0}'",
                        max
                    )));
                }
            }

            match &props.validate {
                Some(cb) => cb.validate(&number),
                None => Ok(()),
            }
        })
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

        let default = match props.default {
            Some(default) => default.to_string().into(),
            None => String::new().into(),
        };

        ManagedFieldState {
            value: value,
            valid: Ok(()),
            default,
            radio_group: false,
            unique: false,
            submit_converter: Some(Callback::from(value_to_number)),
        }
    }

    fn value_changed(&mut self, ctx: &super::ManagedFieldContext<Self>) {
        let props = ctx.props();
        let state = ctx.state();
        let data = match &state.valid {
            Ok(()) => Some(T::value_to_number(&state.value).map_err(|err| err.to_string())),
            Err(err) => Some(Err(err.clone())),
        };
        if let Some(on_change) = &props.on_change {
            on_change.emit(data);
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

    fn create(_ctx: &ManagedFieldContext<Self>) -> Self {
        Self {
            input_ref: NodeRef::default(),
            _phantom_data: PhantomData::<T>,
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
        }
    }

    fn view(&self, ctx: &ManagedFieldContext<Self>) -> Html {
        let props = ctx.props();
        let state = ctx.state();

        let (value, valid) = (&state.value, &state.valid);
        let value_text = match value {
            Value::Null => String::new(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.to_string(),
            _ => String::new(),
        };

        let input_type = "number";

        let oninput = ctx.link().callback(move |event: InputEvent| {
            let input: HtmlInputElement = event.target_unchecked_into();
            Msg::Update(input.value())
        });

        let input: Html = Input::new()
            .node_ref(self.input_ref.clone())
            .with_input_props(&props.input_props)
            .class("pwt-flex-fill")
            .attribute("type", input_type)
            .attribute("value", value_text)
            .attribute("min", props.min.map(|v| v.to_string()))
            .attribute("max", props.max.map(|v| v.to_string()))
            .attribute("step", props.step.map(|v| v.to_string()))
            .oninput(oninput)
            .into();

        let input_container = Container::new()
            .with_std_props(&props.std_props)
            .listeners(&props.listeners)
            .class("pwt-input")
            .class(format!("pwt-input-type-{}", input_type))
            .class("pwt-w-100")
            .class(if valid.is_ok() {
                "is-valid"
            } else {
                "is-invalid"
            })
            .with_child(input);

        let mut tooltip = Tooltip::new(input_container);

        if let Err(msg) = &valid {
            tooltip.set_tip(msg.clone())
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
