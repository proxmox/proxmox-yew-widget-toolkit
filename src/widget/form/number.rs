use std::fmt::{Debug, Display};
use std::marker::PhantomData;

use anyhow::Error;

use gloo_timers::callback::{Interval, Timeout};
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
use crate::widget::{Column, Container, Input, Tooltip};

use crate::tr;

const SPINNER_START_DELAY_MS: u32 = 200;
const SPINNER_REPEAT_INTERVAL_MS: u32 = 50;

pub type PwtNumber<T> = ManagedFieldMaster<NumberField<T>>;

#[doc(hidden)]
pub trait NumberTypeInfo:
    Into<Value> + PartialEq + PartialOrd + Display + Default + Debug + Copy + Clone + Sized + 'static
{
    fn value_to_number(value: &Value) -> Result<Self, Error>;
    fn number_to_value(&self) -> Value;

    fn format(&self) -> String;

    fn step_down(&self, step: Option<Self>) -> Self;
    fn step_up(&self, step: Option<Self>) -> Self;

    fn clamp_value(&self, min: Option<Self>, max: Option<Self>) -> Self;
}

impl NumberTypeInfo for f64 {
    fn value_to_number(value: &Value) -> Result<f64, Error> {
        match value {
            Value::Number(n) => match n.as_f64() {
                Some(n) => Ok(n),
                None => return Err(Error::msg(tr!("cannot represent number as f64"))),
            },
            Value::String(s) => {
                // Note: this handles localized number format
                let number = crate::dom::parse_float(s).map_err(|err| Error::msg(err))?;

                return Ok(number);
            }
            _ => return Err(Error::msg(tr!("got wrong data type"))),
        }
    }
    fn number_to_value(&self) -> Value {
        (*self).into()
    }
    fn format(&self) -> String {
        crate::dom::format_float(*self)
    }
    fn step_up(&self, step: Option<Self>) -> Self {
        self + step.unwrap_or(1.0)
    }
    fn step_down(&self, step: Option<Self>) -> Self {
        self - step.unwrap_or(1.0)
    }
    fn clamp_value(&self, min: Option<Self>, max: Option<Self>) -> Self {
        self.clamp(min.unwrap_or(f64::MIN), max.unwrap_or(f64::MAX))
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
            fn format(&self) -> String {
                (*self).to_string()
            }
            fn step_down(&self, step: Option<Self>) -> Self {
                let step = step.unwrap_or(1);
                if *self >= (<$T>::MIN + step) {
                    self - step
                } else {
                    *self
                }
            }
            fn step_up(&self, step: Option<Self>) -> Self {
                let step = step.unwrap_or(1);
                if *self <= (<$T>::MAX - step) {
                    self + step
                } else {
                    *self
                }
            }
            fn clamp_value(&self, min: Option<Self>, max: Option<Self>) -> Self {
                (*self).clamp(min.unwrap_or(<$T>::MIN), max.unwrap_or(<$T>::MAX))
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
            fn format(&self) -> String {
                (*self).to_string()
            }
            fn step_down(&self, step: Option<Self>) -> Self {
                let step = step.unwrap_or(1);
                if *self >= (<$T>::MIN + step) {
                    self - step
                } else {
                    *self
                }
            }
            fn step_up(&self, step: Option<Self>) -> Self {
                let step = step.unwrap_or(1);
                if *self <= (<$T>::MAX - step) {
                    self + step
                } else {
                    *self
                }
            }
            fn clamp_value(&self, min: Option<Self>, max: Option<Self>) -> Self {
                (*self).clamp(min.unwrap_or(<$T>::MIN), max.unwrap_or(<$T>::MAX))
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
/// When used inside a [FormContext](crate::widget::form::FormContext), values are submitted as
/// json numbers (not strings).
///
/// Accepted floating point number format (f64) is:
///
/// ```BNF
/// DecimalPoint ::= 'read from current locale settings'
/// Number       ::= ( Digit+ |
///                    Digit+ DecimalPoint Digit* |
///                    Digit* DecimalPoint Digit+ ) Exp?
/// Exp          ::= 'e' Sign? Digit+
/// Sign         ::= [+-]
//  Digit        ::= [0-9]
/// ```
///
/// Usage examples:
/// ```
/// # use pwt::widget::form::Number;
/// # fn test() {
/// let f64_input = Number::<f64>::new();
/// let u8_input = Number::<u8>::new();
/// # }
/// ```
///
/// # Note
///
/// This widget does not use `<input type="number">` because:
///
/// - when the number input contains an invalid value and you retrieve
///   the value, you get a blank string. There also seems to be some transformation
///   depending on the locale settings. In general, the returned value is not the text
///   presented to the user. This makes it impossible to implement controlled inputs.
/// - different browsers accept different characters.
/// - see <https://stackoverflow.blog/2022/12/26/why-the-number-input-is-the-worst-input>
///
/// For now, we simply use a text input, and handle number related feature ourselves.
///
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
    ///
    /// Note: for f64, value must be formated using the browser locale!
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
    #[builder_cb(IntoEventCallback, into_event_callback, (String, Option<T>))]
    #[prop_or_default]
    pub on_input: Option<Callback<(String, Option<T>)>>,
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
    Up,
    Down,
    SpinnerStart(bool),
    SpinnerStartRepeat(bool),
    SpinnerStop,
}

#[doc(hidden)]
pub struct NumberField<T> {
    input_ref: NodeRef,
    _phantom_data: PhantomData<T>,
    _spinner_start_timeout: Option<Timeout>,
    _spinner_interval: Option<Interval>,
    focus_input: bool,
}

#[derive(PartialEq)]
pub struct ValidateClosure<T> {
    required: bool,
    min: Option<T>,
    max: Option<T>,
    validate: Option<ValidateFn<T>>,
}

impl<T: NumberTypeInfo> ManagedField for NumberField<T> {
    type Properties = Number<T>;
    type Message = Msg;
    type ValidateClosure = ValidateClosure<T>;

    fn validation_args(props: &Self::Properties) -> Self::ValidateClosure {
        ValidateClosure {
            required: props.input_props.required,
            min: props.min,
            max: props.max,
            validate: props.validate.clone(),
        }
    }

    fn validator(props: &Self::ValidateClosure, value: &Value) -> Result<Value, Error> {
        let is_empty = match value {
            Value::Null => true,
            Value::Number(_) => false,
            Value::String(v) => v.is_empty(),
            _ => return Err(Error::msg(tr!("Got wrong data type!"))),
        };

        if is_empty {
            if props.required {
                return Err(Error::msg(tr!("Field may not be empty.")));
            } else {
                return Ok(Value::Null);
            }
        }

        let number = match T::value_to_number(value) {
            Ok(number) => number,
            Err(err) => return Err(Error::msg(tr!("Input invalid: {}", err.to_string()))),
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

        if let Some(validate) = &props.validate {
            validate.apply(&number)?;
        }
        Ok(number.into())
    }

    fn setup(props: &Self::Properties) -> ManagedFieldState {
        let mut value = Value::Null;

        if let Some(default) = props.default {
            value = T::format(&default).into();
        }
        if let Some(force_value) = &props.value {
            value = force_value.to_string().into();
        }

        let value: Value = value.clone().into();

        let default = match props.default {
            Some(default) => T::number_to_value(&default),
            None => Value::Null,
        };

        ManagedFieldState {
            value,
            valid: Ok(()),
            default,
            radio_group: false,
            unique: false,
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
            ctx.link().force_value(
                props.value.as_ref().map(|v| v.to_string()),
                props.valid.clone(),
            );
        }
        true
    }

    fn create(_ctx: &ManagedFieldContext<Self>) -> Self {
        Self {
            input_ref: NodeRef::default(),
            _phantom_data: PhantomData::<T>,
            _spinner_start_timeout: None,
            _spinner_interval: None,
            focus_input: false,
        }
    }

    fn update(&mut self, ctx: &ManagedFieldContext<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        let state = ctx.state();
        match msg {
            Msg::Update(input) => {
                ctx.link().update_value(input.clone());
                if let Some(on_input) = &props.on_input {
                    let value = T::value_to_number(&input.clone().into()).ok();
                    on_input.emit((input, value));
                }
                true
            }
            Msg::Up => {
                let n = T::value_to_number(&state.value).ok();
                let n = match (n, state.valid.is_ok()) {
                    (None, true) => Some(T::default().clamp_value(props.min, props.max)),
                    (Some(n), _) => {
                        let next = T::step_up(&n, props.step);
                        match props.max {
                            Some(max) if next <= max => {}
                            None => {}
                            Some(_) => return false,
                        }
                        Some(next)
                    }
                    (None, false) => None,
                };
                if let Some(n) = n {
                    ctx.link().update_value(T::number_to_value(&n));
                }
                true
            }
            Msg::Down => {
                let n = T::value_to_number(&state.value).ok();
                let n = match (n, state.valid.is_ok()) {
                    (None, true) => Some(T::default().clamp_value(props.min, props.max)),
                    (Some(n), _) => {
                        let next = T::step_down(&n, props.step);
                        match props.min {
                            Some(min) if next >= min => {}
                            None => {}
                            Some(_) => return false,
                        }
                        Some(next)
                    }
                    (None, false) => None,
                };
                if let Some(n) = n {
                    ctx.link().update_value(T::number_to_value(&n));
                }
                true
            }
            Msg::SpinnerStart(up) => {
                let link = ctx.link();
                ctx.link()
                    .send_message(if up { Msg::Up } else { Msg::Down });
                self._spinner_start_timeout =
                    Some(Timeout::new(SPINNER_START_DELAY_MS, move || {
                        link.send_message(Msg::SpinnerStartRepeat(up))
                    }));
                self.focus_input = true;
                true
            }
            Msg::SpinnerStartRepeat(up) => {
                let link = ctx.link();
                self._spinner_interval =
                    Some(Interval::new(SPINNER_REPEAT_INTERVAL_MS, move || {
                        link.send_message(if up { Msg::Up } else { Msg::Down });
                    }));
                false
            }
            Msg::SpinnerStop => {
                self._spinner_start_timeout = None;
                self._spinner_interval = None;
                false
            }
        }
    }

    fn view(&self, ctx: &ManagedFieldContext<Self>) -> Html {
        let props = ctx.props();
        let state = ctx.state();

        let (value, valid) = (&state.value, &state.valid);
        let value_text = match value {
            Value::Null => String::new(),
            Value::Number(number) => match T::value_to_number(value) {
                Ok(n) => T::format(&n),
                Err(_) => number.to_string(),
            },
            Value::String(s) => s.to_string(),
            _ => String::new(),
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
            .attribute("type", "text") // important (text, not number)
            .attribute("role", "spinbutton")
            .attribute("value", value_text)
            .attribute("aria-valuemin", props.min.map(|v| v.to_string()))
            .attribute("aria-valuemax", props.max.map(|v| v.to_string()))
            .oninput(oninput)
            .onkeydown({
                let link = ctx.link();
                move |event: KeyboardEvent| match event.key().as_str() {
                    "ArrowDown" => link.send_message(Msg::Down),
                    "ArrowUp" => link.send_message(Msg::Up),
                    _ => return,
                }
            })
            .onwheel({
                let link = ctx.link();
                move |event: WheelEvent| {
                    match event.delta_y() {
                        delta if delta < 0.0 => link.send_message(Msg::Up),
                        delta if delta > 0.0 => link.send_message(Msg::Down),
                        _ => {}
                    }
                    event.prevent_default();
                }
            })
            .into();

        let input_container = Container::new()
            .with_std_props(&props.std_props)
            .listeners(&props.listeners)
            .class("pwt-input")
            .class("pwt-input-type-number")
            .class("pwt-w-100")
            .class(disabled.then_some("disabled"))
            .class(if valid.is_ok() {
                "is-valid"
            } else {
                "is-invalid"
            })
            .with_child(input)
            .with_child(
                Column::new()
                    .class("spinner")
                    .with_child(
                        Container::from_tag("i")
                            .class("fa fa-angle-up")
                            .onpointerdown(ctx.link().callback(|event: PointerEvent| {
                                event.prevent_default(); // prevent focus loss
                                Msg::SpinnerStart(true)
                            }))
                            .onpointerout(ctx.link().callback(|_| Msg::SpinnerStop))
                            .onpointerup(ctx.link().callback(|_| Msg::SpinnerStop)),
                    )
                    .with_child(
                        Container::from_tag("i")
                            .class("fa fa-angle-down")
                            .onpointerdown(ctx.link().callback(|event: PointerEvent| {
                                event.prevent_default(); // prevent focus loss
                                Msg::SpinnerStart(false)
                            }))
                            .onpointerout(ctx.link().callback(|_| Msg::SpinnerStop))
                            .onpointerup(ctx.link().callback(|_| Msg::SpinnerStop)),
                    ),
            );

        let mut tooltip = Tooltip::new(input_container);

        if let Err(msg) = &valid {
            tooltip.set_tip(msg.clone())
        }

        tooltip.into()
    }

    fn rendered(&mut self, ctx: &ManagedFieldContext<Self>, first_render: bool) {
        if (first_render && ctx.props().input_props.autofocus) || self.focus_input {
            if let Some(el) = self.input_ref.cast::<web_sys::HtmlElement>() {
                let _ = el.focus();
            }
            self.focus_input = false;
        }
    }
}
