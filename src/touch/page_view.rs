use std::rc::Rc;

use yew::html::IntoEventCallback;
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::impl_to_html;
use crate::props::{ContainerBuilder, WidgetBuilder};
use crate::touch::GestureDetector;
use crate::widget::Container;

use super::GestureSwipeEvent;

/// A scrollable list that works page by page.
#[derive(Properties, Clone, PartialEq)]
pub struct PageView {
    /// The yew component key.
    #[prop_or_default]
    pub key: Option<Key>,

    #[prop_or_default]
    children: Vec<VNode>,

    /// Index of the currently active/viewed page.
    #[prop_or(0)]
    pub view_page: usize,

    /// This callback is called when the user swipes to the next/previous page.
    #[prop_or_default]
    pub on_page_change: Option<Callback<usize>>,
}

impl PageView {
    /// Creates a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to set the actual page.
    pub fn view_page(mut self, page_num: usize) -> Self {
        self.set_view_page(page_num);
        self
    }

    /// Method to set the actual page.
    pub fn set_view_page(&mut self, page_num: usize) {
        self.view_page = page_num;
    }

    /// Builder style method to set the `on_page_change` callback.
    pub fn on_page_change(mut self, cb: impl IntoEventCallback<usize>) -> Self {
        self.on_page_change = cb.into_event_callback();
        self
    }
}

impl ContainerBuilder for PageView {
    fn as_children_mut(&mut self) -> &mut Vec<VNode> {
        &mut self.children
    }
}

#[doc(hidden)]
pub struct PwtPageView {}

pub enum Msg {
    NextLeft,
    NextRight,
}

impl Component for PwtPageView {
    type Message = Msg;
    type Properties = PageView;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::NextLeft => {
                if props.view_page > 0 {
                    if let Some(on_page_change) = &props.on_page_change {
                        on_page_change.emit(props.view_page - 1)
                    }
                }
                false
            }
            Msg::NextRight => {
                if props.view_page < (props.children.len() - 1) {
                    if let Some(on_page_change) = &props.on_page_change {
                        on_page_change.emit(props.view_page + 1)
                    }
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let pages: Vec<Html> = props
            .children
            .iter()
            .enumerate()
            .map(|(i, child)| {
                let pos = if i <  props.view_page {
                    -1
                } else if i > props.view_page {
                    1
                } else {
                    0
                };

                let style = format!(
                    "position:absolute;width:100%;height:100%,top:0;transition: all ease 0.5s;left:calc({}*100%);",
                    pos,
                );
                Container::new()
                    .attribute("style", style)
                    .with_child(child.clone())
                    .into()
            })
            .collect();

        GestureDetector::new(
            Container::new()
                .class("pwt-position-relative")
                .class("pwt-overflow-hidden")
                .attribute("style", "width: 100%; height: 100%;")
                .children(pages),
        )
        .on_swipe({
            let link = ctx.link().clone();
            move |event: GestureSwipeEvent| {
                if event.direction.abs() < 45.0 {
                    link.send_message(Msg::NextLeft)
                } else if event.direction.abs() > 135.0 {
                    link.send_message(Msg::NextRight)
                }
            }
        })
        .into()
    }
}

impl Into<VNode> for PageView {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtPageView>(Rc::new(self), key);
        VNode::from(comp)
    }
}

impl_to_html!(PageView);
