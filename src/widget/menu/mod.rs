//! Operating system style menus.

use std::rc::Rc;

use gloo_timers::callback::Timeout;

use yew::html::IntoEventCallback;

use yew::virtual_dom::{Key, VComp, VNode};

use crate::dom::focus::{get_first_focusable, FocusTracker};
use crate::widget::{get_unique_element_id, Container};
use crate::{impl_class_prop_builder, impl_yew_std_props_builder, prelude::*};

use pwt_macros::builder;

mod menu_event;
pub use menu_event::MenuEvent;

mod menu_popper;
pub(crate) use menu_popper::MenuPopper;

mod menu_item;
pub use menu_item::MenuItem;

mod menu_checkbox;
pub use menu_checkbox::MenuCheckbox;
#[doc(hidden)]
pub use menu_checkbox::PwtMenuCheckbox;

mod menu_button;
pub use menu_button::MenuButton;
#[doc(hidden)]
pub use menu_button::PwtMenuButton;

mod split_button;
#[doc(hidden)]
pub use split_button::PwtSplitButton;
pub use split_button::SplitButton;

/// Menu entries.
#[derive(Clone, PartialEq)]
pub enum MenuEntry {
    /// Checkbox or radio-group entry.
    Checkbox(MenuCheckbox),
    /// Normal item with optional icon and optional submenu.
    MenuItem(MenuItem),
    /// Separator
    Separator,
    /// Custom entry.
    Component(VNode),
}

impl From<MenuItem> for MenuEntry {
    fn from(item: MenuItem) -> Self {
        Self::MenuItem(item)
    }
}

impl From<MenuCheckbox> for MenuEntry {
    fn from(checkbox: MenuCheckbox) -> Self {
        Self::Checkbox(checkbox)
    }
}

impl From<Html> for MenuEntry {
    fn from(comp: Html) -> Self {
        Self::Component(comp)
    }
}

/// Messages for the menu controller ([Menu] or [MenuButton])
#[doc(hidden)]
pub enum MenuControllerMsg {
    Next,
    Previous,
    Collapse,
}

/// Menu - A container for [MenuEntry]s.
///
/// The container implements a roving focus to allow keyboard
/// navigation.
///
/// See <https://www.w3.org/WAI/ARIA/apg/patterns/menu/>.
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct Menu {
    #[prop_or_default]
    children: Vec<MenuEntry>,

    /// Yew component `ref`.
    #[prop_or_default]
    pub node_ref: NodeRef,

    /// The yew component key.
    #[prop_or_default]
    pub key: Option<Key>,

    /// CSS class
    #[prop_or_default]
    pub class: Classes,

    #[prop_or(250)]
    pub submenu_timeout_ms: u32,

    // Methods below are used internally.
    #[prop_or_default]
    #[builder]
    pub(crate) autofocus: bool,

    #[prop_or_default]
    #[builder]
    pub(crate) menubar: bool,

    #[prop_or_default]
    #[builder]
    pub(crate) menubar_child: bool,

    #[builder_cb(IntoEventCallback, into_event_callback, MenuControllerMsg)]
    #[prop_or_default]
    pub(crate) menu_controller: Option<Callback<MenuControllerMsg>>,

    #[builder_cb(IntoEventCallback, into_event_callback, ())]
    #[prop_or_default]
    pub(crate) on_close: Option<Callback<()>>,
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}

impl Menu {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Create a new instance with a horizontal layout.
    ///
    /// See <https://www.w3.org/WAI/ARIA/apg/patterns/menubar/>
    pub fn new_menubar() -> Self {
        Menu::new().menubar(true)
    }

    impl_yew_std_props_builder!();
    impl_class_prop_builder!();

    /// Builder style method to add a simple line to separate menu items.
    pub fn with_separator(mut self) -> Self {
        self.add_separator();
        self
    }

    /// Method to add a simple line to separate menu items.
    pub fn add_separator(&mut self) {
        self.add_item(MenuEntry::Separator);
    }

    /// Builder style method to add a menu item.
    pub fn with_item(mut self, child: impl Into<MenuEntry>) -> Self {
        self.add_item(child);
        self
    }

    /// Method to add a menu item.
    pub fn add_item(&mut self, child: impl Into<MenuEntry>) {
        self.children.push(child.into());
    }

