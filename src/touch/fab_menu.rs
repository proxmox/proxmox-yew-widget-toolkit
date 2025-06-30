use yew::prelude::*;

use crate::css::ColorScheme;
use crate::props::{ContainerBuilder, EventSubscriber, WidgetBuilder};
use crate::widget::{Button, Container};

use pwt_macros::{builder, widget};

use super::fab::FabSize;
use super::Fab;

/// [FabMenu] direction.
#[derive(Copy, Clone, PartialEq)]
pub enum FabMenuDirection {
    Up,
    Down,
}

/// [FabMenu] alignment.
#[derive(Copy, Clone, PartialEq)]
pub enum FabMenuAlign {
    Start,
    End,
}

/// An entry for a [FabMenu]
#[derive(PartialEq, Clone)]
pub struct FabMenuEntry {
    pub text: AttrValue,
    pub icon: AttrValue,
    pub on_activate: Callback<MouseEvent>,
}

impl FabMenuEntry {
    pub fn new(
        text: impl Into<AttrValue>,
        icon: impl Into<AttrValue>,
        on_activate: impl Into<Callback<MouseEvent>>,
    ) -> Self {
        Self {
            text: text.into(),
            icon: icon.into(),
            on_activate: on_activate.into(),
        }
    }
}

/// [FabMenu] Color
///
/// Determines the color of the [Fab] and the [FabMenuEntry].
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum FabMenuColor {
    #[default]
    Primary,
    Secondary,
    Tertiary,
}

/// Favorite actions button Menu.
#[widget(pwt=crate, comp=PwtFabMenu, @element)]
#[derive(Properties, Clone, PartialEq)]
#[builder]
pub struct FabMenu {
    /// The size of the [Fab]
    #[prop_or_default]
    #[builder]
    pub size: FabSize,

    /// The color scheme to apply on the [Fab] and the [FabMenuEntry]
    #[prop_or_default]
    #[builder]
    pub color: FabMenuColor,

    /// Main button Icon (CSS class).
    #[prop_or_default]
    pub main_icon_class: Option<Classes>,

    /// Main button CSS class.
    #[prop_or_default]
    pub main_button_class: Option<Classes>,

    /// Menu popup direction
    #[prop_or(FabMenuDirection::Up)]
    #[builder]
    pub direction: FabMenuDirection,

    /// Menu alignment
    ///
    #[prop_or(FabMenuAlign::End)]
    #[builder]
    pub align: FabMenuAlign,

    /// Child buttons, which popup when main button is pressed.
    ///
    /// We currently support up to 5 children.
    #[prop_or_default]
    pub children: Vec<FabMenuEntry>,
}

impl Default for FabMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl FabMenu {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to set the icon class for the main button.
    pub fn main_icon_class(mut self, class: impl Into<Classes>) -> Self {
        self.set_main_icon_class(class);
        self
    }

    /// Method to set the icon class for the main button.
    pub fn set_main_icon_class(&mut self, class: impl Into<Classes>) {
        self.main_icon_class = Some(class.into());
    }

    /// Builder style method to add a html class to the main button.
    pub fn main_button_class(mut self, class: impl Into<Classes>) -> Self {
        self.add_main_button_class(class);
        self
    }

    /// Method to add a html class to the main button.
    pub fn add_main_button_class(&mut self, class: impl Into<Classes>) {
        if let Some(main_button_class) = &mut self.main_button_class {
            main_button_class.push(class);
        } else {
            self.main_button_class = Some(class.into());
        }
    }

    /// Builder style method to add a child button
    pub fn with_child(mut self, child: impl Into<FabMenuEntry>) -> Self {
        self.add_child(child);
        self
    }

    /// Method to add a child button
    pub fn add_child(&mut self, child: impl Into<FabMenuEntry>) {
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
        Self { show_items: false }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
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

        let main_icon_class = match (self.show_items, &props.main_icon_class) {
            (false, Some(class)) => class.clone(),
            (false, None) => classes!("fa", "fa-plus"),
            (true, _) => classes!("fa", "fa-times"),
        };

        let (close_color, color) = match props.color {
            FabMenuColor::Primary => (ColorScheme::Primary, ColorScheme::PrimaryContainer),
            FabMenuColor::Secondary => (ColorScheme::Secondary, ColorScheme::SecondaryContainer),
            FabMenuColor::Tertiary => (ColorScheme::Tertiary, ColorScheme::TertiaryContainer),
        };

        let main_button = Fab::new(main_icon_class)
            .size(if self.show_items {
                FabSize::Small
            } else {
                props.size
            })
            .class(props.main_button_class.clone())
            .class(if self.show_items { close_color } else { color })
            .class(self.show_items.then_some("rounded"))
            .on_activate(ctx.link().callback(|_| Msg::Toggle));

        let mut container = Container::new()
            .with_std_props(&props.std_props)
            .listeners(&props.listeners)
            .class("pwt-fab-menu-container")
            .class(self.show_items.then_some("active"))
            .onkeydown({
                let link = ctx.link().clone();
                move |event: KeyboardEvent| {
                    if event.key() == "Escape" {
                        link.send_message(Msg::Close)
                    }
                }
            });

        for (i, child) in props.children.iter().enumerate() {
            if i >= 5 {
                log::error!("FabMenu only supports 5 child buttons.");
                break;
            }

            let on_activate = child.on_activate.clone();
            let link = ctx.link().clone();

            let child = Button::new(child.text.clone())
                .icon_class(child.icon.clone())
                .class("medium")
                .class(color)
                .class("pwt-fab-menu-item")
                .on_activate(move |event| {
                    link.send_message(Msg::Toggle);
                    on_activate.emit(event);
                });
            container.add_child(child);
        }

        Container::new()
            .with_std_props(&props.std_props)
            .listeners(&props.listeners)
            .class("pwt-fab-menu-outer")
            .class(match props.align {
                FabMenuAlign::Start => Some("pwt-fab-align-start"),
                FabMenuAlign::End => Some("pwt-fab-align-end"),
            })
            .class(match props.direction {
                FabMenuDirection::Up => "pwt-fab-direction-up",
                FabMenuDirection::Down => "pwt-fab-direction-down",
            })
            .with_child(
                Container::new()
                    .class("pwt-fab-menu-main")
                    .class(match props.size {
                        FabSize::Small => "small",
                        FabSize::Standard => "",
                        FabSize::Large => "large",
                    })
                    .with_child(main_button),
            )
            .with_child(container)
            .into()
    }
}
