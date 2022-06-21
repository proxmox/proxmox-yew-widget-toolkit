use std::rc::Rc;

use serde_json::json;
use wasm_bindgen::JsValue;
use gloo_timers::callback::Timeout;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};
use yew::html::IntoPropValue;

#[derive(Properties, PartialEq, Clone)]
pub struct Tooltip {
    pub tip: Option<VNode>,
    pub children: Vec<VNode>,
}

impl Tooltip {

    pub fn new() -> Self {
        yew::props!{Self {
            children: Vec::new(),
        }}
    }

    /// Builder style method to set the tooltip
    pub fn tip(mut self, tip: impl IntoPropValue<Option<VNode>>) -> Self {
        self.set_tip(tip);
        self
    }

    /// Method to set the tooltip
    pub fn set_tip(&mut self, tip: impl IntoPropValue<Option<VNode>>) {
        self.tip = tip.into_prop_value();
    }

    pub fn with_child(mut self, child: impl Into<VNode>) -> Self {
        self.add_child(child);
        self
    }

    pub fn add_child(&mut self, child: impl Into<VNode>) {
        self.children.push(child.into());
    }
}

pub enum Msg {
    RealShow,
    Show,
    Hide,
}

pub struct PwtTooltip {
    content_ref: NodeRef,
    tooltip_ref: NodeRef,
    popper: Option<JsValue>,
    show: bool,
    timeout: Option<Timeout>,
}

impl Component for PwtTooltip {
    type Message = Msg;
    type Properties = Tooltip;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            content_ref: NodeRef::default(),
            tooltip_ref: NodeRef::default(),
            popper: None,
            show: false,
            timeout: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::RealShow => {
                self.show = true;
            }
            Msg::Show => {
                let link = ctx.link().clone();
                self.timeout = Some(Timeout::new(1_000, move || {
                    link.send_message(Msg::RealShow);
                }));
            }
            Msg::Hide => {
                if let Some(timeout) = self.timeout.take() {
                    timeout.cancel();
                }
                self.show = false;
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let onmouseenter = ctx.link().callback(|_| Msg::Show);
        let onmouseleave = ctx.link().callback(|_| Msg::Hide);

        let show_tooltip = self.show && ctx.props().tip.is_some();

        let onfocus = ctx.link().callback(|_| Msg::Show);
        let onblur = ctx.link().callback(|_| Msg::Hide);
        let onkeydown = Callback::from({
            let link = ctx.link().clone();
            move |event: KeyboardEvent| {
                if show_tooltip && event.key_code() == 27 { // ESC
                    link.send_message(Msg::Hide);
                    event.prevent_default();
                }
            }
        });

        let data_show = show_tooltip.then(|| "");
        html!{
            <>
                <div ref={self.content_ref.clone()} {onmouseenter} {onmouseleave} {onfocus} {onblur} {onkeydown}>{props.children.clone()}</div>
                <div role="tooltip" aria-live="polite" class="tooltip" ref={self.tooltip_ref.clone()} data-show={data_show}>
                if let Some(tip) = &props.tip { {tip.clone()} }
                <div class="tooltip-arrow" data-popper-arrow=""></div>
            </div>
            </>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
           let opts = json!({
               "placement": "bottom-start",
               "strategy": "fixed",
               "modifiers": [
                   {
                       "name": "arrow",
                   },
                   {
                       "name": "preventOverflow",
                       "options": {
                           "mainAxis": true, // true by default
                           "altAxis": false, // false by default
                        },
                   },
                   {
                       "name": "flip",
                       "options": {
                           "fallbackPlacements": ["top-start", "right-start", "left-start"],
                       },
                   },
                   {
                       "name": "offset",
                       "options": {
                           "offset": [4, 4],
                       },
                   },
              ],
           });

            let opts = JsValue::from_serde(&opts).unwrap();

            if let Some(content_node) = self.content_ref.get() {
                if let Some(tooltip_node) = self.tooltip_ref.get() {
                    self.popper = Some(crate::create_popper(content_node, tooltip_node, &opts));
                }
            }
        }

        if let Some(popper) = &self.popper {
            crate::update_popper(popper);
        }
    }
}

impl Into<VNode> for Tooltip {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtTooltip>(Rc::new(self), NodeRef::default(), None);
        VNode::from(comp)
    }
}
