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
    #[prop_or_default]
    pub tip: Option<Html>,

    #[prop_or_default]
    rich: bool,
}

impl Tooltip {
    pub fn new(content: impl Into<VNode>) -> Self {
        yew::props!(Self {
            content: content.into()
        })
    }

    /// Builder style method to set the tooltip
    pub fn tip(mut self, tip: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_tip(tip);
        self
    }

    /// Method to set the tooltip
    pub fn set_tip(&mut self, tip: impl IntoPropValue<Option<AttrValue>>) {
        self.rich = false;
        self.tip = tip.into_prop_value().map(|tip| html! {{tip}});
    }

    /// Builder style method to set the tooltip (rich style)
    pub fn rich_tip(mut self, tip: impl Into<Html>) -> Self {
        self.set_rich_tip(tip);
        self
    }

    /// Method to set the tooltip (rich style)
    pub fn set_rich_tip(&mut self, tip: impl Into<Html>) {
        self.rich = true;
        self.tip = Some(tip.into());
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

        let content = Container::new()
            .class("pwt-flex-fill-first-child")
            .class("pwt-d-flex")
            .with_std_props(&props.std_props)
            .listeners(&props.listeners)
            .with_child(props.content.clone())
            .onmouseenter(ctx.link().callback(|_| Msg::Show))
            .onmouseleave(ctx.link().callback(|_| Msg::Hide))
            .onfocus(ctx.link().callback(|_| Msg::Show))
            .onblur(ctx.link().callback(|_| Msg::Hide))
            .onkeydown(Callback::from({
                let link = ctx.link().clone();
                move |event: KeyboardEvent| {
                    if show_tooltip && event.key() == "Escape" {
                        link.send_message(Msg::Hide);
                        event.prevent_default();
                    }
                }
            }));

        let tip = show_tooltip.then_some(
            Container::new()
                .node_ref(self.tooltip_ref.clone())
                .attribute("role", "tooltip")
                .attribute("aria-live", "polite")
                .attribute("data-show", show_tooltip.then_some(""))
                .class("pwt-tooltip")
                .class(props.rich.then_some("pwt-tooltip-rich"))
                .onmouseenter(ctx.link().callback(|_| Msg::Enter))
                .onmouseleave(ctx.link().callback(|_| Msg::Leave))
                .with_optional_child(props.tip.clone()),
        );

        Container::new()
            .with_child(content)
            .with_optional_child(tip)
            .into()
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
                    .with_fallback_placement(
                        Point::BottomStart,
                        Point::TopStart,
                        GrowDirection::StartEnd,
                    )
                    .with_fallback_placement(
                        Point::TopStart,
                        Point::BottomStart,
                        GrowDirection::StartEnd,
                    )
                    .with_fallback_placement(
                        Point::TopEnd,
                        Point::TopStart,
                        GrowDirection::TopBottom,
                    )
                    .with_fallback_placement(
                        Point::TopStart,
                        Point::TopEnd,
                        GrowDirection::TopBottom,
                    )
                    .offset(4.0, 4.0),
            );
        }

        if self.show && ctx.props().tip.is_some() {
            if let Some(content_node) = props.std_props.node_ref.get() {
                if let Some(tooltip_node) = self.tooltip_ref.get() {
                    let _ = align_to(content_node, tooltip_node, self.align_options.clone());
                }
            }
        }
    }
}
