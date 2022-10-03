use std::rc::Rc;

use wasm_bindgen::JsCast;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use crate::prelude::*;
use crate::widget::Container;

mod menu_item;
pub use menu_item::MenuItem;

#[derive(Clone, PartialEq)]
pub enum MenuEntry{
    MenuItem(MenuItem),
    Separator,
    Component(VNode),
}

impl From<MenuItem> for MenuEntry {
    fn from(item: MenuItem) -> Self {
        Self::MenuItem(item)
    }
}

impl From<Html> for MenuEntry {
    fn from(comp: Html) -> Self {
        Self::Component(comp)
    }
}

/// Menu - A container for [MenuItem]s.
///
/// The container implements a roving focus to allow keyboard
/// navigation.
#[derive(Clone, PartialEq, Properties)]
pub struct Menu {
    #[prop_or_default]
    children: Vec<MenuEntry>,

    #[prop_or_default]
    pub class: Classes,

   // on_focus_change: Option<Callback<bool>>,
}

impl Menu {

    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to add a html class.
    pub fn class(mut self, class: impl Into<Classes>) -> Self {
        self.add_class(class);
        self
    }

    /// Method to add a html class.
    pub fn add_class(&mut self, class: impl Into<Classes>) {
        self.class.push(class);
    }


    /// Builder style method to add a simple line to separate menu items.
    pub fn with_separator(mut self) -> Self {
        self.add_item(MenuEntry::Separator);
        self
    }

    /// Builder style method to add a menu item
    pub fn with_item(mut self, child: impl Into<MenuEntry>) -> Self {
        self.add_item(child);
        self
    }

    /// Method to add a menu item
    pub fn add_item(&mut self, child: impl Into<MenuEntry>) {
        self.children.push(child.into());
    }
}

pub enum Msg {
    Next,
    Previous,
    ActivateItem(usize),
    Redraw,
}

#[doc(hidden)]
pub struct PwtMenu {
    inner_ref: NodeRef,
    cursor: usize,
}
impl PwtMenu {

    fn get_focus_el(&self, cursor: usize) -> Option<web_sys::HtmlElement> {
        let menu_el = match self.inner_ref.cast::<web_sys::Element>() {
            Some(el) => el,
            None => return None,
        };

        let selector = format!(":scope > li[data-index='{}']", cursor);
        let item_el = match menu_el.query_selector(&selector) {
            Ok(Some(item_el)) => item_el,
            _ => return None,
        };

        const FOCUSABLE_SELECTOR: &str = "a:not([disabled]), button:not([disabled]), input[type=text]:not([disabled]), [tabindex]:not([disabled])";

        let focus_el = match item_el.query_selector(FOCUSABLE_SELECTOR) {
            Ok(Some(focus_el)) => focus_el,
            _ => return None,
        };

        match focus_el.dyn_into::<web_sys::HtmlElement>() {
            Ok(el) => Some(el),
            _ => None,
        }
    }

    fn try_focus_item(&mut self, cursor: usize) -> bool {

        let focus_el = match self.get_focus_el(cursor) {
            Some(el) => el,
            None => return false,
        };

        let res = match focus_el.focus() {
            Ok(_) => true,
            Err(_) => false,
        };

        focus_el.set_tab_index(0);

        res
    }

    fn set_cursor(&mut self, cursor: usize) -> bool {
        let last_cursor = self.cursor;
        
        if self.try_focus_item(cursor) {
            self.cursor = cursor;

            if cursor != last_cursor {
                if let Some(last_el) = self.get_focus_el(last_cursor) {
                    last_el.set_tab_index(-1);
                }
            }
            
            true
        } else {
            false
        }
    }
}

impl Component for PwtMenu {
    type Message = Msg;
    type Properties = Menu;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            cursor: 0,
            inner_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::Redraw => true,
            Msg::Next => {
                let mut cursor = self.cursor;
                loop {
                    cursor += 1;
                    if cursor >= props.children.len() { return false; }
                    match props.children[cursor] {
                        MenuEntry::Separator => continue,
                        _ => if self.set_cursor(cursor) { break; },
                    }
                }
                true
            }
            Msg::Previous => {
                let mut cursor = self.cursor;
                loop {
                    if cursor == 0 { return false; }
                    cursor -= 1;
                    match props.children[cursor] {
                        MenuEntry::Separator => continue,
                        _ => if self.set_cursor(cursor) { break; },
                    }
                }
                if self.try_focus_item(cursor) {
                    self.cursor = cursor;
                }
                true
            }
            Msg::ActivateItem(cursor) => {
                if self.cursor == cursor { return false; }
                let focus_el = match self.get_focus_el(cursor) {
                    Some(el) => el,
                    None => return false,
                };
                if let Some(last_el) = self.get_focus_el(self.cursor) {
                    last_el.set_tab_index(-1);
                }
                self.cursor = cursor;
                focus_el.set_tab_index(0);               
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        Container::new()
            .node_ref(self.inner_ref.clone())
            .tag("ul")
            .class("pwt-menu")
            .class(props.class.clone())
        //.onfocusin(ctx.link().callback(|_| Msg::FocusChange(true)))
            //.onfocusout(ctx.link().callback(|_| Msg::FocusChange(false)))
            .onkeydown({
                let link = ctx.link().clone();
                move |event: KeyboardEvent| {
                    match event.key_code() {
                        40 => link.send_message(Msg::Next),
                        38 => link.send_message(Msg::Previous),
                        _ => return,
                    }
                    event.stop_propagation();
                    event.prevent_default();
                }
            })
            .children(props.children.iter().enumerate().map(|(i, entry)| {
                let active = self.cursor == i;
                let child = match entry {
                    MenuEntry::Separator => {
                        html!{<hr/>}
                    }
                    MenuEntry::Component(comp) => {
                        comp.clone()
                    }
                    MenuEntry::MenuItem(item) => {
                        item.clone()
                            .active(active)
                            .show_submenu(active)
                            .into()
                    }
                };

                Container::new()
                    .tag("li")
                    .attribute("data-index", i.to_string())
                    .class(active.then(|| "active"))
                    .with_child(child)
                    .onfocusin(ctx.link().callback(move |_| Msg::ActivateItem(i)))
                    .into()
            }))
            .into()
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let props = ctx.props();
            // initialize roving tabindex
            for i in 0..props.children.len() {
                if self.set_cursor(i) {
                    ctx.link().send_message(Msg::Redraw);
                    break;
                }
            }
        }
    }
    
}

impl Into<VNode> for Menu {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtMenu>(Rc::new(self), None);
        VNode::from(comp)
    }
}
