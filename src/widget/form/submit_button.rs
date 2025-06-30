use wasm_bindgen::JsCast;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;

use crate::props::{EventSubscriber, WidgetBuilder};
use crate::tr;
use crate::widget::{Button, ButtonType};

use super::{FormContext, FormContextObserver};

use pwt_macros::{builder, widget};

/// Submit button.
///
/// The button automatically listens for [FormContext] changes, and
/// enables the button only if the form is valid and dirty (contains
/// modified data).
#[widget(pwt=crate, comp=crate::widget::form::PwtSubmitButton, @element)]
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct SubmitButton {
    /// Button disabled flag.
    #[prop_or_default]
    #[builder]
    pub disabled: bool,

    /// Submit button press callback.
    #[prop_or_default]
    #[builder_cb(IntoEventCallback, into_event_callback, FormContext)]
    pub on_submit: Option<Callback<FormContext>>,

    /// Button text (default "Submit").
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub text: Option<AttrValue>,

    /// Icon (CSS class).
    #[prop_or_default]
    pub icon_class: Option<Classes>,

    /// Disable submit button if there are no changes.
    #[prop_or(true)]
    #[builder]
    pub check_dirty: bool,
}

impl Default for SubmitButton {
    fn default() -> Self {
        Self::new()
    }
}

impl SubmitButton {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to set the icon CSS class.
    pub fn icon_class(mut self, icon_class: impl Into<Classes>) -> Self {
        self.set_icon_class(icon_class);
        self
    }

    /// Method to set the icon CSS class.
    pub fn set_icon_class(&mut self, icon_class: impl Into<Classes>) {
        self.icon_class = Some(icon_class.into());
    }
}

pub enum Msg {
    Submit,
    FormCtxUpdate(FormContext),
    FormCtxDataChange,
}

#[doc(hidden)]
pub struct PwtSubmitButton {
    form_ctx: Option<FormContext>,
    _form_ctx_handle: Option<ContextHandle<FormContext>>,
    _form_ctx_observer: Option<FormContextObserver>,
    on_form_data_change: Callback<FormContext>,
}

impl Component for PwtSubmitButton {
    type Message = Msg;
    type Properties = SubmitButton;

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
            Msg::Submit => {
                if let Some(on_submit) = &props.on_submit {
                    if let Some(form_ctx) = self.form_ctx.clone() {
                        on_submit.emit(form_ctx);
                    }
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let (form_dirty, form_valid) = match &self.form_ctx {
            Some(form_ctx) => {
                let guard = form_ctx.read();
                (guard.is_dirty(), guard.is_valid())
            }
            None => (true, true),
        };

        let submit = ctx.link().callback({
            move |e: MouseEvent| {
                let event = e.unchecked_into::<Event>();
                event.prevent_default(); // prevent reload
                Msg::Submit
            }
        });

        let disabled = !form_valid || props.disabled || (props.check_dirty && !form_dirty);

        let text = match &props.text {
            Some(text) => text.clone(),
            None => AttrValue::from(tr!("Submit")),
        };

        Button::new(text)
            .with_std_props(props.as_std_props())
            .button_type(ButtonType::Submit)
            .disabled(disabled)
            .icon_class(props.icon_class.clone())
            .onclick(submit)
            .into()
    }
}
