use std::rc::Rc;

use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};
//use yew::html::IntoEventCallback;
use yew::virtual_dom::Key;

use crate::props::{
    ContainerBuilder, CssLength, EventSubscriber, IntoVTag, WidgetBuilder, WidgetStyleBuilder,
};
use crate::state::NavigationContext;
use crate::widget::Container;

/// Page animation style.
///
/// The animations come from the theme CSS. Without a style, pages switch without animation.
///
/// Which style fits depends on how the pages relate:
///
/// - Going deeper into a hierarchy and back, like from a list to a detail page:
///   [FadeFromRight](Self::FadeFromRight) looks like current Android, [Parallax](Self::Parallax)
///   like iOS.
/// - Moving sideways between pages of the same level, like wizard steps or swipeable pages:
///   [Push](Self::Push) shows them side by side, like a strip that scrolls by one page.
/// - Opening a page for a short task on top of the current one, like a form for a new item:
///   [FadeFromBottom](Self::FadeFromBottom) or [Cover](Self::Cover).
/// - Switching between unrelated pages, like top level sections: [Fade](Self::Fade).
/// - [Zoom](Self::Zoom) looks like older Android versions, [Flip](Self::Flip) is a special effect,
///   like turning a card.
#[derive(Copy, Clone, PartialEq)]
pub enum PageAnimationStyle {
    /// The new page pushes the old one out, both move by the full width.
    Push,
    /// The pages cross-fade without moving.
    Fade,
    /// Both pages move by a quarter of the width, the old one fades out quickly and the new one
    /// fades in, like on Android 14 and later.
    FadeFromRight,
    /// Like [FadeFromRight](Self::FadeFromRight), but both pages move up by a tenth of the height.
    FadeFromBottom,
    /// The page turns around its vertical axis like a card, with the new page on its back.
    Flip,
    /// The new page slides in over the old one, which shrinks into the background.
    Cover,
    /// The new page slides over the old one, which moves slower and darkens, like on iOS.
    Parallax,
    /// The new page fades in while growing to full size, like on Android 10 to 13.
    Zoom,
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
            PageAnimationStyle::Parallax => "pwt-page-animation-parallax",
            PageAnimationStyle::Zoom => "pwt-page-animation-zoom",
        }
        .into()
    }
}

/// Stack of Pages using animation when switching pages.
///
/// If a [NavigationContext] is set, each page gets it through its own
/// [ContextProvider]. Pages that leave the stack keep the context they had while
/// they animate out, so routed widgets on them (like
/// [NavigationBar](super::NavigationBar)) do not switch to the new route. This
/// does not apply to the location provided by `yew_router` (`Switch`, `use_location`),
/// which is always the current one.
#[derive(Clone, PartialEq, Properties)]
pub struct PageStack {
    #[prop_or_default]
    stack: Vec<Html>,

    #[prop_or_default]
    animation_style: Option<PageAnimationStyle>,

    #[prop_or_default]
    navigation_context: Option<NavigationContext>,
}

impl PageStack {
    pub fn new(pages: Vec<Html>) -> Self {
        yew::props!(Self { stack: pages })
    }

    pub fn animation_style(
        mut self,
        style: impl IntoPropValue<Option<PageAnimationStyle>>,
    ) -> Self {
        self.animation_style = style.into_prop_value();
        self
    }

    /// Builder style method to set the navigation context of the current pages.
    pub fn navigation_context(
        mut self,
        nav_ctx: impl IntoPropValue<Option<NavigationContext>>,
    ) -> Self {
        self.navigation_context = nav_ctx.into_prop_value();
        self
    }
}

#[derive(Clone, PartialEq)]
struct PageEntry {
    key: Key,
    page: Html,
    nav_ctx: Option<NavigationContext>,
}

impl PageEntry {
    fn new(page: Html, nav_ctx: Option<NavigationContext>) -> Self {
        Self {
            key: Key::from(crate::widget::get_unique_element_id()),
            page,
            nav_ctx,
        }
    }
}

#[derive(Clone, PartialEq)]
enum ViewState {
    Normal,
    Grow(PageEntry),
    Shrink(PageEntry),
}
pub enum Msg {
    AnimationEnd,
}

#[doc(hidden)]
pub struct PwtPageStack {
    state: ViewState,
    outer_ref: NodeRef,

    // tracked copy of props.stack, in order to assign a key to each pages
    stack: Vec<PageEntry>,
}

