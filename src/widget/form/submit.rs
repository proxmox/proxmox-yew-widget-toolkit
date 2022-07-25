use wasm_bindgen::JsCast;

use std::rc::Rc;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use crate::state::{ButtonFormRef, FormState};


#[derive(Clone, PartialEq, Properties)]
pub struct Submit {
    pub form_ref: Option<ButtonFormRef>,

    #[prop_or_default]
    pub disabled: bool,

    pub onsubmit: Option<Callback<()>>,

    #[prop_or(String::from("Submit"))]
    pub text: String,
}

impl Submit {

    pub fn new() -> Self {
        yew::props!(Submit {})
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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

    pub fn onsubmit(mut self, callback: Callback<()>) -> Self {
        self.onsubmit = Some(callback);
        self
    }
}

pub struct PwtSubmit;

impl Component for PwtSubmit {
    type Message = ();
    type Properties = Submit;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {

        let props = ctx.props();

        let form_valid = match &props.form_ref {
            Some(form_ref) => form_ref.form.valid(),
            None => true,
        };

        let submit = ctx.link().callback({
            let onsubmit = props.onsubmit.clone();
            move |e: MouseEvent| {
                let event = e.unchecked_into::<Event>();
                event.prevent_default(); // prevent reload
                if let Some(onsubmit) = &onsubmit {
                    onsubmit.emit(());
                }
            }
        });

        html!{
            <button
                type="submit"
                onclick={submit}
                class="pwt-button primary"
                disabled={!form_valid || props.disabled}>{&props.text}
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
