use wasm_bindgen::JsCast;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;

use crate::props::{EventSubscriber, WidgetBuilder};
use crate::tr;
use crate::widget::Button;

use super::{FormContext, FormContextObserver};

use pwt_macros::{builder, widget};

/// Reset button.
///
/// The button automatically listens for [FormContext] changes, and
/// enables the button only if the form is dirty (contains
/// modified data).
#[widget(pwt=crate, comp=crate::widget::form::PwtResetButton, @element)]
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct ResetButton {
    /// Button text (default "Reset").
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub text: Option<AttrValue>,

    /// Reset button press callback.
    #[prop_or_default]
    #[builder_cb(IntoEventCallback, into_event_callback, ())]
    pub on_reset: Option<Callback<()>>,
}

impl Default for ResetButton {
    fn default() -> Self {
        Self::new()
    }
}

impl ResetButton {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }
}

pub enum Msg {
    Reset,
    FormCtxUpdate(FormContext),
    FormCtxDataChange,
}

#[doc(hidden)]
pub struct PwtResetButton {
    form_ctx: Option<FormContext>,
    _form_ctx_handle: Option<ContextHandle<FormContext>>,
    _form_ctx_observer: Option<FormContextObserver>,
    on_form_data_change: Callback<FormContext>,
}

impl Component for PwtResetButton {
    type Message = Msg;
    type Properties = ResetButton;

    fn create(ctx: &Context<Self>) -> Self {
        let on_form_ctx_change = Callback::from({
            let link = ctx.link().clone();
            move |form_ctx: FormContext| link.send_message(Msg::FormCtxUpdate(form_ctx))
        });

        let on_form_data_change = Callback::from({
            let link = ctx.link().clone();
            move |_form_ctx: FormContext| link.send_message(Msg::FormCtxDataChange)
        });

        let mut _form_ctx_handle = None;
        let mut _form_ctx_observer = None;
        let mut form_ctx = None;
        if let Some((form, handle)) = ctx.link().context::<FormContext>(on_form_ctx_change) {
            _form_ctx_handle = Some(handle);
            _form_ctx_observer = Some(form.add_listener(on_form_data_change.clone()));
            form_ctx = Some(form);
        }

        Self {
            on_form_data_change,
            _form_ctx_handle,
            _form_ctx_observer,
            form_ctx,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::FormCtxUpdate(form_ctx) => {
                self._form_ctx_observer =
                    Some(form_ctx.add_listener(self.on_form_data_change.clone()));
                self.form_ctx = Some(form_ctx);
                true
            }
            Msg::FormCtxDataChange => true,
            Msg::Reset => {
                if let Some(form_ctx) = &self.form_ctx {
                    form_ctx.write().reset_form();
                }
                if let Some(on_reset) = &props.on_reset {
                    on_reset.emit(());
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let form_dirty = match &self.form_ctx {
            Some(form_ctx) => form_ctx.read().is_dirty(),
            None => true,
        };

        let reset = ctx.link().callback({
            move |e: MouseEvent| {
                let event = e.unchecked_into::<Event>();
                event.prevent_default(); // prevent reload
                Msg::Reset
            }
        });

        let builder = if let Some(text) = &props.text {
            Button::new(text)
        } else {
            Button::new(tr!("Reset"))
        };
        builder
            .with_std_props(props.as_std_props())
            .disabled(!form_dirty)
            .onclick(reset)
            .into()
    }
}
