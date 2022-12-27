use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::{Listeners, VList, VNode, VTag};
use yew::virtual_dom::ApplyAttributeAs;

use super::FormContext;

use pwt_macros::widget;

/// Html form with [ContextProvider](yew::context::ContextProvider)<[FormContext]>
///
/// We automatically add the `novalidate` attribute, because our form
/// fields do validation themselves.
///
/// The form creates an empty [FormContext] if you do not provide one.
#[widget(pwt=crate, comp=PwtForm, @element, @container)]
#[derive(Clone, PartialEq, Properties)]
pub struct Form {
    pub form_context: Option<FormContext>,
}

impl Form {
    /// Creates a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    pub fn form_context(mut self, form_context: FormContext) -> Self {
        self.set_form_context(form_context);
        self
    }

    pub fn set_form_context(&mut self, form_context: FormContext) {
        self.form_context = Some(form_context);
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

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let form_ctx = match &props.form_context {
            Some(form_ctx) => form_ctx.clone(),
            None => FormContext::new(),
        };

        Self { form_ctx }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        if props.form_context != old_props.form_context {
            if let Some(form_ctx) = &props.form_context {
                self.form_ctx = form_ctx.clone();
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props().clone();

        let form_ctx = self.form_ctx.clone();

        let mut attributes = props.std_props.cumulate_attributes(None::<&str>);
        let attr_map = attributes.get_mut_index_map();
        attr_map.insert(AttrValue::Static("novalidate"), (AttrValue::Static("true"), ApplyAttributeAs::Attribute));

        let listeners = Listeners::Pending(
            props.listeners.listeners.into_boxed_slice()
        );

        let children = VList::with_children(props.children, None);

        let form = VNode::from(VTag::__new_other(
            Cow::from("form"),
            props.std_props.node_ref,
            None,
            attributes,
            listeners,
            children,
        ));

        html!{
            <ContextProvider<FormContext> context={form_ctx}>{form}</ContextProvider<FormContext>>
        }
    }
}
