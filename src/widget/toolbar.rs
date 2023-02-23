use std::borrow::Cow;

use gloo_timers::callback::Timeout;

use yew::prelude::*;
use yew::virtual_dom::{Listeners, VList, VTag};

use pwt_macros::widget;

use super::dom::element_direction_rtl;
use super::focus::{init_roving_tabindex, roving_tabindex_next, update_roving_tabindex};
use super::{IntoOptionalMiniScrollMode, MiniScroll, MiniScrollMode};
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
pub struct Toolbar {
    /// Use [MiniScroll] to allow scrolling.
    #[prop_or_default]
    pub scroll_mode: Option<MiniScrollMode>,
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
        self.add_child(
            html! {<div aria-hidden="true" class="pwt-align-self-stretch pwt-vertical-rule"/>},
        );
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
        self.add_child(html! {<div aria-hidden="true" class="pwt-flex-fill"/>});
    }

    /// Builder style method to set the scroll mode.
    pub fn scroll_mode(mut self, scroll_mode: impl IntoOptionalMiniScrollMode) -> Self {
        self.set_scroll_mode(scroll_mode);
        self
    }

    /// Method to set the scroll mode.
    pub fn set_scroll_mode(&mut self, scroll_mode: impl IntoOptionalMiniScrollMode) {
        self.scroll_mode = scroll_mode.into_optional_mini_scroll_mode();
    }
}

pub enum Msg {
    FocusChange(bool),
    DelayedFocusChange(bool),
    Scroll(bool),
}

#[doc(hidden)]
pub struct PwtToolbar {
    inner_ref: NodeRef,
    timeout: Option<Timeout>,
    rtl: Option<bool>,
}

impl Component for PwtToolbar {
    type Message = Msg;
    type Properties = Toolbar;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            rtl: None,
            inner_ref: NodeRef::default(),
            timeout: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
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
                self.rtl = element_direction_rtl(&props.std_props.node_ref);
                true
            }
            Msg::Scroll(left) => {
                let el = match props.std_props.node_ref.cast::<web_sys::HtmlElement>() {
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
            .onfocusin(ctx.link().callback(|_| Msg::FocusChange(true)))
            .onfocusout(ctx.link().callback(|_| Msg::FocusChange(false)))
            .onwheel({
                let link = ctx.link().clone();
                move |event: WheelEvent| {
                    event.prevent_default();
                    link.send_message(Msg::Scroll(event.delta_y() > 0.0))
                }
            })
            .onkeydown(move |event: KeyboardEvent| {
                match event.key_code() {
                    39 => {
                        // left
                        roving_tabindex_next(&inner_ref, rtl, true);
                    }
                    37 => {
                        // right
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
            props.std_props.node_ref,
            props.std_props.key,
            attributes,
            listeners,
            VList::with_children(vec![inner], None),
        )
        .into()
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            init_roving_tabindex(&self.inner_ref);
        }
    }
}
