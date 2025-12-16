use anyhow::Error;
use serde_json::Value;

use crate::dom::align::{AlignOptions, GrowDirection, Point};
use crate::prelude::*;
use crate::props::FieldBuilder;
use crate::widget::form::{
    ManagedField, ManagedFieldContext, ManagedFieldMaster, ManagedFieldState,
};
use crate::widget::{Dropdown, DropdownController};

use yew::html::{IntoEventCallback, IntoPropValue};

use super::date_panel::DatePanel;
use super::plain_date::PlainDate;

use pwt_macros::{builder, widget};

#[widget(pwt=crate, comp=ManagedFieldMaster<DateFieldComp>, @input, @element)]
#[builder]
#[derive(Clone, PartialEq, Properties)]
pub struct DateField {
    /// Force value.
    ///
    /// To implement controlled components (for use without a FormContext).
    /// This is ignored if the field has a name.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub value: Option<AttrValue>,

    /// Default value.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub default: Option<AttrValue>,

    /// The date format string (e.g. "Y-m-d").
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or(AttrValue::from("Y-m-d"))]
    pub format: AttrValue,

    /// Format to use for the submitted value.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub submit_format: Option<AttrValue>,

    /// Callback triggered when the value changes (either via input or picker).
    #[builder_cb(IntoEventCallback, into_event_callback, Option<PlainDate>)]
    #[prop_or_default]
    pub on_change: Option<Callback<Option<PlainDate>>>,

    /// Days of the week to disable in the calendar (0 = Sunday, 6 = Saturday).
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub disabled_days: Vec<u32>,

    /// Allow manual entry of date. Defaults to true.
    #[builder]
    #[prop_or(true)]
    pub editable: bool,

    /// The minimum allowed date.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub min_value: Option<PlainDate>,

    /// The maximum allowed date.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub max_value: Option<PlainDate>,

    /// Callback to disable specific dates.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub disabled_dates: Option<Callback<PlainDate, bool>>,

    /// Alternative date formats to try if the primary format fails.
    /// Defaults to a comprehensive list of common formats.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or(AttrValue::from(
        "m/d/Y|n/j/Y|n/j/y|m/j/y|n/d/y|m/j/Y|n/d/Y|m-d-y|m-d-Y|m/d|m-d|md|mdy|mdY|d|Y-m-d|n-j|n/j"
    ))]
    pub alt_formats: AttrValue,

    /// Show the week numbers.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or(true)]
    pub show_week_numbers: bool,

    /// The index of the first day of the week (0-based, 0 = Sunday). Defaults to 0.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or(0)]
    pub start_day: u32,

    /// False to hide the footer area containing the Today button and disable the keyboard
    /// handler for spacebar that selects the current date.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or(true)]
    pub show_today: bool,
}

impl DateField {
    pub fn new() -> Self {
        yew::props!(Self {})
    }
}

pub enum Msg {
    ValueChange(String),
}

pub struct DateFieldComp {}

impl DateFieldComp {
    fn try_parse(value: &str, format: &str, alt_formats: &str) -> Option<PlainDate> {
        if let Ok(date) = PlainDate::from_format(value, format) {
            return Some(date);
        }
        for fmt in alt_formats.split('|') {
            if let Ok(date) = PlainDate::from_format(value, fmt) {
                return Some(date);
            }
        }
        None
    }
}

#[derive(Clone, PartialEq)]
pub struct DateFieldValidationArgs {
    pub min_value: Option<PlainDate>,
    pub max_value: Option<PlainDate>,
    pub format: AttrValue,
    pub submit_format: Option<AttrValue>,
    pub alt_formats: AttrValue,
    pub disabled_days: Vec<u32>,
    pub disabled_dates: Option<Callback<PlainDate, bool>>,
}

impl ManagedField for DateFieldComp {
    type Properties = DateField;
    type Message = Msg;
    type ValidateClosure = DateFieldValidationArgs;

    fn validation_args(props: &Self::Properties) -> Self::ValidateClosure {
        DateFieldValidationArgs {
            min_value: props.min_value,
            max_value: props.max_value,
            format: props.format.clone(),
            submit_format: props.submit_format.clone(),
            alt_formats: props.alt_formats.clone(),
            disabled_days: props.disabled_days.clone(),
            disabled_dates: props.disabled_dates.clone(),
        }
    }

