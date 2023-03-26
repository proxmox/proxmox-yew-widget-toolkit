use std::rc::Rc;

use yew::prelude::*;
use yew::{
    virtual_dom::{VComp, VNode},
};
//use yew::html::IntoEventCallback;
use yew::virtual_dom::Key;

use crate::props::{ContainerBuilder, EventSubscriber, WidgetBuilder};
use crate::widget::Container;

/// Page animation style.
#[derive(Copy, Clone, PartialEq)]
pub enum PageAnimationStyle {
    Push,
    Fade,
    Cover,
}

impl Into<Classes> for PageAnimationStyle {
    fn into(self) -> Classes {
        match self {
            PageAnimationStyle::Push => "pwt-page-animation-push",
            PageAnimationStyle::Fade => "pwt-page-animation-fade",
            PageAnimationStyle::Cover => "pwt-page-animation-cover",
        }.into()
    }
}

/// Stack of Pages using animation when switching pages.
#[derive(Clone, PartialEq, Properties)]
pub struct PageStack {
    #[prop_or_default]
    stack: Vec<Html>,

    #[prop_or(PageAnimationStyle::Push)]
    animation_style: PageAnimationStyle,
}

impl PageStack {
    pub fn new(pages: Vec<Html>) -> Self {
        yew::props!(Self { stack: pages })
    }

    pub fn animation_style(mut self, style: PageAnimationStyle) -> Self {
        self.animation_style = style;
        self
    }
}


#[derive(Clone, PartialEq)]
enum ViewState {
    Normal,
    Grow,
    Shrink(Html),
}
pub enum Msg {
    AnimationEnd,
}

pub struct PmgPageStack {
    state: ViewState,
}

impl Component for PmgPageStack {
    type Message = Msg;
    type Properties = PageStack;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            state: ViewState::Normal,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::AnimationEnd => {
                log::info!("AnimationEnd");
                self.state = ViewState::Normal;
                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();

        if props.stack.len() > old_props.stack.len() {
            self.state = ViewState::Grow;
        } else if props.stack.len() < old_props.stack.len() {
            self.state = ViewState::Shrink(old_props.stack.last().unwrap().clone());
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let mut children: Vec<Html> = Vec::new();

        let mut stack = props.stack.clone();
        if let ViewState::Shrink(child) = &self.state {
            stack.push(child.clone());
        }

        let stack_len = stack.len();

        let animation: Classes = props.animation_style.into();

        for (i, child) in stack.into_iter().enumerate() {

            let top_child = (i + 1) == stack_len;
            let parent = (i + 2) == stack_len;


            let child_class = match self.state {
                ViewState::Grow if top_child => "pwt-page-grow",
                ViewState::Grow if parent => "pwt-page-shrink",
                ViewState::Shrink(_) if top_child => "pwt-page-grow-reverse",
                ViewState::Shrink(_) if parent => "pwt-page-shrink-reverse",
                _ if top_child => "pwt-page-visible",
                _ => "pwt-page-hidden",
            };

            let page = Container::new()
                .class("pwt-bg-color-neutral")
                .class("pwt-page-container")
                .class(animation.clone())
                .class(child_class)
                .key(Key::from(format!("stack-level-{i}")))
                .onanimationend(ctx.link().callback(|_| Msg::AnimationEnd))
                .with_child(child);


            children.push(page.into())
        }

        Container::new()
            .class("pwt-position-relative")
            .class("pwt-overflow-hidden")
            .attribute("style", "width: 100%; height: 100%;")
            .children(children)
            .into()
    }
}

impl Into<VNode> for PageStack {
    fn into(self) -> VNode {
        let comp = VComp::new::<PmgPageStack>(Rc::new(self), None);
        VNode::from(comp)
    }
}
