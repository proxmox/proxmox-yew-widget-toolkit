use wasm_bindgen::JsCast;

use std::rc::Rc;
use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use super::{FormContext, FormContextObserver};

/// Submit button.
///
/// The button automatically listens for [FormContext] changes, and
/// enables the button only if the form is valid and dirty (contains
/// modified data).
#[derive(Clone, PartialEq, Properties)]
pub struct SubmitButton {
    /// Button disabled flag.
    #[prop_or_default]
    pub disabled: bool,

    /// Submit button press callback.
    #[prop_or_default]
    pub on_submit: Option<Callback<FormContext>>,

    /// Button text (default "Submit").
    #[prop_or(AttrValue::Static("Submit"))]
    pub text: AttrValue,

    /// CSS class
    #[prop_or_default]
    pub class: Classes,
}

impl SubmitButton {
    /// Createa new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to set the button text.
    pub fn text(mut self, text: impl IntoPropValue<AttrValue>) -> Self {
        self.set_text(text);
        self
    }

    /// Method to set the button text.
    pub fn set_text(&mut self, text: impl IntoPropValue<AttrValue>) {
        self.text = text.into_prop_value();
    }

    /// Builder style method to set the disabled flag.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.set_disabled(disabled);
        self
    }

    /// Method to set the disabled flag.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Builder style method to add a html class
    pub fn class(mut self, class: impl Into<Classes>) -> Self {
        self.add_class(class);
        self
    }

    /// Method to add a html class
    pub fn add_class(&mut self, class: impl Into<Classes>) {
        self.class.push(class);
    }

    /// Builder style method to set the button press callback.
    pub fn on_submit(mut self, cb: impl IntoEventCallback<FormContext>) -> Self {
        self.on_submit = cb.into_event_callback();
        self
    }
}

pub enum Msg {
    Submit,
    FormCtxUpdate(FormContext),
    FormCtxDataChange,
}

#[doc(hidden)]
pub struct PwtSubmitButton {
    form_valid: bool,
    form_dirty: bool,
    form_ctx: Option<FormContext>,
    _form_ctx_handle: Option<ContextHandle<FormContext>>,
    _form_ctx_observer: Option<FormContextObserver>,
    on_form_data_change: Callback<FormContext>,
}

impl Component for PwtSubmitButton {
    type Message = Msg;
    type Properties = SubmitButton;

    fn create(ctx: &Context<Self>) -> Self {
        let mut form_valid = true;
        let mut form_dirty = false;

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
            form_valid = form.read().is_valid();
            form_dirty = form.read().is_dirty();
            _form_ctx_handle = Some(handle);
            _form_ctx_observer = Some(form.add_listener(on_form_data_change.clone()));
            form_ctx = Some(form);
        }

        Self {
            on_form_data_change,
            _form_ctx_handle,
            _form_ctx_observer,
            form_ctx,
            form_valid,
            form_dirty,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::FormCtxUpdate(form_ctx) => {
                self._form_ctx_observer =
                    Some(form_ctx.add_listener(self.on_form_data_change.clone()));
                self.form_dirty = form_ctx.read().is_dirty();
                self.form_valid = form_ctx.read().is_valid();
                self.form_ctx = Some(form_ctx);
                true
            }
            Msg::FormCtxDataChange => {
                let (form_dirty, form_valid) = match &self.form_ctx {
                    Some(form_ctx) => (form_ctx.read().is_dirty(), form_ctx.read().is_valid()),
                    None => (false, false),
                };
                if self.form_dirty == form_dirty && self.form_valid == form_valid {
                    return false;
                }
                self.form_valid = form_valid;
                self.form_dirty = form_dirty;
                true
            }
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

        let submit = ctx.link().callback({
            move |e: MouseEvent| {
                let event = e.unchecked_into::<Event>();
                event.prevent_default(); // prevent reload
                Msg::Submit
            }
        });

        let class = classes!(
            "pwt-button",
            props.class.clone(),
        );

        html! {
            <button
                type="submit"
                onclick={submit}
                {class}
                disabled={!self.form_valid || props.disabled || !self.form_dirty}>{&props.text}
            </button>
        }
    }
}

impl Into<VNode> for SubmitButton {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtSubmitButton>(Rc::new(self), None);
        VNode::from(comp)
    }
}
