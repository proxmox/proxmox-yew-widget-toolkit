use serde_json::Value;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;

use pwt_macros::{builder, widget};

use super::{ManagedField, ManagedFieldContext, ManagedFieldMaster, ManagedFieldState};

pub type PwtHidden = ManagedFieldMaster<HiddenField>;

#[cfg(doc)]
use super::FormContext;

/// Hidden input element (for use with [FormContext])
///
/// Stores/Reads data using the form context. Displays nothing.
///
/// This field can store any json data (strings, objects, arrays),
/// and there is no validation.
#[widget(pwt=crate, comp=ManagedFieldMaster<HiddenField>, @input)]
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct Hidden {
    /// Default value.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub default: Option<Value>,

    /// Change callback
    #[builder_cb(IntoEventCallback, into_event_callback, Value)]
    #[prop_or_default]
    pub on_change: Option<Callback<Value>>,
}

impl Default for Hidden {
    fn default() -> Self {
        Self::new()
    }
}

impl Hidden {
    /// Creates a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }
}

#[doc(hidden)]
pub struct HiddenField {
    state: ManagedFieldState,
}

crate::impl_deref_mut_property!(HiddenField, state, ManagedFieldState);

impl ManagedField for HiddenField {
    type Properties = Hidden;
    type Message = ();
    type ValidateClosure = ();

    fn validation_args(_props: &Self::Properties) -> Self::ValidateClosure {}

    fn create(ctx: &ManagedFieldContext<Self>) -> Self {
        let props = ctx.props();
        let mut value = Value::Null;
        if let Some(default) = &props.default {
            value = default.clone();
        }

        let default = props.default.clone().unwrap_or(Value::Null);

        Self {
            state: ManagedFieldState::new(value, default),
        }
    }

    fn value_changed(&mut self, ctx: &ManagedFieldContext<Self>) {
        let props = ctx.props();
        if let Some(on_change) = &props.on_change {
            on_change.emit(self.value.clone());
        }
    }

    fn view(&self, _ctx: &ManagedFieldContext<Self>) -> Html {
        html! {}
    }
}
