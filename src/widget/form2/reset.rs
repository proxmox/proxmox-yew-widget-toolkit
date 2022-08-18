use wasm_bindgen::JsCast;

use std::rc::Rc;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};
use yew::html::{IntoEventCallback, IntoPropValue};

use super::FormContext;

#[derive(Clone, PartialEq, Properties)]
pub struct Reset {
    #[prop_or(AttrValue::Static("Reset"))]
    pub text: AttrValue,
    pub on_reset: Option<Callback<()>>,
}

impl Reset {

    pub fn new() -> Self {
        yew::props!(Reset {})
    }

    pub fn text(mut self, text: impl IntoPropValue<AttrValue>) -> Self {
        self.set_text(text);
        self
    }

    pub fn set_text(&mut self, text: impl IntoPropValue<AttrValue>) {
        self.text = text.into_prop_value();
    }

    pub fn on_reset(mut self, cb: impl IntoEventCallback<()>) -> Self {
        self.on_reset = cb.into_event_callback();
        self
    }
}

pub enum Msg {
    Reset,
    FormCtxUpdate(FormContext),
}

pub struct PwtReset {
    form_dirty: bool,
    form_ctx: Option<FormContext>,
    _form_ctx_handle: Option<ContextHandle<FormContext>>,
}

impl Component for PwtReset {
    type Message = Msg;
    type Properties = Reset;

    fn create(ctx: &Context<Self>) -> Self {
        let mut form_dirty = true;
        
        let on_form_ctx_change = Callback::from({
            let link = ctx.link().clone();
            move |form_ctx: FormContext| link.send_message(Msg::FormCtxUpdate(form_ctx))
        });
        
        let mut _form_ctx_handle = None;
        let mut form_ctx = None;
        if let Some((form, handle)) = ctx.link().context::<FormContext>(on_form_ctx_change) {
            form_dirty = form.dirty();
            form_ctx = Some(form);
            _form_ctx_handle = Some(handle);
        }
  
        Self {
            _form_ctx_handle,
            form_ctx,
            form_dirty
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::FormCtxUpdate(form_ctx) => {
                let form_dirty = form_ctx.dirty();
                if self.form_dirty == form_dirty { return false; }
                
                self.form_dirty = form_dirty;
                self.form_ctx = Some(form_ctx);
                true
            }
            Msg::Reset => {
                if let Some(form_ctx) = &self.form_ctx {
                    form_ctx.reset_form();
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

        let form_dirty = self.form_dirty;

        let reset = ctx.link().callback({
            move |e: MouseEvent| {
                let event = e.unchecked_into::<Event>();
                event.prevent_default(); // prevent reload
                Msg::Reset
            }
        });

        html!{
            <button onclick={reset} class="pwt-button" disabled={!form_dirty}>{&props.text}</button>
        }
    }
}

impl Into<VNode> for Reset {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtReset>(Rc::new(self), None);
        VNode::from(comp)
    }
}
