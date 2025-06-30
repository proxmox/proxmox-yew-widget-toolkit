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
    FadeFromRight,
    FadeFromBottom,
    Flip,
    Cover,
}

impl From<PageAnimationStyle> for Classes {
    fn from(val: PageAnimationStyle) -> Self {
        match val {
            PageAnimationStyle::Push => "pwt-page-animation-push",
            PageAnimationStyle::Fade => "pwt-page-animation-fade",
            PageAnimationStyle::FadeFromRight => "pwt-page-animation-fade-from-right",
            PageAnimationStyle::FadeFromBottom => "pwt-page-animation-fade-from-bottom",
            PageAnimationStyle::Flip => "pwt-page-animation-flip",
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
    Grow((Key, Html)),
    Shrink((Key, Html)),
}
pub enum Msg {
    AnimationEnd,
}

#[doc(hidden)]
pub struct PwtPageStack {
    state: ViewState,

    // tracked copy of props.stack, in order to assign a key to each pages
    stack: Vec<(Key, Html)>,
}

impl Component for PwtPageStack {
    type Message = Msg;
    type Properties = PageStack;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        // assign a key to each page
        let stack: Vec<(Key, Html)> = props
            .stack
            .iter()
            .map(|page| {
                (
                    Key::from(crate::widget::get_unique_element_id()),
                    page.clone(),
                )
            })
            .collect();

        Self {
            state: ViewState::Normal,
            stack,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::AnimationEnd => {
                self.state = ViewState::Normal;
                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        let props = ctx.props();

        if let Some(last) = self.stack.last() {
            match props.stack.len().cmp(&self.stack.len()) {
                std::cmp::Ordering::Less => self.state = ViewState::Shrink(last.clone()),
                std::cmp::Ordering::Greater => self.state = ViewState::Grow(last.clone()),
                _ => {}
            }
        }

        let min_len = props.stack.len().min(self.stack.len());

        let mut new_stack = Vec::new();
        for i in 0..min_len {
            let new_page = &props.stack[i];
            let changed = new_page != &self.stack[i].1;
            if changed {
                new_stack.push((
                    Key::from(crate::widget::get_unique_element_id()),
                    new_page.clone(),
                ));
            } else {
                new_stack.push(self.stack[i].clone());
            }
        }

        for i in min_len..props.stack.len() {
            new_stack.push((
                Key::from(crate::widget::get_unique_element_id()),
                props.stack[i].clone(),
            ));
        }
        self.stack = new_stack;

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let mut children: Vec<Html> = Vec::new();

        let mut stack = self.stack.clone();

        let parent_key = match &self.state {
            ViewState::Grow((parent_key, parent_page)) => {
                // show last visible page as parent
                if self.stack.iter().find(|(k, _)| k == parent_key).is_some() {
                    // parent still there
                    Some(parent_key.clone())
                } else {
                    // else we need to insert a copy temporarily
                    // Note: Grow is only set if stack size is >= 2
                    let top = stack.pop().unwrap();
                    stack.push((parent_key.clone(), parent_page.clone()));
                    stack.push(top);
                    Some(parent_key.clone())
                }
            }
            ViewState::Shrink(old_top) => {
                let parent_key = stack.last().map(|(key, _page)| key.clone());
                // show last visible page as top page
                stack.push(old_top.clone());
                parent_key
            }
            ViewState::Normal => {
                let stack_len = stack.len();
                (stack_len > 1).then(|| stack[stack_len - 2].0.clone())
            }
        };

        let stack_len = stack.len();

        let animation: Classes = props.animation_style.into();

        for (i, (child_key, child)) in stack.into_iter().enumerate() {
            let top_child = (i + 1) == stack_len;
            let parent = parent_key.as_ref() == Some(&child_key);

            let child_class = match self.state {
                ViewState::Grow(_) if top_child => "pwt-page-grow",
                ViewState::Grow(_) if parent => "pwt-page-shrink",
                ViewState::Shrink(_) if top_child => "pwt-page-grow-reverse",
                ViewState::Shrink(_) if parent => "pwt-page-shrink-reverse",
                ViewState::Normal if top_child => "pwt-page-visible",
                _ => "pwt-page-hidden",
            };

            let page = Container::new()
                .class("pwt-bg-color-neutral")
                .class("pwt-page-container")
                .class(animation.clone())
                .class(child_class)
                .key(child_key)
                .onanimationend(ctx.link().callback(|_| Msg::AnimationEnd))
                .with_child(child);

            children.push(page.into())
        }

        Container::new()
            .class("pwt-position-relative")
            .class("pwt-overflow-hidden")
            .class("pwt-page-outer")
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