    /// Builder style method to add multiple menu items.
    pub fn items(mut self, child: impl IntoIterator<Item = MenuEntry>) -> Self {
        self.add_items(child);
        self
    }

    /// Method to add multiple items.
    pub fn add_items(&mut self, children: impl IntoIterator<Item = MenuEntry>) {
        self.children.extend(children);
    }
}

#[doc(hidden)]
pub enum Msg {
    Collapse,
    FocusChange(bool),
    Next,
    Previous,
    DelayedNext,
    DelayedPrevious,
    ActivateItem(usize, bool),
    OnMouseOver(usize),
    SetActiveSubmenu(usize),
    ShowSubmenu(bool, bool),
    SubmenuClose,
    Redraw,
}

#[doc(hidden)]
pub struct PwtMenu {
    unique_id: String,
    inner_ref: NodeRef,
    menu_controller: Option<Callback<MenuControllerMsg>>,
    cursor: Option<usize>,
    inside_submenu: bool,
    show_submenu: bool,
    collapsed: bool,
    focus_tracker: FocusTracker,
    has_focus: bool,
    move_timeout: Option<Timeout>, // for Next/Prev
    active_submenu: Option<usize>,
    submenu_timer: Option<Timeout>,
}

impl PwtMenu {
    fn get_unique_item_id(&self, n: usize) -> String {
        format!("{}-item-{}", self.unique_id, n)
    }

    // find the first focusable element inside an menu item
    fn get_focus_el(&self, cursor: usize) -> Option<web_sys::HtmlElement> {
        let menu_el = self.inner_ref.cast::<web_sys::Element>()?;

        let selector = format!(":scope > li[data-index='{}']", cursor);
        let item_el = match menu_el.query_selector(&selector) {
            Ok(Some(item_el)) => item_el,
            _ => return None,
        };

        get_first_focusable(item_el)
    }

    fn try_focus_item(&mut self, cursor: usize, has_focus: bool) -> bool {
        let focus_el = match self.get_focus_el(cursor) {
            Some(el) => el,
            None => return false,
        };

        let res = focus_el.focus().is_ok();

        if has_focus {
            //log::info!("FOCUS {:?}", focus_el);
            focus_el.set_tab_index(0);
        }

        res
    }

    fn set_cursor(&mut self, cursor: usize, has_focus: bool) -> bool {
        let last_cursor = self.cursor;

        if self.try_focus_item(cursor, has_focus) {
            self.cursor = Some(cursor);
            self.active_submenu = Some(cursor);

            if self.cursor != last_cursor {
                if let Some(last_cursor) = last_cursor {
                    if let Some(last_el) = self.get_focus_el(last_cursor) {
                        last_el.set_tab_index(-1);
                    }
                }
            }

            true
        } else {
            false
        }
    }

    fn activate_first_item(&mut self, ctx: &Context<Self>) {
        let props = ctx.props();

        if !props.autofocus {
            return;
        }

        if self.inside_submenu {
            return;
        }

        for i in 0..props.children.len() {
            //log::info!("INIT {} {} {}", self.unique_id, props.autofocus, self.inside_submenu);
            if self.set_cursor(i, true) {
                ctx.link().send_message(Msg::Redraw);
                break;
            }
        }
    }

    fn init_roving_tabindex(&mut self, ctx: &Context<Self>) {
        let props = ctx.props();

        if let Some(cursor) = self.cursor {
            if let Some(focus_el) = self.get_focus_el(cursor) {
                focus_el.set_tab_index(0);
                return;
            }
        }

        let mut found = false;

        for i in 0..props.children.len() {
            let focus_el = match self.get_focus_el(i) {
                Some(el) => el,
                None => continue,
            };
            if !found {
                found = true;
                focus_el.set_tab_index(0);
            } else {
                focus_el.set_tab_index(-1);
            }
        }
    }
}

impl Component for PwtMenu {
    type Message = Msg;
    type Properties = Menu;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let menu_controller = if props.menubar {
            let link = ctx.link().clone();
            Some(Callback::from(move |msg: MenuControllerMsg| match msg {
                MenuControllerMsg::Next => link.send_message(Msg::Next),
                MenuControllerMsg::Previous => link.send_message(Msg::Previous),
                MenuControllerMsg::Collapse => link.send_message(Msg::Collapse),
            }))
        } else {
            props.menu_controller.clone()
        };

