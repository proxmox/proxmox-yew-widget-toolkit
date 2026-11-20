use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::Key;

use crate::props::{ContainerBuilder, EventSubscriber, IntoVTag, WidgetBuilder};
use crate::widget::Container;

use pwt_macros::{builder, widget};

/// Widget that flies from one page to the next when the [PageStack](super::PageStack) switches
/// pages.
///
/// If the page that leaves and the page that arrives both have a hero with the same tag, the hero
/// moves and scales from its place on the old page to its place on the new one, above both pages,
/// instead of animating with its page. The typical use is a list thumbnail that grows into the
/// header image of a detail page. [Scaffold](super::Scaffold) uses a hero for its floating action
/// button, so a button that both pages have stays in place.
///
/// What flies is a static copy of the hero on the new page, while both real heroes are hidden. So
/// live content like a canvas or the value of an input does not fly along, and styles that the
/// hero gets from its ancestors do not apply to the copy. Heroes should therefore be
/// self-contained, like icons, images, titles, or buttons. A hero that is not rendered with the
/// first render of its page, for example because its content still loads, does not fly.
///
/// Tags only need to be unique within a page. Without a page stack, or if only one page has the
/// tag, the hero is a plain container that animates with its page.
#[widget(pwt=crate, comp=PwtHero, @element, @container)]
#[derive(Properties, Clone, PartialEq)]
#[builder]
pub struct Hero {
    /// Pairs heroes of the old and the new page.
    #[builder(IntoPropValue, into_prop_value)]
    pub tag: AttrValue,
}

impl Hero {
    /// Create a new instance.
    pub fn new(tag: impl Into<AttrValue>) -> Self {
        yew::props!(Self { tag: tag.into() })
    }
}

struct HeroEntry {
    id: usize,
    page: Key,
    tag: AttrValue,
    node_ref: NodeRef,
}

/// The heroes of all pages of a page stack.
#[derive(Default)]
pub(crate) struct HeroRegistry {
    next_id: usize,
    heroes: Vec<HeroEntry>,
}

impl HeroRegistry {
    fn register(&mut self, page: Key, tag: AttrValue, node_ref: NodeRef) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.heroes.push(HeroEntry {
            id,
            page,
            tag,
            node_ref,
        });
        id
    }

    fn set_tag(&mut self, id: usize, tag: AttrValue) {
        if let Some(entry) = self.heroes.iter_mut().find(|entry| entry.id == id) {
            entry.tag = tag;
        }
    }

    fn unregister(&mut self, id: usize) {
        self.heroes.retain(|entry| entry.id != id);
    }

    /// Returns the heroes of a page that are in the DOM, only the first one of each tag.
    pub(crate) fn page_heroes(&self, page: &Key) -> Vec<(AttrValue, web_sys::HtmlElement)> {
        let mut list: Vec<(AttrValue, web_sys::HtmlElement)> = Vec::new();
        for entry in self.heroes.iter().filter(|entry| &entry.page == page) {
            let Some(el) = entry.node_ref.cast::<web_sys::HtmlElement>() else {
                continue;
            };
            if list.iter().any(|(tag, _)| *tag == entry.tag) {
                log::warn!("page has more than one hero with tag '{}'", entry.tag);
                continue;
            }
            list.push((entry.tag.clone(), el));
        }
        list
    }

    /// Returns the elements of all heroes that are in the DOM.
    pub(crate) fn elements(&self) -> Vec<web_sys::Element> {
        self.heroes
            .iter()
            .filter_map(|entry| entry.node_ref.get())
            .filter_map(|node| node.dyn_into::<web_sys::Element>().ok())
            .collect()
    }
}

/// Provided by the page stack for each page, so heroes know which page they are on.
#[derive(Clone)]
pub(crate) struct HeroContext {
    pub(crate) registry: Rc<RefCell<HeroRegistry>>,
    pub(crate) page: Key,
}

impl PartialEq for HeroContext {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.registry, &other.registry) && self.page == other.page
    }
}

#[doc(hidden)]
pub struct PwtHero {
    node_ref: NodeRef,
    registration: Option<(HeroContext, usize)>,
}

impl Component for PwtHero {
    type Message = ();
    type Properties = Hero;

    fn create(ctx: &Context<Self>) -> Self {
        let node_ref = NodeRef::default();

        // The context of a page does not change, as a page with another key is a new page.
        let registration =
            ctx.link()
                .context::<HeroContext>(Callback::noop())
                .map(|(hero_ctx, _)| {
                    let id = hero_ctx.registry.borrow_mut().register(
                        hero_ctx.page.clone(),
                        ctx.props().tag.clone(),
                        node_ref.clone(),
                    );
                    (hero_ctx, id)
                });

        Self {
            node_ref,
            registration,
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let tag = &ctx.props().tag;
        if *tag != old_props.tag
            && let Some((hero_ctx, id)) = &self.registration
        {
            hero_ctx.registry.borrow_mut().set_tag(*id, tag.clone());
        }
        true
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        if let Some((hero_ctx, id)) = &self.registration {
            hero_ctx.registry.borrow_mut().unregister(*id);
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        Container::new()
            .with_std_props(&props.std_props)
            .listeners(&props.listeners)
            .children(props.children.clone())
            .into_html_with_ref(self.node_ref.clone())
    }
}
