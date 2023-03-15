use std::rc::Rc;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::css::FlexDirection;
use crate::props::{AsClassesMut, EventSubscriber, WidgetBuilder, ContainerBuilder};
use crate::widget::{Container};

use super::{Fab, GestureSwipeEvent};

#[derive(Copy, Clone, PartialEq)]
pub enum FabMenuDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Copy, Clone, PartialEq)]
pub enum FabMenuAlign {
    Start,
    Center,
    End,
}

/// Favorite actions button Menu.
#[derive(Properties, Clone, PartialEq)]
pub struct FabMenu {
    /// The yew component key.
    pub key: Option<Key>,

    /// Main button Icon (CSS class).
    pub main_icon_class: Option<Classes>,

    /// Menu popup direction
    #[prop_or(FabMenuDirection::Up)]
    pub direction: FabMenuDirection,

    /// Menu alignment
    ///
    #[prop_or(FabMenuAlign::Center)]
    pub align: FabMenuAlign,

    /// Child buttons, which popup when main button is pressed.
    ///
    /// We currently support up to 5 children.
    #[prop_or_default]
    pub children: Vec<Fab>,
}

impl FabMenu {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to set the yew `key` property
    pub fn key(mut self, key: impl IntoPropValue<Option<Key>>) -> Self {
        self.set_key(key);
        self
    }

    /// Method to set the yew `key` property
    pub fn set_key(&mut self, key: impl IntoPropValue<Option<Key>>) {
        self.key = key.into_prop_value();
    }

    /// Builder style method to set the popup alignment
    pub fn align(mut self, align: FabMenuAlign) -> Self {
        self.set_align(align);
        self
    }

    /// Method to set the popup alignment
    pub fn set_align(&mut self, align: FabMenuAlign) {
        self.align = align;
    }

    /// Builder style method to set the popup direction
    pub fn direction(mut self, direction: FabMenuDirection) -> Self {
        self.set_direction(direction);
        self
    }

    /// Method to set the popup direction
    pub fn set_direction(&mut self, direction: FabMenuDirection) {
        self.direction = direction;
    }

    /// Builder style method to add a child button
    pub fn with_child(mut self, child: impl Into<Fab>) -> Self {
        self.add_child(child);
        self
    }

    /// Method to add a child button
    pub fn add_child(&mut self, child: impl Into<Fab>) {
        self.children.push(child.into());
    }

}

pub enum Msg {
    Toggle,
    Close,
}

#[doc(hidden)]
pub struct PwtFabMenu {
    show_items: bool,
}

impl Component for PwtFabMenu {
    type Message = Msg;
    type Properties = FabMenu;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            show_items: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Toggle => {
                self.show_items = !self.show_items;
                true
            }
            Msg::Close => {
                self.show_items = false;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let main_class = match &props.main_icon_class {
            Some(class) => class.clone(),
            None => classes!("fa", "fa-plus"),
        };

        let mut container = Container::new()
            .class("pwt-fab-menu-container")
            .class(match props.align {
                FabMenuAlign::Start => Some("pwt-fab-align-start"),
                FabMenuAlign::End => Some("pwt-fab-align-end"),
                FabMenuAlign::Center => None,
            })
            .class(match props.direction {
                FabMenuDirection::Up => "pwt-fab-direction-up",
                FabMenuDirection::Down => "pwt-fab-direction-down",
                FabMenuDirection::Left => "pwt-fab-direction-left",
                FabMenuDirection::Right => "pwt-fab-direction-right",
            })
            .class(self.show_items.then(|| "active"))
            .onkeydown({
                let link = ctx.link().clone();
                move |event: KeyboardEvent| {
                    if event.key() == "Escape" {
                        link.send_message(Msg::Close)
                    }
                }
            });

        let main_button = Fab::new(main_class)
            .on_click(ctx.link().callback(|_| Msg::Toggle));

        container.add_child(main_button);

        for (i, child) in props.children.iter().enumerate() {
            if i >= 5 {
                log::error!("FabMenu only supports 5 child buttons.");
                break;
            }
            let orig_on_click = child.on_click.clone();
            let link = ctx.link().clone();

            let child_button = child.clone()
                .small(true)
                .class("pwt-fab-menu-item")
                .on_click(move |event| {
                    link.send_message(Msg::Toggle);
                    if let Some(on_click) = &orig_on_click {
                        on_click.emit(event);
                    }
                });
            container.add_child(child_button);
        }

        container.into()
    }
}

impl Into<VNode> for FabMenu {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtFabMenu>(Rc::new(self), key);
        VNode::from(comp)
    }
}
