mod form_context;
pub use form_context::{delete_empty_values, FieldOptions, FormContext};

mod field;
pub use field::Field;

mod input;
pub use input::Input;

mod validate;
pub use validate::ValidateFn;

mod reset;
pub use reset::{Reset, PwtReset};

mod submit;
pub use submit::{Submit, PwtSubmit};

mod checkbox;
pub use checkbox::{Checkbox, PwtCheckbox};

mod combobox;
pub use combobox::{Combobox, PwtCombobox};

mod selector;
pub use selector::{CreatePickerArgs, Selector, PwtSelector};

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
        form_context_provider(self.form_ctx.clone(), content)
    }
}

impl Into<VNode> for Form {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtForm>(Rc::new(self), None);
        VNode::from(comp)
    }
}

pub fn form_context_provider(form_ctx: FormContext, content: impl Into<VNode>) -> Html {
    html!{
        <ContextProvider<FormContext> context={form_ctx}>
            <form novalidate=true>{content}</form>
        </ContextProvider<FormContext>>
    }
}
