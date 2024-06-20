use std::fmt::Display;
use std::str::FromStr;

use anyhow::bail;
use web_sys::HtmlElement;

use yew::html::IntoPropValue;

use crate::dom::IntoHtmlElement;
use crate::prelude::*;
use crate::props::{EventSubscriber, WidgetBuilder, WidgetStyleBuilder};
use crate::widget::Container;

use pwt_macros::{builder, widget};

#[derive(PartialEq, Clone, Copy)]
pub enum ButtonType {
    Button,
    Submit,
    Reset,
}

impl Display for ButtonType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ButtonType::Button => "button",
            ButtonType::Submit => "submit",
            ButtonType::Reset => "reset",
        })
    }
}

impl FromStr for ButtonType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "button" => Ok(ButtonType::Button),
            "submit" => Ok(ButtonType::Submit),
            "reset" => Ok(ButtonType::Reset),
            _ => bail!("invalid button type"),
        }
    }
}

impl Default for ButtonType {
    fn default() -> Self {
        ButtonType::Button
    }
}

/// Button.
///
/// Buttons can be text only, icons with text, or icons only.
#[widget(pwt=crate, comp=crate::widget::PwtButton, @element)]
#[derive(Properties, PartialEq, Clone)]
#[builder]
pub struct Button {
    /// Button text.
    #[prop_or_default]
    pub text: Option<AttrValue>,

    /// Icon (CSS class).
    #[prop_or_default]
    pub icon_class: Option<Classes>,

    /// Html tabindex attribute.
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub tabindex: Option<i32>,

    /// ARIA label.
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub aria_label: Option<AttrValue>,

    /// Html autofocus attribute.
    #[prop_or_default]
    #[builder]
    pub autofocus: bool,

    /// Disable flag.
    #[prop_or_default]
    #[builder]
    pub disabled: bool,

    /// Draw button in pressed state (for use in Demo)
    #[prop_or_default]
    #[builder]
    pub pressed: bool,

    /// Whether to show an arrow at the end of the menu.
    #[prop_or_default]
    #[builder]
    pub show_arrow: bool,

    /// Button type property (default is "button")
    ///
    /// We overwrite the default ("submit") because that can cause unexpected
    /// problems when you have multiple "submit" buttons.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub button_type: ButtonType,
}

impl Button {
    /// Create a new button.
    pub fn new(text: impl IntoPropValue<Option<AttrValue>>) -> Self {
        yew::props!(Self {
            text: text.into_prop_value()
        })
    }

    /// Create a new icon button (without text).
    pub fn new_icon(icon_class: impl Into<Classes>) -> Self {
        yew::props!(Self {}).icon_class(icon_class)
    }

    /// Create a Refresh/Reload button.
    pub fn refresh(loading: bool) -> Self {
        let icon_class = if loading {
            "fa fa-fw pwt-loading-icon"
        } else {
            "fa fa-fw fa-refresh"
        };
        Self::new_icon(icon_class)
            .aria_label("Refresh")
            .disabled(loading)
    }

    /// Builder style method to set the icon CSS class.
    pub fn icon_class(mut self, icon_class: impl Into<Classes>) -> Self {
        self.set_icon_class(icon_class);
        self
    }

    /// Method to set the icon CSS class.
    pub fn set_icon_class(&mut self, icon_class: impl Into<Classes>) {
        self.icon_class = Some(icon_class.into());
    }
}

pub enum Msg {
    ShowRippleAnimation(PointerEvent),
    AnimationEnd,
}

#[doc(hidden)]
pub struct PwtButton {
    ripple_pos: Option<(i32, i32, i32)>,
}

impl Component for PwtButton {
    type Message = Msg;
    type Properties = Button;

    fn create(_ctx: &Context<Self>) -> Self {
        Self { ripple_pos: None }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();

        match msg {
            Msg::ShowRippleAnimation(event) => {
                if props.disabled {
                    return false;
                }
                if let Some(element) = props.std_props.node_ref.clone().into_html_element() {
                    let client = element.get_bounding_client_rect();
                    let x = event.client_x() as f64 - client.x();
                    let y = event.client_y() as f64 - client.y();
                    let width = client.width();
                    let height = client.height();
                    let radius = width.max(height);
                    self.ripple_pos = Some((x as i32, y as i32, radius as i32));
                }
                true
            }
            Msg::AnimationEnd => {
                self.ripple_pos = None;
                true
            }
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        let props = ctx.props();
        if first_render && ctx.props().autofocus {
            if let Some(button) = props.std_props.node_ref.cast::<HtmlElement>() {
                let _ = button.focus();
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let mut children = Vec::new();

        let suppress_onclick: Option<Callback<MouseEvent>> = match props.disabled {
            true => Some(Callback::from(|event: MouseEvent| {
                event.prevent_default();
                event.stop_propagation();
            })),
            false => None,
        };

        if let Some(icon_class) = &props.icon_class {
            if !icon_class.is_empty() {
                // Chromium fires onclick from nested elements, so we need to suppress that manually here
                children.push(html!{
                    <span class="pwt-font-label-large" onclick={suppress_onclick.clone()}><i role="none" class={icon_class.clone()}></i></span>
                });
            }
        }

        if let Some(text) = &props.text {
            children.push((&*text).into());
        }

        let (x, y, radius) = self.ripple_pos.unwrap_or((0, 0, 0));

        // Note: We always add the pwt-button-ripple container, and start animation
        // by setting the "animate" class. Else, onclick handler is not reliable
        // triggered (dont know why).
        children.push({
            Container::new()
                .class("pwt-button-ripple")
                .class(self.ripple_pos.is_some().then(|| "animate"))
                .style("--pwt-ripple-x", format!("{x}px"))
                .style("--pwt-ripple-y", format!("{y}px"))
                .style("--pwt-ripple-radius", format!("{radius}px"))
                .onanimationend(ctx.link().callback(|_| Msg::AnimationEnd))
                // Chromium fires onclick from nested elements, so we need to suppress that manually here
                .onclick(suppress_onclick)
                .into()
        });

        if props.show_arrow {
            children.push(html! {<i role="none" class="fa fa-caret-down"/>});
        }

        let listeners = (!props.disabled).then_some(props.listeners.clone());

        Container::form_widget_props(props.std_props.clone(), listeners)
            .children(children)
            .tag("button")
            .class("pwt-button")
            .class(props.pressed.then(|| "pressed"))
            .attribute("type", Some(props.button_type.to_string()))
            .attribute("aria-disabled", props.disabled.then(|| "true"))
            .attribute("autofocus", props.autofocus.then(|| ""))
            .attribute("aria-label", props.aria_label.clone())
            .attribute("tabindex", props.tabindex.map(|i| i.to_string()))
            .onpointerdown(ctx.link().callback(Msg::ShowRippleAnimation))
            .into()
    }
}
