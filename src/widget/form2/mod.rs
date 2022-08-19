mod form_context;
pub use form_context::{FieldOptions, FormContext};

mod field;
pub use field::Field;

mod reset;
pub use reset::{Reset, PwtReset};

mod submit;
pub use submit::{Submit, PwtSubmit};

mod checkbox;
pub use checkbox::{Checkbox, PwtCheckbox};

use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use crate::props::RenderFn;

#[derive(Clone, PartialEq, Properties)]
pub struct Form {
    pub renderer: RenderFn<FormContext>,
}

impl Form {

    pub fn new(renderer: impl 'static + Fn(&FormContext) -> Html) -> Self {
        yew::props!(Self { renderer: RenderFn::new(renderer) })
    }
}

pub enum Msg {
    Update,
}

pub struct PwtForm {
    form_ctx: FormContext,
}

impl Component for PwtForm {
    type Message = Msg;
    type Properties = Form;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            form_ctx: FormContext::new(ctx.link().callback(|()| Msg::Update)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Update => {
                self.form_ctx.context_change_trigger();
                true
            }
        }
    }
    
    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let content = props.renderer.apply(&self.form_ctx);
        
        html!{
            <ContextProvider<FormContext> context={self.form_ctx.clone()}>
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

