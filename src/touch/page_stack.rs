use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};
//use yew::html::IntoEventCallback;
use yew::virtual_dom::Key;

use crate::props::{
    ContainerBuilder, CssLength, EventSubscriber, WidgetBuilder, WidgetStyleBuilder,
};
use crate::widget::Container;

/// Page animation style.
#[derive(Copy, Clone, PartialEq)]
pub enum PageAnimationStyle {
    Push,
    Fade,
    Cover,
}

impl From<PageAnimationStyle> for Classes {
    fn from(val: PageAnimationStyle) -> Self {
        match val {
            PageAnimationStyle::Push => "pwt-page-animation-push",
            PageAnimationStyle::Fade => "pwt-page-animation-fade",
            PageAnimationStyle::Cover => "pwt-page-animation-cover",
        }
        .into()
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
    Grow(Html),
    Shrink(Html),
}
pub enum Msg {
    AnimationEnd,
}

#[doc(hidden)]
pub struct PwtPageStack {
    state: ViewState,
}

impl Component for PwtPageStack {
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

        if let Some(last) = old_props.stack.last() {
            match props.stack.len().cmp(&old_props.stack.len()) {
                std::cmp::Ordering::Less => self.state = ViewState::Shrink(last.clone()),
                std::cmp::Ordering::Greater => self.state = ViewState::Grow(last.clone()),
                _ => {}
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let mut children: Vec<Html> = Vec::new();

        let mut stack = props.stack.clone();
        if let ViewState::Grow(parent) = &self.state {
            let stack_len = stack.len();
            if stack_len > 1 {
                // show last visible page as parent
                stack[stack_len - 2] = parent.clone();
            }
        }
        if let ViewState::Shrink(child) = &self.state {
            // show last visible page as top page
            stack.push(child.clone());
        }

        let stack_len = stack.len();

        let animation: Classes = props.animation_style.into();

        for (i, child) in stack.into_iter().enumerate() {
            let top_child = (i + 1) == stack_len;
            let parent = (i + 2) == stack_len;

            let child_class = match self.state {
                ViewState::Grow(_) if top_child => "pwt-page-grow",
                ViewState::Grow(_) if parent => "pwt-page-shrink",
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
            .width(CssLength::Fraction(1.0))
            .height(CssLength::Fraction(1.0))
            .children(children)
            .into()
    }
}

impl From<PageStack> for VNode {
    fn from(val: PageStack) -> Self {
        let comp = VComp::new::<PwtPageStack>(Rc::new(val), None);
        VNode::from(comp)
    }
}
