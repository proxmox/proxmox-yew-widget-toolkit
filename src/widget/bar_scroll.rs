use yew::prelude::*;

use gloo_timers::callback::Timeout;

use crate::props::{ContainerBuilder, EventSubscriber, WidgetBuilder};
use crate::widget::{Container, SizeObserver};

use pwt_macros::widget;

/// Scroll content horizontally.
#[widget(pwt=crate, comp=PwtBarScroll, @element)]
#[derive(Clone, PartialEq, Properties)]
pub struct BarScroll {
    content: Html,
}

impl BarScroll {
    /// Creates a new instance.
    pub fn new(content: impl Into<Html>) -> Self {
        yew::props!(Self { content: content.into() })
    }
}

#[doc(hidden)]
pub enum Msg {
    ScrollResize(f64, f64),
    ContentResize(f64, f64),
    HandleResize(f64, f64),
    Scroll,
    ScrollStop,
    ScrollLeft,
    ScrollRight,
}

enum ScrollMode {
    None,
    Left,
    Right,
}

#[doc(hidden)]
pub struct PwtBarScroll {
    handle_ref: NodeRef,
    handle_size_observer: Option<SizeObserver>,
    scroll_ref: NodeRef,
    content_ref: NodeRef,
    content_size_observer: Option<SizeObserver>,
    scroll_size_observer: Option<SizeObserver>,
    width: f64,
    handle_width: f64,
    content_width: f64,
    content_height: f64,
    pos: f64,
    scroll_mode: ScrollMode,
    scroll_timeout: Option<Timeout>,
}

impl PwtBarScroll {

    fn set_scroll_timeout(&mut self, ctx: &Context<Self>) {
        if self.scroll_timeout.is_some() { return; }
        let link = ctx.link().clone();
        self.scroll_timeout = Some(Timeout::new(1, move || {
            link.send_message(Msg::Scroll);
        }));
    }
}

impl Component for PwtBarScroll {
    type Message = Msg;
    type Properties = BarScroll;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            handle_ref: NodeRef::default(),
            scroll_ref: NodeRef::default(),
            content_ref: NodeRef::default(),
            handle_size_observer: None,
            content_size_observer: None,
            scroll_size_observer: None,
            width: 0f64,
            handle_width: 0f64,
            content_width: 0f64,
            content_height: 0f64,
            pos: 0f64,
            scroll_mode: ScrollMode::None,
            scroll_timeout: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ScrollLeft => {
                self.scroll_mode = ScrollMode::Left;
                self.set_scroll_timeout(ctx);
                false
            }
            Msg::ScrollRight => {
                self.scroll_mode = ScrollMode::Right;
                self.set_scroll_timeout(ctx);
                false
            }
            Msg::ScrollResize(width, _height) => {
                self.width = width;
                true
            }
            Msg::ContentResize(width, height) => {
                self.content_width = width;
                self.content_height = height;
                true
            }
            Msg::HandleResize(width, _height) => {
                self.handle_width = width;
                true
            }
            Msg::Scroll => {
                self.scroll_timeout = None;

                let el = match self.scroll_ref.cast::<web_sys::Element>() {
                    None => return false,
                    Some(el) => el,
                };

                let width = el.get_bounding_client_rect().width();
                let diff = self.content_width - width;

                if diff <= 0f64 {
                    el.set_scroll_left(0);
                    self.pos = 0.0;
                    self.scroll_timeout = None;
                    return true;
                }

                let inc = (1.0/diff)*2.0;

                match self.scroll_mode  {
                    ScrollMode::None => {
                        return true;
                    }
                    ScrollMode::Right => {
                        self.pos = (self.pos + inc).min(1.0);
                        el.set_scroll_left((diff*self.pos) as i32);
                        if self.pos == 1f64 {
                            self.scroll_mode = ScrollMode::None;
                            return true;
                        }
                    }
                    ScrollMode::Left => {
                        self.pos = (self.pos - inc).max(0f64);
                        el.set_scroll_left((diff*self.pos) as i32);
                        if self.pos == 0f64 {
                            self.scroll_mode = ScrollMode::None;
                            return true;
                        }
                    }
                }

                self.set_scroll_timeout(ctx);

                true
            }
            Msg::ScrollStop => {
                self.scroll_mode = ScrollMode::None;
                self.scroll_timeout = None;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let content = Container::new()
            .node_ref(self.content_ref.clone())
            .class("pwt-position-absolute")
            .with_child(props.content.clone());

       let scroll = Container::new()
            .node_ref(self.scroll_ref.clone())
            .attribute("style", format!(
                "position:relative;cursor:{};overflow:hidden;flex:1 1 auto;height:{}px;",
                if self.scroll_timeout.is_some() { "wait" } else { "normal" },
                self.content_height,
            ))
            .with_child(content);

        let arrow_visible = (self.width + 2.0*self.handle_width) < self.content_width;

        let left = Container::new()
            .node_ref(self.handle_ref.clone())
            .class("pwt-autoscroll-left-arrow")
            .class(arrow_visible.then(|| "visible"))
            .class((self.pos <= 0.0).then(|| "disabled"))
            .with_child(html!{<i class="fa fa-chevron-left"/>})
            .onmousedown(ctx.link().callback(|_| Msg::ScrollLeft))
            .onmouseout(ctx.link().callback(|_| Msg::ScrollStop))
            .onmouseup(ctx.link().callback(|_| Msg::ScrollStop));

        let right = Container::new()
            .class("pwt-autoscroll-right-arrow")
            .class(arrow_visible.then(|| "visible"))
            .class((self.pos >= 1.0).then(|| "disabled"))
            .with_child(html!{<i class="fa fa-chevron-right"/>})
            .onmousedown(ctx.link().callback(|_| Msg::ScrollRight))
            .onmouseout(ctx.link().callback(|_| Msg::ScrollStop))
            .onmouseup(ctx.link().callback(|_| Msg::ScrollStop));

        yew::props!(Container {
            std_props: props.std_props.clone(),
            listeners: props.listeners.clone(),
        })
            .class("pwt-autoscroll")
            .with_child(left)
            .with_child(scroll)
            .with_child(right)
            .into()

    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            if let Some(el) = self.scroll_ref.cast::<web_sys::Element>() {
                let link = ctx.link().clone();
                let size_observer = SizeObserver::new(&el, move |(width, height)| {
                    link.send_message(Msg::ScrollResize(width, height));
                });
                self.scroll_size_observer = Some(size_observer);
            }
            if let Some(el) = self.content_ref.cast::<web_sys::Element>() {
                let link = ctx.link().clone();
                let size_observer = SizeObserver::new(&el, move |(width, height)| {
                    link.send_message(Msg::ContentResize(width, height));
                });
                self.content_size_observer = Some(size_observer);
            }
            if let Some(el) = self.handle_ref.cast::<web_sys::Element>() {
                let link = ctx.link().clone();
                let size_observer = SizeObserver::new(&el, move |(width, height)| {
                    link.send_message(Msg::HandleResize(width, height));
                });
                self.handle_size_observer = Some(size_observer);
            }
        }

    }
}
