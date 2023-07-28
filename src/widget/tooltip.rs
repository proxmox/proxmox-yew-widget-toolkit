use gloo_timers::callback::Timeout;

use yew::html::IntoPropValue;
use yew::virtual_dom::VNode;

use crate::prelude::*;
use crate::widget::align::{align_to, AlignOptions, GrowDirection, Point};
use crate::widget::Container;

use pwt_macros::widget;

#[widget(pwt=crate, comp=PwtTooltip, @element)]
#[derive(Properties, PartialEq, Clone)]
pub struct Tooltip {
    content: VNode,

    /// The tooltip content/message.
    pub tip: Option<VNode>,
    #[prop_or_default]
    pub rich: bool,
}

impl Tooltip {
    pub fn new(content: impl Into<VNode>) -> Self {
        yew::props!(Self { content: content.into() })
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

    /// Builder style method to set the tooltip (rich style)
    pub fn rich_tip(mut self, tip: impl IntoPropValue<Option<VNode>>) -> Self {
        self.set_rich_tip(tip);
        self
    }

    /// Method to set the tooltip (rich style)
    pub fn set_rich_tip(&mut self, tip: impl IntoPropValue<Option<VNode>>) {
        self.rich = true;
        self.tip = tip.into_prop_value();
    }
}

pub enum Msg {
    RealShow,
    RealHide,
    Show,
    Hide,
    Enter,
    Leave,
}

#[doc(hidden)]
pub struct PwtTooltip {
    tooltip_ref: NodeRef,
    align_options: Option<AlignOptions>,
    show: bool,
    hover_tooltip: bool,
    timeout: Option<Timeout>,
}

impl Component for PwtTooltip {
    type Message = Msg;
    type Properties = Tooltip;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            tooltip_ref: NodeRef::default(),
            show: false,
            hover_tooltip: false,
            timeout: None,
            align_options: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Enter => {
                self.hover_tooltip = true;
            }
            Msg::Leave => {
                self.hover_tooltip = false;
                ctx.link().send_message(Msg::Hide);
            }
            Msg::RealShow => {
                self.show = true;
            }
            Msg::RealHide => {
                if !self.hover_tooltip {
                    self.show = false;
                }
            }
            Msg::Show => {
                let link = ctx.link().clone();
                self.timeout = Some(Timeout::new(1_000, move || {
                    link.send_message(Msg::RealShow);
                }));
            }
            Msg::Hide => {
                let link = ctx.link().clone();
                self.timeout = Some(Timeout::new(200, move || {
                    link.send_message(Msg::RealHide);
                }));
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let show_tooltip = self.show && ctx.props().tip.is_some();

        let content =
            Container::form_widget_props(props.std_props.clone(), Some(props.listeners.clone()))
                .class("pwt-flex-fill-first-child")
                .class("pwt-d-flex")
                .with_child(props.content.clone())
                .onmouseenter(ctx.link().callback(|_| Msg::Show))
                .onmouseleave(ctx.link().callback(|_| Msg::Hide))
                .onfocus(ctx.link().callback(|_| Msg::Show))
                .onblur(ctx.link().callback(|_| Msg::Hide))
                .onkeydown(Callback::from({
                    let link = ctx.link().clone();
                    move |event: KeyboardEvent| {
                        if show_tooltip && event.key_code() == 27 {
                            // ESC
                            link.send_message(Msg::Hide);
                            event.prevent_default();
                        }
                    }
                }));

        let tip = Container::new()
            .node_ref(self.tooltip_ref.clone())
            .attribute("role", "tooltip")
            .attribute("aria-live", "polite")
            .attribute("data-show", show_tooltip.then(|| ""))
            .class("pwt-tooltip")
            .class(props.rich.then(|| "pwt-tooltip-rich"))
            .onmouseenter(ctx.link().callback(|_| Msg::Enter))
            .onmouseleave(ctx.link().callback(|_| Msg::Leave))
            .with_optional_child(props.tip.clone());

        html! { <>{content}{tip}</> }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        let props = ctx.props();

        if first_render {
            self.align_options = Some(
                AlignOptions::new(Point::BottomStart, Point::TopStart, GrowDirection::None)
                    .with_fallback_placement(
                        Point::TopStart,
                        Point::BottomStart,
                        GrowDirection::None,
                    )
                    .with_fallback_placement(Point::TopEnd, Point::TopStart, GrowDirection::None)
                    .with_fallback_placement(Point::TopStart, Point::TopEnd, GrowDirection::None)
                    .offset(4.0, 4.0),
            );
        }

        if let Some(content_node) = props.std_props.node_ref.get() {
            if let Some(tooltip_node) = self.tooltip_ref.get() {
                let _ = align_to(content_node, tooltip_node, self.align_options.clone());
            }
        }
    }
}