        let focus_tracker = FocusTracker::new(ctx.link().callback(Msg::FocusChange));

        Self {
            cursor: None,
            unique_id: get_unique_element_id(),
            inner_ref: props.node_ref.clone(),
            menu_controller,
            inside_submenu: false,
            show_submenu: !props.menubar,
            collapsed: true,
            focus_tracker,
            has_focus: false,
            move_timeout: None,
            active_submenu: None,
            submenu_timer: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        if let Some(timeout) = self.submenu_timer.take() {
            timeout.cancel();
        }
        match msg {
            // Note: only used by menubar
            Msg::FocusChange(has_focus) => {
                self.has_focus = has_focus;
                if !has_focus {
                    self.show_submenu = false;
                    self.inside_submenu = false;
                    self.collapsed = true;
                    self.init_roving_tabindex(ctx);
                }
                true
            }
            // Note: only used by menubar
            Msg::Collapse => {
                self.show_submenu = false;
                self.inside_submenu = false;
                self.collapsed = true;
                if let Some(cursor) = self.cursor {
                    self.set_cursor(cursor, true);
                }
                //log::info!("CLOSE {} {} {}", self.unique_id, self.show_submenu, self.inside_submenu);
                true
            }
            Msg::Redraw => true,
            Msg::Next => {
                let link = ctx.link().clone();
                self.move_timeout = Some(Timeout::new(1, move || {
                    link.send_message(Msg::DelayedNext);
                }));
                false
            }
            Msg::DelayedNext => {
                let mut cursor = match self.cursor {
                    Some(cursor) => cursor + 1,
                    None => 0,
                };

                loop {
                    if cursor >= props.children.len() {
                        return false;
                    }
                    match props.children[cursor] {
                        MenuEntry::Separator => {}
                        _ => {
                            if self.set_cursor(cursor, true) {
                                break;
                            }
                        }
                    }
                    cursor += 1;
                }
                self.show_submenu = true;
                self.collapsed = false;
                self.inside_submenu = false;
                true
            }
            Msg::Previous => {
                let link = ctx.link().clone();
                self.move_timeout = Some(Timeout::new(1, move || {
                    link.send_message(Msg::DelayedPrevious);
                }));
                false
            }
            Msg::DelayedPrevious => {
                let mut cursor = match self.cursor {
                    Some(cursor) => {
                        if cursor == 0 {
                            return false;
                        } else {
                            cursor - 1
                        }
                    }
                    None => {
                        if props.children.is_empty() {
                            return false;
                        } else {
                            props.children.len() - 1
                        }
                    }
                };

                loop {
                    match props.children[cursor] {
                        MenuEntry::Separator => {}
                        _ => {
                            if self.set_cursor(cursor, true) {
                                break;
                            }
                        }
                    }
                    if cursor == 0 {
                        return false;
                    }
                    cursor -= 1;
                }
                self.show_submenu = true;
                self.collapsed = false;
                self.inside_submenu = false;
                true
            }
            Msg::ShowSubmenu(show, with_keyboard) => {
                let cursor = match self.cursor {
                    Some(cursor) => cursor,
                    None => return false,
                };

                if let Some(MenuEntry::MenuItem(item)) = props.children.get(cursor) {
                    self.active_submenu = Some(cursor);
                    self.collapsed = false;
                    if item.has_menu() {
                        if self.show_submenu != show {
                            self.show_submenu = show;
                            self.inside_submenu = false;
                            return true;
                        } else if show {
                            self.inside_submenu = with_keyboard;
                            //log::info!("MOVE FOCUS");
                            return true;
                        }
                    }
                }

                if !show {
                    if let Some(on_close) = &props.on_close {
                        //log::info!("PROPAGATE CLOSE {} {}", self.unique_id, show);
                        on_close.emit(());
                    }
                }

                true
            }
            Msg::SubmenuClose => {
                let cursor = match self.cursor {
                    Some(cursor) => cursor,
                    None => return false,
                };
                self.inside_submenu = false;
                self.try_focus_item(cursor, true); // fixme : use set_cursor

                true
            }
            // Note: called onfocusin
            Msg::ActivateItem(cursor, inside_submenu) => {
                self.inside_submenu = inside_submenu;
                self.show_submenu = true;

                let focus_el = match self.get_focus_el(cursor) {
                    Some(el) => el,
                    None => return false,
                };

                if let Some(last_cursor) = self.cursor {
                    if let Some(last_el) = self.get_focus_el(last_cursor) {
                        last_el.set_tab_index(-1);
                    }
                }

                if self.cursor != Some(cursor) {
                    //log::info!("DELETE COLAPSE FLAGE");
                    // self.collapsed = false;
                }

                self.cursor = Some(cursor);

                //log::info!("ACTIVATE {} {} {}", self.unique_id, props.autofocus, inside_submenu);
                if !inside_submenu {
                    focus_el.set_tab_index(0);
                }
                true
            }
            Msg::OnMouseOver(index) => {
                let link = ctx.link().clone();
                if props.menubar || props.submenu_timeout_ms == 0 {
                    self.active_submenu = Some(index);
                    self.show_submenu = true;
                } else {
                    self.submenu_timer = Some(Timeout::new(props.submenu_timeout_ms, move || {
                        link.send_message(Msg::SetActiveSubmenu(index))
                    }));
                }
                true
            }
            Msg::SetActiveSubmenu(index) => {
                self.active_submenu = Some(index);
                self.show_submenu = true;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let focus_on_over = !props.menubar || self.has_focus;

        let menu = Container::from_tag("ul")
            .node_ref(self.inner_ref.clone())
            .attribute("role", if props.menubar { "menubar" } else { "menu" })
            .attribute("id", self.unique_id.clone())
            .class(if props.menubar {
                "pwt-menubar"
            } else {
                "pwt-menu"
            })
            .class(props.class.clone())
            .children(props.children.iter().enumerate().map(|(i, entry)| {
                let mut has_submenu = false;
                let active = self.cursor == Some(i);
                let submenu_active = self.active_submenu == Some(i);
                let show_submenu =
                    submenu_active && self.show_submenu && !(props.menubar && self.collapsed);
                let child = match entry {
                    MenuEntry::Separator => {
                        if props.menubar {
                            html! {<div role="separator" class="pwt-h-100 pwt-vertical-rule"/>}
                        } else {
                            html! {<div role="separator" class="pwt-w-100 pwt-horizontal-rule"/>}
                        }
                    }
                    MenuEntry::Component(comp) => comp.clone(),
                    MenuEntry::MenuItem(item) => {
                        has_submenu = item.menu.is_some();
                        item.clone()
                            .active(active || submenu_active)
                            .on_close(ctx.link().callback(|_| Msg::SubmenuClose))
                            .menu_controller(self.menu_controller.clone())
                            .show_submenu(show_submenu)
                            .focus_submenu(self.inside_submenu)
                            .inside_menubar(props.menubar)
                            .into()
                    }
                    MenuEntry::Checkbox(checkbox) => checkbox
                        .clone()
                        .menu_controller(self.menu_controller.clone())
                        .into(),
                };

                let item_id = self.get_unique_item_id(i);
                let link = ctx.link().clone();
                let menu_controller = self.menu_controller.clone();
                let menubar = props.menubar;
                let menubar_child = props.menubar_child;

                Container::from_tag("li")
                    .attribute("id", item_id.clone())
                    .attribute("data-index", i.to_string()) // fixme: remove
                    .attribute("role", "none")
                    .class((active).then_some("active"))
                    .with_child(child)
                    .onkeydown({
                        let link = ctx.link().clone();
                        move |event: KeyboardEvent| {
                            if menubar {
                                match event.key().as_str() {
                                    "ArrowRight" => link.send_message(Msg::Next),
                                    "ArrowLeft" => link.send_message(Msg::Previous),
                                    "Enter" | "ArrowDown" | " " => {
                                        link.send_message(Msg::ShowSubmenu(true, true))
                                    }
                                    "ArrowUp" => link.send_message(Msg::ShowSubmenu(false, true)),
                                    "Escape" => link.send_message(Msg::Collapse),
                                    _ => return,
                                }
                            } else {
                                match event.key().as_str() {
                                    "ArrowDown" => link.send_message(Msg::Next),
                                    "ArrowUp" => link.send_message(Msg::Previous),
                                    "Enter" | " " => {
                                        if has_submenu {
                                            link.send_message(Msg::ShowSubmenu(true, true));
                                        }
                                    }
                                    "ArrowRight" => {
                                        if !has_submenu {
                                            if let Some(menu_controller) = &menu_controller {
                                                menu_controller.emit(MenuControllerMsg::Next);
                                            }
                                        } else {
                                            link.send_message(Msg::ShowSubmenu(true, true));
                                        }
                                    }
                                    "ArrowLeft" => {
                                        link.send_message(Msg::ShowSubmenu(false, true));
                                        if menubar_child {
                                            if let Some(menu_controller) = &menu_controller {
                                                menu_controller.emit(MenuControllerMsg::Previous);
                                            }
                                        }
                                    }
                                    _ => return,
                                }
                            }
                            event.stop_propagation();
                        }
                    })
                    .onmouseover({
                        let item_id = item_id.clone();
                        move |event: MouseEvent| {
                            event.stop_propagation();
                            if focus_on_over {
                                dom_focus_submenu(&item_id);
                                link.send_message(Msg::OnMouseOver(i))
                            }
                        }
                    })
                    .ondblclick(Callback::from(move |event: MouseEvent| {
                        event.stop_propagation();
                    }))
                    .onclick(ctx.link().callback({
                        move |event: MouseEvent| {
                            event.stop_propagation();
                            Msg::ShowSubmenu(true, false)
                        }
                    }))
                    .onfocusin(ctx.link().callback(move |event: FocusEvent| {
                        let inside_submenu = dom_focus_inside_submenu(&event, &item_id);
                        Msg::ActivateItem(i, inside_submenu)
                    }))
                    .into()
            }));

        if props.menubar {
            menu.onfocusin(self.focus_tracker.get_focus_callback(true))
                .onfocusout(self.focus_tracker.get_focus_callback(false))
                .into()
        } else {
            menu.into()
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        if props.autofocus != old_props.autofocus {
            //log::info!("FOCUSCANGE {}", self.unique_id);
            if props.autofocus && !self.inside_submenu {
                if let Some(cursor) = self.cursor {
                    if self.set_cursor(cursor, true) {
                        ctx.link().send_message(Msg::Redraw);
                    }
                } else {
                    self.activate_first_item(ctx);
                }
            }
        }
        true
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let props = ctx.props();
            if props.menubar {
                self.init_roving_tabindex(ctx);
            } else {
                self.activate_first_item(ctx);
            }
        }
    }
}

impl From<Menu> for VNode {
    fn from(props: Menu) -> Self {
        let key = props.key.clone();
        let comp = VComp::new::<PwtMenu>(Rc::new(props), key);
        VNode::from(comp)
    }
}

// Note: With yew, Element.current_target() always points to the HtmlBodyElement
// see: https://github.com/yewstack/yew/issues/2572
// We need to use unique element IDs instead.
fn dom_focus_inside_submenu(event: &FocusEvent, item_id: &str) -> bool {
    let mut cur_el: Option<web_sys::Element> = event.target_dyn_into();
    while let Some(el) = cur_el {
        if el.id() == item_id {
            break;
        }

        if el.tag_name() == "UL" {
            return true;
        }

        cur_el = el.parent_element();
    }

    false
}

fn dom_focus_submenu(item_id: &str) {
    let el = match gloo_utils::document().get_element_by_id(item_id) {
        Some(el) => el,
        None => return,
    };

    if let Some(focus_el) = get_first_focusable(el) {
        let _ = focus_el.focus();
    }
}

/// Generates an artificial `keydown` event for the `Escape` key.
///
/// This closes the menu if sent from inside a menubar or menubutton.
pub fn dispatch_menu_close_event(event: impl AsRef<web_sys::Event>) {
    let target: web_sys::Element = event.target_unchecked_into();

    let mut options = web_sys::KeyboardEventInit::new();
    options.bubbles(true);
    options.cancelable(true);
    options.key("Escape");
    options.key_code(27);

    let event =
        web_sys::KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &options).unwrap();

    let _ = target.dispatch_event(&event);
}
