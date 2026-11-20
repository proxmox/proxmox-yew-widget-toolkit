use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
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

use super::hero::{HeroContext, HeroRegistry};

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
///
/// When a page is pushed or popped with an animation style set, each [Hero](super::Hero) of the
/// new top page flies from the place of the hero with the same tag on the old top page.
///
/// A page that is still at the same position after a change is kept, if it has the same key as
/// before, even if its properties changed, so it is updated instead of created again. A page
/// without a key is only kept if it did not change at all.
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

/// A rectangle relative to the page stack.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Rect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl Rect {
    fn of(el: &web_sys::Element, outer: &web_sys::DomRect) -> Self {
        let rect = el.get_bounding_client_rect();
        Self {
            x: rect.x() - outer.x(),
            y: rect.y() - outer.y(),
            width: rect.width(),
            height: rect.height(),
        }
    }
}

/// The transform that moves a hero from its place on the target rectangle back to the start of
/// its flight, with the transform origin in its top left corner.
fn flight_transform(from: Rect, to: Rect) -> String {
    let scale = |from: f64, to: f64| if to > 0.0 { from / to } else { 1.0 };
    format!(
        "translate({}px, {}px) scale({}, {})",
        from.x - to.x,
        from.y - to.y,
        scale(from.width, to.width),
        scale(from.height, to.height),
    )
}

struct HeroFlight {
    tag: AttrValue,
    copy: web_sys::Element,
    heroes: [web_sys::Element; 2],
}

#[doc(hidden)]
pub struct PwtPageStack {
    state: ViewState,
    outer_ref: NodeRef,
    hero_layer_ref: NodeRef,

    // tracked copy of props.stack, in order to assign a key to each pages
    stack: Vec<PageEntry>,

    heroes: Rc<RefCell<HeroRegistry>>,
    // heroes of the old top page, measured before the DOM update
    departing_heroes: Vec<(AttrValue, Rect, web_sys::Element)>,
    flights: Vec<HeroFlight>,
}

impl PwtPageStack {
    /// Measures the heroes of the top page before the pages switch.
    fn prepare_hero_flights(&mut self) {
        let Some(outer) = self.outer_ref.cast::<web_sys::Element>() else {
            return;
        };
        let outer_rect = outer.get_bounding_client_rect();

        // A flight that is cut short starts the next one from where it is.
        let in_flight: Vec<(AttrValue, Rect)> = self
            .flights
            .iter()
            .map(|flight| (flight.tag.clone(), Rect::of(&flight.copy, &outer_rect)))
            .collect();
        self.finish_hero_flights();

        if let Some(top) = self.stack.last() {
            self.departing_heroes = self
                .heroes
                .borrow()
                .page_heroes(&top.key)
                .into_iter()
                .map(|(tag, el)| {
                    let rect = in_flight
                        .iter()
                        .find(|(t, _)| *t == tag)
                        .map(|(_, rect)| *rect)
                        .unwrap_or_else(|| Rect::of(&el, &outer_rect));
                    (tag, rect, el.into())
                })
                .collect();
        }

        // Only after measuring: the old top page is still visible until the next render, and a
        // floating action button there would start to scale in. From then on, all pages animate
        // or hide, so no visible hero is affected.
        for el in self.heroes.borrow().elements() {
            let _ = el.class_list().remove_1("pwt-hero-landed");
        }
    }