impl Component for PwtPageStack {
    type Message = Msg;
    type Properties = PageStack;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        // assign a key to each page
        let stack: Vec<PageEntry> = props
            .stack
            .iter()
            .map(|page| PageEntry::new(page.clone(), props.navigation_context.clone()))
            .collect();

        Self {
            state: ViewState::Normal,
            outer_ref: NodeRef::default(),
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
            if props.animation_style.is_some() {
                match props.stack.len().cmp(&self.stack.len()) {
                    std::cmp::Ordering::Less => self.state = ViewState::Shrink(last.clone()),
                    std::cmp::Ordering::Greater => self.state = ViewState::Grow(last.clone()),
                    _ => {}
                }
            }
        }

        let min_len = props.stack.len().min(self.stack.len());

        // Only pages that stay on the stack follow the new context, the leaving page
        // captured in the view state above keeps the one it was rendered with.
        let nav_ctx = &props.navigation_context;
        let mut new_stack = Vec::new();
        for i in 0..min_len {
            let new_page = &props.stack[i];
            let changed = new_page != &self.stack[i].page;
            if changed {
                new_stack.push(PageEntry::new(new_page.clone(), nav_ctx.clone()));
            } else {
                let mut entry = self.stack[i].clone();
                entry.nav_ctx = nav_ctx.clone();
                new_stack.push(entry);
            }
        }

        for page in &props.stack[min_len..] {
            new_stack.push(PageEntry::new(page.clone(), nav_ctx.clone()));
        }
        self.stack = new_stack;

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let mut children: Vec<Html> = Vec::new();

        let mut stack = self.stack.clone();

        let parent_key = match &self.state {
            ViewState::Grow(parent) => {
                // show last visible page as parent
                if self.stack.iter().any(|entry| entry.key == parent.key) {
                    // parent still there
                    Some(parent.key.clone())
                } else {
                    // else we need to insert a copy temporarily
                    // Note: Grow is only set if stack size is >= 2
                    let top = stack.pop().unwrap();
                    stack.push(parent.clone());
                    stack.push(top);
                    Some(parent.key.clone())
                }
            }
            ViewState::Shrink(old_top) => {
                let parent_key = stack.last().map(|entry| entry.key.clone());
                // show last visible page as top page
                stack.push(old_top.clone());
                parent_key
            }
            ViewState::Normal => {
                let stack_len = stack.len();
                (stack_len > 1).then(|| stack[stack_len - 2].key.clone())
            }
        };

        let stack_len = stack.len();

        let animation: Classes = props.animation_style.into();

        for (i, entry) in stack.into_iter().enumerate() {
            let top_child = (i + 1) == stack_len;
            let parent = parent_key.as_ref() == Some(&entry.key);

            let child_class = match self.state {
                ViewState::Grow(_) if top_child => "pwt-page-grow",
                ViewState::Grow(_) if parent => "pwt-page-shrink",
                ViewState::Shrink(_) if top_child => "pwt-page-grow-reverse",
                ViewState::Shrink(_) if parent => "pwt-page-shrink-reverse",
                ViewState::Normal if top_child => "pwt-page-visible",
                _ => "pwt-page-hidden",
            };

            let child = match entry.nav_ctx {
                Some(nav_ctx) => html! {
                    <ContextProvider<NavigationContext> context={nav_ctx}>
                        {entry.page}
                    </ContextProvider<NavigationContext>>
                },
                None => entry.page,
            };

            let page = Container::new()
                .class("pwt-bg-color-neutral")
                .class("pwt-page-container")
                .class(animation.clone())
                .class(child_class)
                .key(entry.key)
                .onanimationend(ctx.link().batch_callback({
                    let outer_ref = self.outer_ref.clone();
                    // Animations inside a page, like a button ripple, bubble up as well. Yew
                    // delegates events, so the current target is not the page container.
                    move |event: AnimationEvent| {
                        let target = event.target_dyn_into::<web_sys::Node>()?;
                        let parent = target.parent_node()?;
                        (Some(parent) == outer_ref.get()).then_some(Msg::AnimationEnd)
                    }
                }))
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
            .into_html_with_ref(self.outer_ref.clone())
    }
}

impl From<PageStack> for VNode {
    fn from(val: PageStack) -> Self {
        let comp = VComp::new::<PwtPageStack>(Rc::new(val), None);
        VNode::from(comp)
    }
}
