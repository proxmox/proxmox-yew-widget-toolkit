use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::{Listeners, VList, VTag};

use pwt_macros::widget;

use crate::prelude::*;
use super::focus::{focus_next_tabable, init_roving_tabindex};

#[widget(PwtToolbar, @element, @container)]
#[derive(Properties, PartialEq, Clone)]
pub struct Toolbar {}

impl Toolbar {

    pub fn new() -> Self {
        yew::props!(Toolbar {})
    }

    pub fn with_spacer(mut self) -> Self {
        self.add_spacer();
        self
    }

    pub fn add_spacer(&mut self) {
        self.add_child(html!{<div class="pwt-user-select-none">{"|"}</div>});
    }

    pub fn with_flex_spacer(mut self) -> Self {
        self.add_flex_spacer();
        self
    }

    pub fn add_flex_spacer(&mut self) {
        self.add_child(html!{<div class="pwt-flex-fill"/>});
    }
}

#[doc(hidden)]
pub struct PwtToolbar {
    inner_ref: NodeRef,
}

impl Component for PwtToolbar {
    type Message = ();
    type Properties = Toolbar;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            inner_ref: NodeRef::default(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let inner_ref =  self.inner_ref.clone();

        let props = ctx.props()
            .clone()
            .onkeydown(move |event: KeyboardEvent| {
                match event.key_code() {
                    39 => { // left
                        focus_next_tabable(&inner_ref, false, true);
                    }
                    37 => { // right
                        focus_next_tabable(&inner_ref, true, true);
                    }
                    _ => return,
                }
                event.prevent_default();
            });

        // Note: use nested div for better overflow control

        let attributes = props.std_props.cumulate_attributes(Some("pwt-toolbar pwt-p-2"));

        let listeners = Listeners::Pending(
            props.listeners.listeners.into_boxed_slice()
        );

        let children = VList::with_children(props.children, None);

        let inner_class = classes!{
            "pwt-d-flex",
            "pwt-gap-2",
            "pwt-align-items-center",
            "pwt-overflow-hidden",
        };

        let inner = html!{ <div ref={self.inner_ref.clone()} class={inner_class}>{children}</div> };

        VTag::__new_other(
            Cow::Borrowed("div"),
            props.std_props.node_ref,
            props.std_props.key,
            attributes,
            listeners,
            VList::with_children(vec![inner], None),
        ).into()
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            init_roving_tabindex(&self.inner_ref);
        }
    }

}
