use std::rc::Rc;

use web_sys::HtmlElement;
use wasm_bindgen::JsCast;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::IntoPropValue;

use crate::widget::Fa;

#[derive(Clone, PartialEq, Properties)]
pub struct Mask {
    #[prop_or_default]
    node_ref: NodeRef,
    pub key: Option<Key>,

    #[prop_or_default]
    pub visible: bool,
    #[prop_or_default]
    pub children: Vec<VNode>,
    #[prop_or_default]
    pub text: String,
    #[prop_or_default]
    pub form_wrapper: bool,
}

impl Mask {

    pub fn new() -> Self {
        yew::props!(Mask {})
    }

    /// Builder style method to set the yew `node_ref`
    pub fn node_ref(mut self, node_ref: ::yew::html::NodeRef) -> Self {
        self.set_node_ref(node_ref);
        self
    }

    /// Method to set the yew `node_ref`
    pub fn set_node_ref(&mut self, node_ref: ::yew::html::NodeRef) {
        self.node_ref = node_ref;
    }

    /// Builder style method to set the yew `key` property
    pub fn key(mut self, key: impl IntoPropValue<Option<Key>>) -> Self {
        self.set_key(key);
        self
    }

    /// Method to set the yew `key` property
    pub fn set_key(&mut self, key: impl IntoPropValue<Option<Key>>) {
        self.key = key.into_prop_value();
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn form_wrapper(mut self, form_wrapper: bool) -> Self {
        self.form_wrapper = form_wrapper;
        self
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    pub fn with_child(mut self, child: impl Into<VNode>) -> Self {
        self.add_child(child);
        self
    }

    pub fn with_optional_child(
        mut self,
        child: Option<impl Into<VNode>>,
    ) -> Self {
        if let Some(child) = child {
            self.add_child(child);
        } else {
            // important for VDOM diff
            // self.add_child(html!{}); // this can still change child order,
            // so we add a real DOM element for now
            self.add_child(html!{<div class="pwt-d-none"/>});
        }
        self
    }

    pub fn add_child(&mut self, child: impl Into<VNode>) {
        self.children.push(child.into());
    }
}

pub struct PwtMask {
    last_visible: bool, //change tracking
    last_active: Option<web_sys::HtmlElement>, // last focused element
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

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            last_active: None,
            last_visible: ctx.props().visible,
        }
    }

     fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
         if first_render {
             let props = ctx.props();
             if props.visible {
                 self.save_focused_element(&props.node_ref);
             }
         }
     }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        let props = ctx.props();
        let visible = props.visible;
        if self.last_visible != visible {
            self.last_visible = visible;
            if visible {
                self.save_focused_element(&props.node_ref);
            } else {
                self.restore_focused_element();
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let text = if props.text.is_empty() {
            "Loading..."
        } else {
            &props.text
        };

        html!{
            <div class="pwt-fit" ref={props.node_ref.clone()} style="position:relative;">
                if props.form_wrapper {
                    <form class="pwt-fit" novalidate=true>{props.children.clone()}</form>
                } else {
                    {props.children.clone()}
                }
            if props.visible {
                <div class="pwt-load-mask">
                    <div class="pwt-load-mask-inner">
                        {Fa::new("spinner").pulse()}
                        {text}
                    </div>
                </div>
            }
            </div>
        }
    }
}

impl Into<VNode> for Mask {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtMask>(Rc::new(self), NodeRef::default(), key);
        VNode::from(comp)
    }
}
