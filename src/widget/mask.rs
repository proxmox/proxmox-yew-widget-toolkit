use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use yew::html::IntoPropValue;
use yew::virtual_dom::VNode;

use crate::prelude::*;
use crate::widget::Container;

use pwt_macros::{builder, widget};

/// Container which optionaly masks its content.
#[widget(pwt=crate, comp=PwtMask, @element)]
#[derive(Default, Debug, Clone, PartialEq, Properties)]
#[builder]
pub struct Mask {
    content: VNode,

    /// Flag to show/hide the mask
    #[prop_or_default]
    #[builder]
    pub visible: bool,

    /// Mask text. Defaults to "Loading...".
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub text: Option<AttrValue>,
}

impl Mask {
    /// Create a new instance.
    pub fn new(content: impl Into<VNode>) -> Self {
        yew::props!(Mask {
            content: content.into()
        })
    }
}

#[doc(hidden)]
pub struct PwtMask {
    last_active: Option<web_sys::HtmlElement>, // last focused element
    node_ref: NodeRef,
}

impl PwtMask {
    fn save_focused_element(&mut self, node_ref: &NodeRef) {
        if let Some(el) = node_ref.cast::<HtmlElement>() {
            if let Ok(Some(focused_el)) = el.query_selector(":focus") {
                if let Ok(focused_el) = focused_el.dyn_into::<HtmlElement>() {
                    let _ = focused_el.blur();
                    self.last_active = Some(focused_el);
                }
            }
        }
    }

    fn restore_focused_element(&mut self) {
        if let Some(el) = self.last_active.take() {
            let _ = el.focus();
        }
    }
}

impl Component for PwtMask {
    type Message = ();
    type Properties = Mask;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            last_active: None,
            node_ref: NodeRef::default(),
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let props = ctx.props();
            if props.visible {
                self.save_focused_element(&self.node_ref.clone());
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        let visible = props.visible;
        if old_props.visible != visible {
            if visible {
                self.save_focused_element(&self.node_ref.clone());
            } else {
                self.restore_focused_element();
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let text = match props.text.as_deref() {
            None => "Loading...",
            Some(text) => text,
        };

        let mut mask = Container::new()
            .class("pwt-load-mask")
            .class(props.visible.then_some("visible"));

        if props.visible {
            mask.add_child(
                Container::new()
                    .class("pwt-load-mask-inner")
                    .with_child(html! {<i class={"pwt-loading-icon"} />})
                    .with_child(text),
            )
        }

        yew::props!(Container {
            std_props: props.std_props.clone(),
            listeners: props.listeners.clone(),
        })
        .class("pwt-flex-fill-first-child")
        .class("pwt-d-flex")
        .class("pwt-position-relative")
        .with_child(props.content.clone())
        .with_child(mask)
        .into()
    }
}
