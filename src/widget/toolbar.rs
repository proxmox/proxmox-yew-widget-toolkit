use std::borrow::Cow;

use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::{Listeners, VList, VTag};

use pwt_macros::{builder, widget};

use super::{MiniScroll, MiniScrollMode};
use crate::dom::element_direction_rtl;
use crate::dom::focus::{
    init_roving_tabindex, roving_tabindex_next, update_roving_tabindex, FocusTracker,
};
use crate::prelude::*;

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
/// the first element, focus the last element.
#[widget(pwt=crate, comp=PwtToolbar, @element, @container)]
#[derive(Properties, PartialEq, Clone)]
#[builder]
pub struct Toolbar {
    /// Use [MiniScroll] to allow scrolling.
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub scroll_mode: Option<MiniScrollMode>,
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}

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
        self.add_child(html! {<div role="none" class="pwt-align-self-stretch pwt-vertical-rule"/>});
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
        self.add_child(html! {<div role="none" class="pwt-flex-fill"/>});
    }
}

pub enum Msg {
    FocusChange(bool),
    Scroll(bool),
}

#[doc(hidden)]
pub struct PwtToolbar {
    node_ref: NodeRef,
    inner_ref: NodeRef,
    focus_tracker: FocusTracker,
    rtl: Option<bool>,
}

impl Component for PwtToolbar {
    type Message = Msg;
    type Properties = Toolbar;

    fn create(ctx: &Context<Self>) -> Self {
        let focus_tracker = FocusTracker::new(ctx.link().callback(Msg::FocusChange));
        Self {
            rtl: None,
            node_ref: NodeRef::default(),
            inner_ref: NodeRef::default(),
            focus_tracker,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::FocusChange(has_focus) => {
                if has_focus {
                    update_roving_tabindex(&self.inner_ref);
                }
                self.rtl = element_direction_rtl(&self.node_ref);
                true
            }
            Msg::Scroll(left) => {
                let el = match self.node_ref.cast::<web_sys::HtmlElement>() {
                    None => return false,
                    Some(el) => el,
                };
                let pos = el.scroll_left();
                if left {
                    el.set_scroll_left(pos + 30);
                } else {
                    el.set_scroll_left(pos - 30);
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let inner_ref = self.inner_ref.clone();
        let rtl = self.rtl.unwrap_or(false);

        let props = ctx
            .props()
            .clone()
            .onfocusin(self.focus_tracker.get_focus_callback(true))
            .onfocusout(self.focus_tracker.get_focus_callback(false))
            .onwheel({
                let link = ctx.link().clone();
                move |event: WheelEvent| {
                    event.prevent_default();
                    link.send_message(Msg::Scroll(event.delta_y() > 0.0))
                }
            })
            .onkeydown(move |event: KeyboardEvent| {
                match event.key().as_str() {
                    "ArrowRight" => {
                        roving_tabindex_next(&inner_ref, rtl, true);
                    }
                    "ArrowLeft" => {
                        roving_tabindex_next(&inner_ref, !rtl, true);
                    }
                    _ => return,
                }
                event.prevent_default();
            });

        // Note: use nested div for better overflow control

        let attributes = props.std_props.cumulate_attributes(Some("pwt-toolbar"));

        let listeners = Listeners::Pending(props.listeners.listeners.into_boxed_slice());

        let children = VList::with_children(props.children, None);

        let mut inner = html! { <div ref={self.inner_ref.clone()} class="pwt-toolbar-content">{children}</div> };

        if let Some(scroll_mode) = props.scroll_mode {
            inner = MiniScroll::new(inner).scroll_mode(scroll_mode).into();
        }

        VTag::__new_other(
            Cow::Borrowed("div"),
            self.node_ref.clone(),
            props.std_props.key,
            attributes,
            listeners,
            VList::with_children(vec![inner], None).into(),
        )
        .into()
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            init_roving_tabindex(&self.inner_ref);
        }
    }
}