    fn validator(args: &Self::ValidateClosure, value: &Value) -> Result<Value, Error> {
        match value {
            Value::Null => Ok(value.clone()),
            Value::String(s) => {
                if s.is_empty() {
                    return Ok(Value::String(String::new()));
                }
                if let Some(date) = Self::try_parse(s, &args.format, &args.alt_formats) {
                    if let Some(min) = &args.min_value {
                        if date < *min {
                            return Err(anyhow::anyhow!(tr!(
                                "Date is before minimum allowed date"
                            )));
                        }
                    }
                    if let Some(max) = &args.max_value {
                        if date > *max {
                            return Err(anyhow::anyhow!(tr!("Date is after maximum allowed date")));
                        }
                    }
                    if args.disabled_days.contains(&date.week_day()) {
                        return Err(anyhow::anyhow!(tr!("Date is disabled")));
                    }
                    if let Some(callback) = &args.disabled_dates {
                        if callback.emit(date) {
                            return Err(anyhow::anyhow!(tr!("Date is disabled")));
                        }
                    }

                    // Return value in submit_format
                    let fmt = args.submit_format.as_ref().unwrap_or(&args.format);
                    let s = date.format(fmt);
                    Ok(Value::String(s))
                } else {
                    Err(anyhow::anyhow!(tr!("Invalid date format")))
                }
            }
            _ => Err(anyhow::anyhow!("Invalid value type")),
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

        let default: Value = props.default.as_deref().unwrap_or("").into();

        ManagedFieldState::new(value, default)
    }

    fn create(_ctx: &ManagedFieldContext<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, ctx: &ManagedFieldContext<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ValueChange(val) => {
                ctx.link().update_value(val);
                true
                //if let Some(on_input) = &ctx.props().on_input {
                //    on_input.emit(None);
                //}
            }
        }
    }

    fn value_changed(&mut self, ctx: &ManagedFieldContext<Self>) {
        let props = ctx.props();
        let state = ctx.state();
        let value = match &state.value {
            Value::String(s) => s.clone(),
            _ => "".to_string(),
        };
        let date = Self::try_parse(&value, &props.format, &props.alt_formats);
        if let Some(on_change) = &props.on_change {
            on_change.emit(date);
        }
    }

    fn changed(&mut self, ctx: &ManagedFieldContext<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        if props.value != old_props.value {
            ctx.link()
                .force_value(props.value.as_ref().map(|v| v.to_string()), None);
        }
        true
    }

    fn view(&self, ctx: &ManagedFieldContext<Self>) -> Html {
        let props = ctx.props();
        let state = ctx.state();
        let value_str = match &state.value {
            Value::String(s) => s.clone(),
            _ => "".to_string(),
        };
        let validation_result = &state.result;

        let current_value = Self::try_parse(&value_str, &props.format, &props.alt_formats);
        let props = props.clone();

        // The picker function
        let picker = move |controller: &DropdownController| {
            let on_select = controller.on_select_callback();

            DatePanel::new()
                .value(current_value)
                .disabled_days(props.disabled_days.clone())
                .min_value(props.min_value)
                .max_value(props.max_value)
                .disabled_dates(props.disabled_dates.clone())
                .show_week_numbers(props.show_week_numbers)
                .start_day(props.start_day)
                .show_today(props.show_today)
                .on_select(on_select)
                .into()
        };

        let align_options = AlignOptions::new(
            Point::BottomStart,
            Point::TopStart,
            GrowDirection::TopBottom,
        )
        .viewport_padding(5.0)
        .align_width(false);

        let is_valid = validation_result.is_ok();
        let tip = validation_result.as_ref().err().map(|err| err.to_string());

        Dropdown::new(picker)
            .with_std_props(&props.std_props)
            .with_input_props(&props.input_props)
            .align_options(align_options)
            .value(value_str)
            .valid(is_valid)
            .tip(tip)
            .editable(props.editable)
            .on_change(ctx.link().callback(move |val| Msg::ValueChange(val)))
            .with_trigger("fa fa-calendar", true)
            .into()
    }
}