    /// Lets the heroes of the old top page fly to the place of those of the new top page.
    fn start_hero_flights(&mut self) {
        let departing = std::mem::take(&mut self.departing_heroes);
        if departing.is_empty() {
            return;
        }
        let (Some(outer), Some(layer), Some(top)) = (
            self.outer_ref.cast::<web_sys::Element>(),
            self.hero_layer_ref.cast::<web_sys::Element>(),
            self.stack.last(),
        ) else {
            return;
        };

        let pairs: Vec<_> = self
            .heroes
            .borrow()
            .page_heroes(&top.key)
            .into_iter()
            .filter_map(|(tag, el)| {
                let (_, from, from_el) = departing.iter().find(|(t, _, _)| *t == tag)?;
                Some((tag, *from, from_el.clone(), el))
            })
            .collect();
        if pairs.is_empty() {
            return;
        }

        // The new page sits at the start of its animation now. Measure where its heroes end up
        // with the page animations switched off. They start anew once switched on again, before
        // anything was painted.
        let class_list = outer.class_list();
        let _ = class_list.add_1("pwt-page-measure");
        let outer_rect = outer.get_bounding_client_rect();
        let targets: Vec<Rect> = pairs
            .iter()
            .map(|(_, _, _, el)| Rect::of(el, &outer_rect))
            .collect();
        let _ = class_list.remove_1("pwt-page-measure");

        for ((tag, from, from_el, to_el), to) in pairs.into_iter().zip(targets) {
            let Some(copy) = to_el
                .clone_node_with_deep(true)
                .ok()
                .and_then(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
            else {
                continue;
            };

            let _ = copy.remove_attribute("id");
            if let Ok(list) = copy.query_selector_all("[id]") {
                for i in 0..list.length() {
                    if let Some(el) = list
                        .item(i)
                        .and_then(|n| n.dyn_into::<web_sys::Element>().ok())
                    {
                        let _ = el.remove_attribute("id");
                    }
                }
            }
            let _ = copy.set_attribute("aria-hidden", "true");
            let _ = copy.set_attribute("inert", "");
            let _ = copy.class_list().add_1("pwt-hero-flight");

            // Important, so the position and size win over those of the hero's own classes and
            // styles, whether physical or logical.
            let style = copy.style();
            let set = |name: &str, value: &str| {
                let _ = style.set_property_with_priority(name, value, "important");
            };
            set("position", "absolute");
            set("inset", "auto");
            set("margin", "0");
            set("box-sizing", "border-box");
            set("left", &format!("{}px", to.x));
            set("top", &format!("{}px", to.y));
            set("width", &format!("{}px", to.width));
            set("height", &format!("{}px", to.height));
            set("--pwt-hero-from", &flight_transform(from, to));

            if layer.append_child(&copy).is_err() {
                continue;
            }

            let to_el: web_sys::Element = to_el.into();
            let _ = from_el.class_list().add_1("pwt-hero-hidden");
            let _ = to_el.class_list().add_1("pwt-hero-hidden");
            // Keeps a floating action button from scaling in after it flew.
            let _ = to_el.class_list().add_1("pwt-hero-landed");

            self.flights.push(HeroFlight {
                tag,
                copy: copy.into(),
                heroes: [from_el, to_el],
            });
        }
    }

    fn finish_hero_flights(&mut self) {
        for flight in self.flights.drain(..) {
            flight.copy.remove();
            for el in flight.heroes {
                let _ = el.class_list().remove_1("pwt-hero-hidden");
            }
        }
    }
}

// Pages with a key are the same page as long as the key is, pages without one only if nothing
// changed.
fn same_page(new: &Html, old: &Html) -> bool {
    match (new.key(), old.key()) {
        (Some(new_key), Some(old_key)) => new_key == old_key,
        _ => new == old,
    }
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
            hero_layer_ref: NodeRef::default(),
            heroes: Rc::new(RefCell::new(HeroRegistry::default())),
            departing_heroes: Vec::new(),
            flights: Vec::new(),
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
                let state = match props.stack.len().cmp(&self.stack.len()) {
                    std::cmp::Ordering::Less => Some(ViewState::Shrink(last.clone())),
                    std::cmp::Ordering::Greater => Some(ViewState::Grow(last.clone())),
                    _ => None,
                };
                if let Some(state) = state {
                    self.state = state;
                    self.prepare_hero_flights();
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
            if same_page(new_page, &self.stack[i].page) {
                let mut entry = self.stack[i].clone();
                entry.page = new_page.clone();
                entry.nav_ctx = nav_ctx.clone();
                new_stack.push(entry);
            } else {
                new_stack.push(PageEntry::new(new_page.clone(), nav_ctx.clone()));
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

            let hero_ctx = HeroContext {
                registry: self.heroes.clone(),
                page: entry.key.clone(),
            };
            let child = html! {
                <ContextProvider<HeroContext> context={hero_ctx}>
                    {child}
                </ContextProvider<HeroContext>>
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

        // All children need a key, else yew does not match the pages by theirs.
        children.push(
            Container::new()
                .key("pwt-page-hero-layer")
                .class("pwt-page-hero-layer")
                .into_html_with_ref(self.hero_layer_ref.clone()),
        );

        Container::new()
            .class("pwt-position-relative")
            .class("pwt-overflow-hidden")
            .class("pwt-page-outer")
            .width(CssLength::Fraction(1.0))
            .height(CssLength::Fraction(1.0))
            .children(children)
            .into_html_with_ref(self.outer_ref.clone())
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        match self.state {
            // The flights take as long as the page animations, which ended now. Ending them only
            // once the page shows its final state shows the heroes in the same paint.
            ViewState::Normal => self.finish_hero_flights(),
            _ => self.start_hero_flights(),
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        self.finish_hero_flights();
    }
}

impl From<PageStack> for VNode {
    fn from(val: PageStack) -> Self {
        let comp = VComp::new::<PwtPageStack>(Rc::new(val), None);
        VNode::from(comp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, PartialEq, Properties)]
    struct PageProps {
        value: u32,
    }

    #[function_component]
    fn Page(props: &PageProps) -> Html {
        html! {{props.value}}
    }

    fn rect(x: f64, y: f64, width: f64, height: f64) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn flight_starts_at_the_old_place() {
        let from = rect(10.0, 20.0, 40.0, 30.0);
        let to = rect(50.0, 100.0, 80.0, 60.0);
        assert_eq!(
            flight_transform(from, to),
            "translate(-40px, -80px) scale(0.5, 0.5)"
        );
    }

    #[test]
    fn flight_in_place_does_not_move() {
        let r = rect(10.0, 20.0, 40.0, 30.0);
        assert_eq!(flight_transform(r, r), "translate(0px, 0px) scale(1, 1)");
    }

    #[test]
    fn flight_to_an_empty_target_does_not_scale() {
        let from = rect(0.0, 0.0, 40.0, 30.0);
        let to = rect(0.0, 0.0, 0.0, 0.0);
        assert_eq!(
            flight_transform(from, to),
            "translate(0px, 0px) scale(1, 1)"
        );
    }

    #[test]
    fn keyed_page_stays_with_new_properties() {
        let old = html! {<Page key="item" value={1}/>};
        let new = html! {<Page key="item" value={2}/>};
        assert!(same_page(&new, &old));
    }

    #[test]
    fn page_with_other_key_is_new() {
        let old = html! {<Page key="item-1" value={1}/>};
        let new = html! {<Page key="item-2" value={1}/>};
        assert!(!same_page(&new, &old));
    }

    #[test]
    fn unkeyed_page_stays_only_unchanged() {
        let old = html! {<Page value={1}/>};
        assert!(same_page(&html! {<Page value={1}/>}, &old));
        assert!(!same_page(&html! {<Page value={2}/>}, &old));
    }

    #[test]
    fn adding_or_removing_a_key_makes_a_new_page() {
        let keyed = html! {<Page key="item" value={1}/>};
        let unkeyed = html! {<Page value={1}/>};
        assert!(!same_page(&keyed, &unkeyed));
        assert!(!same_page(&unkeyed, &keyed));
    }
}
