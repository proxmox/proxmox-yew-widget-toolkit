use yew::prelude::*;

use crate::css::{self, ColorScheme};
use crate::props::{ContainerBuilder, CssPaddingBuilder, EventSubscriber, WidgetBuilder};
use crate::touch::{SideDialog, SideDialogController};
use crate::tr;
use crate::widget::{Button, Column, Container};

use pwt_macros::{builder, widget};

use super::fab::FabSize;
use super::Fab;

/// [FabMenu] variant
#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub enum FabMenuVariant {
    #[default]
    Sheet,
    Material3,
}

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

    /// Menu variant
    #[prop_or_default]
    #[builder]
    pub variant: FabMenuVariant,

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

        let (fab_size, fab_classes) = match (props.variant, self.show_items) {
            (FabMenuVariant::Material3, true) => (FabSize::Small, classes!(close_color, "rounded")),
            (_, false) | (FabMenuVariant::Sheet, true) => (props.size, classes!()),
        };

        let main_button = Fab::new(main_icon_class)
            .size(fab_size)
            .class(props.main_button_class.clone())
            .class(fab_classes)
            .on_activate(ctx.link().callback(|_| Msg::Toggle));

        let btn_class = match props.variant {
            FabMenuVariant::Sheet => classes!("pwt-button-text"),
            FabMenuVariant::Material3 => classes!(color, "pwt-fab-menu-item", "medium"),
        };

        let children = props.children.iter().enumerate().filter_map(|(i, child)| {
            if i >= 5 {
                log::error!("FabMenu only supports 5 child buttons.");
                return None;
            }

            let on_activate = child.on_activate.clone();
            let link = ctx.link().clone();

            Some(
                Button::new(child.text.clone())
                    .icon_class(child.icon.clone())
                    .class(btn_class.clone())
                    .on_activate(move |event| {
                        link.send_message(Msg::Toggle);
                        on_activate.emit(event);
                    })
                    .into(),
            )
        });

        let container: Option<Html> = match props.variant {
            FabMenuVariant::Sheet => {
                let controller = SideDialogController::new();
                self.show_items.then_some(
                    SideDialog::new()
                        .controller(controller.clone())
                        .location(crate::touch::SideDialogLocation::Bottom)
                        .on_close(ctx.link().callback(|_| Msg::Toggle))
                        .with_child(
                            Column::new()
                                .class(css::FlexFit)
                                .padding(2)
                                .gap(1)
                                .children(children)
                                .with_child(html!(<hr />))
                                .with_child(
                                    Button::new(tr!("Cancel"))
                                        .class("pwt-button-text")
                                        .on_activate(move |_| controller.close_dialog()),
                                ),
                        )
                        .into(),
                )
            }
            FabMenuVariant::Material3 => Some(
                Container::new()
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
                    })
                    .children(children)
                    .into(),
            ),
        };

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
            .with_optional_child(container)
            .into()
    }
}
