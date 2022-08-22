use wasm_bindgen::JsCast;

use std::rc::Rc;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};
use yew::html::{IntoEventCallback, IntoPropValue};

use super::FormContext;

#[derive(Clone, PartialEq, Properties)]
pub struct Submit {
    #[prop_or_default]
    pub disabled: bool,

    pub on_submit: Option<Callback<()>>,

    #[prop_or(AttrValue::Static("Submit"))]
    pub text: AttrValue,
}

impl Submit {

    pub fn new() -> Self {
        yew::props!(Submit {})
    }

    pub fn text(mut self, text: impl IntoPropValue<AttrValue>) -> Self {
        self.set_text(text);
        self
    }

    pub fn set_text(&mut self, text: impl IntoPropValue<AttrValue>) {
        self.text = text.into_prop_value();
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.set_disabled(disabled);
        self
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    pub fn on_submit(mut self, cb: impl IntoEventCallback<()>) -> Self {
        self.on_submit = cb.into_event_callback();
        self
    }
}

pub enum Msg {
    Submit,
    FormCtxUpdate(FormContext),
}

pub struct PwtSubmit {
    form_valid: bool,
    form_ctx: Option<FormContext>,
    _form_ctx_handle: Option<ContextHandle<FormContext>>,
}

impl Component for PwtSubmit {
    type Message = Msg;
    type Properties = Submit;

    fn create(ctx: &Context<Self>) -> Self {
        let mut form_valid = true;

        let on_form_ctx_change = Callback::from({
            let link = ctx.link().clone();
            move |form_ctx: FormContext| link.send_message(Msg::FormCtxUpdate(form_ctx))
        });

        let mut _form_ctx_handle = None;
        let mut form_ctx = None;
        if let Some((form, handle)) = ctx.link().context::<FormContext>(on_form_ctx_change) {
            form_valid = form.valid();
            form_ctx = Some(form);
            _form_ctx_handle = Some(handle);
        }

        Self {
            _form_ctx_handle,
            form_ctx,
            form_valid,
        }

    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::FormCtxUpdate(form_ctx) => {
                let form_valid = form_ctx.valid();
                if self.form_valid == form_valid { return false; }
                self.form_valid = form_valid;
                self.form_ctx = Some(form_ctx);
                true
            }
            Msg::Submit => {
                if let Some(on_submit) = &props.on_submit {
                    on_submit.emit(());
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

        html!{
            <button
                type="button" // Note: important, as we do not want type=submit behavior
                onclick={submit}
                class="pwt-button primary"
                disabled={!self.form_valid || props.disabled}>{&props.text}
            </button>
        }
    }
}

impl Into<VNode> for Submit {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtSubmit>(Rc::new(self), None);
        VNode::from(comp)
    }
}
