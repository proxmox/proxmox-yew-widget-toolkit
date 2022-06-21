use std::fmt::Debug;
use std::rc::Rc;
use std::marker::PhantomData;

use yew::prelude::*;

use wasm_bindgen::JsCast;
use wasm_bindgen::UnwrapThrowExt;
use wasm_bindgen::{prelude::*};

use crate::web_sys_ext::{ResizeObserver, ResizeObserverEntry};
use crate::props::RenderFn;

pub enum Msg {
    ViewportResize(i32, i32),
    ScrollTo(i32, i32)
}

// VirtualScroller properties
#[derive(Properties, Clone, PartialEq)]
pub struct Props<T>
where
    T: Clone + PartialEq + 'static,
{
    pub items: Rc<Vec<T>>,

    /// Height of each row (pixels).
    pub row_height: i32,

    pub render: RenderFn<T>,
}

pub struct VirtualScroll<T> {
    viewport_ref: NodeRef,
    viewport_height: i32,
    scroll_top: i32,
    visible_rows: i32,
    size_closure: Option<Closure::<dyn Fn(Vec<ResizeObserverEntry>)>>, //keep it alive
    observer: Option<ResizeObserver>, // to unobserve
    _phantom: PhantomData<T>
}

impl <T> Component for VirtualScroll<T>
where
    T: Into<Html> + Clone + PartialEq + Debug + std::fmt::Display +'static,
{
    type Message = Msg;
    type Properties = Props<T>;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            viewport_ref: NodeRef::default(),
            viewport_height: 0,
            visible_rows: 0,
            scroll_top: 0,
            size_closure: None,
            observer: None,
            _phantom: PhantomData::<T>,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ViewportResize(_width, height) => {
                //log::info!("VR {} {}", width, height);
                self.viewport_height = height;
                self.visible_rows = (height / ctx.props().row_height) + 2;
                true
            }
            Msg::ScrollTo(_x, y) => {
                //log::info!("ST {} {}", x, y);
                self.scroll_top = y;
                true
           }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
 
        let mut start = (self.scroll_top / ctx.props().row_height) as usize;
        if start > 0 { start -= 1; }
        let end = (start+self.visible_rows as usize).min(ctx.props().items.len()) as usize;
        let items = &ctx.props().items[start..end];

        let item_style = format!("height:{}px", ctx.props().row_height);
        let content: Vec<Html> = items.iter().map(|item| {
             html!{
                <div style={item_style.clone()}>{ ctx.props().render.apply(item) }</div>
            }
        }).collect();

        let virtual_height =  ctx.props().items.len() * ctx.props().row_height as usize;

        let viewport_ref = self.viewport_ref.clone();
        let onscroll = ctx.link().batch_callback(move |_: Event| {
            if let Some(el) = viewport_ref.cast::<web_sys::Element>() {
                Some(Msg::ScrollTo(el.scroll_left(), el.scroll_top()))
            } else {
                None
            }
        });

        let content_style = format!("height:{}px", virtual_height);
        let window_style = format!(
            // "transform: translateY({}px);",
            "position:relative;top:{}px;",
            start * (ctx.props().row_height as usize),
        );
        let viewport_style = "border: 1px solid; overflow:auto;";

        html!{
            <div {onscroll} ref={self.viewport_ref.clone()} style={viewport_style}>
                <div style={content_style}>
                    <div style={window_style}>
                        {content}
                    </div>
                </div>
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            if let Some(el) = self.viewport_ref.cast::<web_sys::Element>() {
                let link = ctx.link().clone();

                let size_closure = Closure::wrap(Box::new(move |entries: Vec<ResizeObserverEntry>| {
                    if entries.len() == 1 {
                        let el = entries[0].target();
                        link.send_message(Msg::ViewportResize(el.client_width(), el.client_height()));
                    } else {
                        unreachable!();
                    }
                }) as Box<dyn Fn(Vec<ResizeObserverEntry>)>);

                let observer = ResizeObserver::new(size_closure.as_ref().unchecked_ref()).unwrap_throw();
                observer.observe(&el);
                self.size_closure = Some(size_closure);
                self.observer = Some(observer);
            }
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        if let Some(observer) = self.observer.take() {
            observer.disconnect();
        }
    }
}
