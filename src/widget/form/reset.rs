use wasm_bindgen::JsCast;

use std::rc::Rc;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use crate::state::{ButtonFormRef, FormState};

#[derive(Clone, PartialEq, Properties)]
pub struct Reset {
    pub form_ref: Option<ButtonFormRef>,

    pub onreset: Option<Callback<()>>,

    #[prop_or(String::from("Reset"))]
    pub text: String,
}

impl Reset {

    pub fn new() -> Self {
        yew::props!(Reset {})
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    pub fn form(mut self, form: &FormState) -> Self {
        self.form_ref = Some(form.button_ref());
        self
    }

    pub fn form_ref(mut self, form_ref: ButtonFormRef) -> Self {
        self.form_ref = Some(form_ref);
        self
    }
}

pub enum Msg {
    Reset
}

pub struct PwtReset;

impl Component for PwtReset {
    type Message = Msg;
    type Properties = Reset;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::Reset => {
                if let Some(form_ref) = &props.form_ref {
                    form_ref.form.reset_form();
                }
                if let Some(onreset) = &props.onreset {
                    onreset.emit(());
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {

        let props = ctx.props();

        let form_dirty = match &props.form_ref {
            Some(form_ref) => form_ref.dirty,
            None => true,
        };

        let reset = ctx.link().callback({
            move |e: MouseEvent| {
                let event = e.unchecked_into::<Event>();
                event.prevent_default(); // prevent reload
                Msg::Reset
            }
        });

        html!{
            <button
                onclick={reset}
                type="button"
                class="pwt-button"
                disabled={!form_dirty}>{&props.text}
            </button>
        }
    }
}

impl Into<VNode> for Reset {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtReset>(Rc::new(self), None);
        VNode::from(comp)
    }
}
