use std::borrow::Cow;

use gloo_timers::callback::Timeout;

use yew::prelude::*;
use yew::virtual_dom::{Listeners, VList, VTag};

use pwt_macros::widget;

use crate::prelude::*;
use super::focus::{roving_tabindex_next, init_roving_tabindex, update_roving_tabindex};

/// Horizontal container for buttons with roving tabindex.
///
/// This container uses a CSS flexbox layout, and implements a [roving
/// tabindex](https://developer.mozilla.org/en-US/docs/Web/Accessibility/Keyboard-navigable_JavaScript_widgets)
/// among children having a tabindex attribute.
///
/// # Note
///
/// Avoid including controls whose operation requires left/right arrow
/// keys used for toolbar navigation.
///
/// See: <https://www.w3.org/WAI/ARIA/apg/patterns/toolbar/>.
///
/// # Keyboard bindings
///
/// * `Tab` and `Shift Tab`: Move focus into and out of the toolbar.
///
/// * `Right Arrow`: Moves focus one cell to the right. If focus is on
/// the last element, focus the first element.
///
/// * `Left Arrow`: Moves focus one cell to the left. If focus is on
/// the firs element, focus the last element.
#[widget(pwt=crate, comp=PwtToolbar, @element, @container)]
#[derive(Properties, PartialEq, Clone)]
pub struct Toolbar {}

impl Toolbar {

    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Toolbar {})
    }

    /// Builder style method to add a spacer.
    pub fn with_spacer(mut self) -> Self {
        self.add_spacer();
        self
    }

    /// Method to add a spacer.
    ///
    /// Spacers separate elements by a simple vertical rule.
    pub fn add_spacer(&mut self) {
        self.add_child(html!{<div aria-hidden="true" class="pwt-align-self-stretch pwt-vertical-rule"/>});
    }

    /// Builder style method to add a flex spacer.
    pub fn with_flex_spacer(mut self) -> Self {
        self.add_flex_spacer();
        self
    }

    /// Method to add a flex spacer.
    ///
    /// Flex spacers are empty cells filling the remainig space.
    pub fn add_flex_spacer(&mut self) {
        self.add_child(html!{<div aria-hidden="true" class="pwt-flex-fill"/>});
    }
}

pub enum Msg {
    FocusChange(bool),
    DelayedFocusChange(bool),
}

#[doc(hidden)]
pub struct PwtToolbar {
    inner_ref: NodeRef,
    timeout: Option<Timeout>,
}

impl Component for PwtToolbar {
    type Message = Msg;
    type Properties = Toolbar;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            inner_ref: NodeRef::default(),
            timeout: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        //let props = ctx.props();
        match msg {
            Msg::FocusChange(has_focus) => {
                let link = ctx.link().clone();
                self.timeout = Some(Timeout::new(1, move || {
                    link.send_message(Msg::DelayedFocusChange(has_focus));
                }));
                false
            }
            Msg::DelayedFocusChange(has_focus) => {
                if has_focus {
                    update_roving_tabindex(&self.inner_ref);
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let inner_ref =  self.inner_ref.clone();

        let props = ctx.props()
            .clone()
            .onfocusin(ctx.link().callback(|_| Msg::FocusChange(true)))
            .onfocusout(ctx.link().callback(|_| Msg::FocusChange(false)))
            .onkeydown(move |event: KeyboardEvent| {
                match event.key_code() {
                    39 => { // left
                        roving_tabindex_next(&inner_ref, false, true);
                    }
                    37 => { // right
                        roving_tabindex_next(&inner_ref, true, true);
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
