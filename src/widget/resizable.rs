use yew::prelude::*;

use gloo_events::EventListener;
use wasm_bindgen::JsCast;
use wasm_bindgen::UnwrapThrowExt;


pub struct Resizable {
    node_ref: NodeRef,
    width: i32,
    mousemove_listener: Option<EventListener>,
    mouseup_listener: Option<EventListener>,
}

pub enum Msg {
    StartResize,
    StopResize,
    MouseMove(i32)
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub justify: String, // flex-start, flex-end, center
    pub onresize: Callback<i32>,
    #[prop_or_default]
    pub children: Children,
}

impl Component for Resizable {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            node_ref: NodeRef::default(),
            width: 0,
            mousemove_listener: None,
            mouseup_listener: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::MouseMove(x) => {
                if let Some(el) = self.node_ref.cast::<web_sys::Element>() {

                    let rect = el.get_bounding_client_rect();
                    let new_width = x - (rect.x() as i32);
                    //log::info!("MOVE {} {} {} {}", el.client_left(), rect.x(), x, new_width);
                    self.width = new_width.max(40);
                    ctx.props().onresize.emit(self.width);
                } else {
                    unreachable!();
                }
                true
            }
            Msg::StopResize => {
                self.mouseup_listener = None;
                self.mousemove_listener = None;
                false
            }
            Msg::StartResize => {
                let window = web_sys::window().unwrap();
                let link = ctx.link();
                let onmousemove = link.callback(|e: Event| {
                    let event = e.dyn_ref::<web_sys::MouseEvent>().unwrap_throw();
                    Msg::MouseMove(event.client_x())
                });
                let mousemove_listener = EventListener::new(
                    &window,
                    "mousemove",
                    move |e| onmousemove.emit(e.clone()),
                );
                self.mousemove_listener = Some(mousemove_listener);

                let onmouseup = link.callback(|_: Event| Msg::StopResize);
                let mouseup_listener = EventListener::new(
                    &window,
                    "mouseup",
                    move |e| onmouseup.emit(e.clone()),
                );
                self.mouseup_listener = Some(mouseup_listener);

                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {

        let onmousedown = ctx.link().callback(|_| Msg::StartResize);

        let style = "height:100%;display:flex; align-items:stretch; justify-content:space-between;";
        let style = if self.width > 0 {
            format!("{} width: {}px;", style, self.width)
        } else {
            format!("{}", style)
        };

        let title_style = format!(
            "display: flex; flex: 1 1 auto; align-items:flex-end; justify-content: {};",
            ctx.props().justify,
        );
        html! {
            <div ref={self.node_ref.clone()} style={style}>
                <div style={title_style}>
                   { for ctx.props().children.iter() }
                </div>
                <div {onmousedown} class="resize-handle"/>
                </div>
        }
    }
}
