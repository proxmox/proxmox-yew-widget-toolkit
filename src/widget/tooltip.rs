use gloo_timers::callback::Timeout;

use yew::html::IntoPropValue;
use yew::virtual_dom::VNode;

use crate::prelude::*;
use crate::widget::align::{align_to, AlignOptions, GrowDirection, Point};
use crate::widget::Container;

use pwt_macros::widget;

#[widget(pwt=crate, comp=crate::widget::PwtTooltip, @element, @container)]
#[derive(Properties, PartialEq, Clone)]
pub struct Tooltip {
    /// The tooltip content/message.
    pub tip: Option<VNode>,
}

impl Tooltip {
    pub fn new() -> Self {
        yew::props! {Self {
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
}

pub enum Msg {
    RealShow,
    Show,
    Hide,
}

#[doc(hidden)]
pub struct PwtTooltip {
    tooltip_ref: NodeRef,
    align_options: Option<AlignOptions>,
    show: bool,
    timeout: Option<Timeout>,
}

impl Component for PwtTooltip {
    type Message = Msg;
    type Properties = Tooltip;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            tooltip_ref: NodeRef::default(),
            show: false,
            timeout: None,
            align_options: None,
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

        let show_tooltip = self.show && ctx.props().tip.is_some();

        let content =
            Container::form_widget_props(props.std_props.clone(), Some(props.listeners.clone()))
                .children(props.children.clone())
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

        let data_show = show_tooltip.then(|| "");
        html! {
            <>
                {content}
                <div role="tooltip" aria-live="polite" class="tooltip" ref={self.tooltip_ref.clone()} data-show={data_show}>
                if let Some(tip) = &props.tip { {tip.clone()} }
                </div>
            </>
        }
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
