use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use crate::props::RenderFn;
use crate::state::form::FormContext;

#[derive(Clone, PartialEq, Properties)]
pub struct Form {
    pub renderer: RenderFn<FormContext>,
}

impl Form {

    pub fn new(renderer: impl Into<RenderFn<FormContext>>) -> Self {
        Self {
            renderer: renderer.into(),
        }
    }
}

#[doc(hidden)]
pub enum Msg {
    Update,
}

#[doc(hidden)]
pub struct PwtForm {
    form_ctx: FormContext,
}

impl Component for PwtForm {
    type Message = Msg;
    type Properties = Form;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            form_ctx: FormContext::new(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let content = props.renderer.apply(&self.form_ctx);
        let form_ctx = self.form_ctx.clone();

        html!{
            <ContextProvider<FormContext> context={form_ctx}>
                <form novalidate=true>{content}</form>
            </ContextProvider<FormContext>>
        }
    }
}

impl Into<VNode> for Form {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtForm>(Rc::new(self), None);
        VNode::from(comp)
    }
}
