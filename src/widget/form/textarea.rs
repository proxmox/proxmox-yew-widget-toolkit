use anyhow::Error;
use serde_json::Value;

use web_sys::HtmlTextAreaElement;
use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;
use yew::virtual_dom::{Listeners, VTag};

use pwt_macros::{builder, widget};

use super::{
    IntoValidateFn, ManagedField, ManagedFieldContext, ManagedFieldMaster, ManagedFieldState,
    ValidateFn,
};
use crate::props::EventSubscriber;

use crate::tr;

pub type PwtTextArea = ManagedFieldMaster<TextAreaField>;

/// Checkbox input element, which stores values as boolean
#[widget(pwt=crate, comp=ManagedFieldMaster<TextAreaField>, @input, @element)]
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct TextArea {
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
}

impl TextArea {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
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
}

fn create_field_validation_cb(props: TextArea) -> ValidateFn<Value> {
    ValidateFn::new(move |value: &Value| {
        let value = match value {
            Value::Null => String::new(),
            Value::String(v) => v.clone(),
            _ => {
                // should not happen
                log::error!("TextArea: got wrong data type in validate!");
                String::new()
            }
        };

        if value.is_empty() {
            if props.input_props.required {
                return Err(Error::msg(tr!("Field may not be empty.")));
            } else {
                return Ok(());
            }
        }

        match &props.validate {
            Some(cb) => cb.validate(&value),
            None => Ok(()),
        }
    })
}

pub enum Msg {
    Update(String),
}

#[doc(hidden)]
pub struct TextAreaField {
    input_ref: NodeRef,
}

// TextArea is type Value::String()
fn value_to_text(value: &Value) -> String {
    match value {
        Value::String(s) => s.to_string(),
        _ => String::new(),
    }
}

impl ManagedField for TextAreaField {
    type Properties = TextArea;
    type Message = Msg;

    fn setup(props: &Self::Properties) -> ManagedFieldState {
        let mut value = String::new();

        if let Some(default) = &props.default {
            value = default.to_string();
        }
        if let Some(force_value) = &props.value {
            value = force_value.to_string();
        }

        let validate = create_field_validation_cb(props.clone());
        let value: Value = value.clone().into();
        let valid = validate.validate(&value).map_err(|err| err.to_string());

        let default = props.default.as_deref().unwrap_or("").into();

        ManagedFieldState {
            value: value,
            valid,
            validate,
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

    fn create(_ctx: &ManagedFieldContext<Self>) -> Self {
        Self {
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
        }
    }

    fn view(&self, ctx: &ManagedFieldContext<Self>) -> Html {
        let props = ctx.props().clone();
        let state = ctx.state();

        let (value, valid) = (&state.value, &state.valid);
        let value = value_to_text(value);

        let oninput = ctx.link().callback(move |event: InputEvent| {
            let input: HtmlTextAreaElement = event.target_unchecked_into();
            Msg::Update(input.value())
        });

        let disabled = props.input_props.disabled;
        let props = props.oninput((!disabled).then_some(oninput));

        let classes = classes!(
            "pwt-textarea",
            if valid.is_ok() {
                "is-valid"
            } else {
                "is-invalid"
            },
            disabled.then_some("disabled"),
        );

        let mut attributes = props.std_props.cumulate_attributes(Some(classes));
        let attr_map = attributes.get_mut_index_map();
        props.input_props.cumulate_attributes(attr_map);

        let listeners = Listeners::Pending(props.listeners.listeners.clone().into_boxed_slice());

        let textarea = VTag::__new_textarea(
            Some(value.into()),
            props.std_props.node_ref.clone(),
            props.std_props.key.clone(),
            attributes,
            listeners,
        );

        textarea.into()
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
