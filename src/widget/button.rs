use web_sys::HtmlElement;

use yew::html::IntoPropValue;

use crate::prelude::*;
use crate::props::{EventSubscriber, WidgetBuilder};
use crate::widget::dom::IntoHtmlElement;
use crate::widget::Container;

use pwt_macros::{builder, widget};

/// Button.
///
/// Buttons can be text only, icons with text, or icons only.
#[widget(pwt=crate, comp=crate::widget::PwtButton, @element)]
#[derive(Properties, PartialEq, Clone)]
#[builder]
pub struct Button {
    /// Button text.
    pub text: Option<AttrValue>,
    /// Icon (CSS class).
    pub icon_class: Option<Classes>,

    /// Html tabindex attribute.
    pub tabindex: Option<i32>,
    /// ARIA label.
    pub aria_label: Option<AttrValue>,
    /// Html placeholder attribute.
    pub placeholder: Option<AttrValue>,

    /// Html autofocus attribute.
    #[prop_or_default]
    pub autofocus: bool,

    /// Disable flag.
    #[prop_or_default]
    pub disabled: bool,

    /// Draw button in pressed state (for use in Demo)
    #[prop_or_default]
    pub pressed: bool,

    /// Whether to show an arrow at the end of the menu.
    #[prop_or_default]
    #[builder]
    pub show_arrow: bool,
}

impl Button {
    /// Create a new button.
    pub fn new(text: impl IntoPropValue<Option<AttrValue>>) -> Self {
        yew::props!(Self {
            text: text.into_prop_value()
        })
    }

    /// Builder style method to set the html aria-label attribute.
    pub fn aria_label(mut self, label: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_aria_label(label);
        self
    }

    /// Method to set the html aria-label attribute.
    pub fn set_aria_label(&mut self, label: impl IntoPropValue<Option<AttrValue>>) {
        self.aria_label = label.into_prop_value();
    }

    /// Builder style method to set the html tabindex attribute.
    pub fn tabindex(mut self, index: impl IntoPropValue<Option<i32>>) -> Self {
        self.set_tabindex(index);
        self
    }

    /// Method to set the html tabindex attribute.
    pub fn set_tabindex(&mut self, index: impl IntoPropValue<Option<i32>>) {
        self.tabindex = index.into_prop_value();
    }

    /// Builder style method to set the autofocus flag.
    pub fn autofocus(mut self, autofocus: bool) -> Self {
        self.set_autofocus(autofocus);
        self
    }

    /// Method to set the autofocus flag.
    pub fn set_autofocus(&mut self, autofocus: bool) {
        self.autofocus = autofocus;
    }

    /// Builder style method to set the disabled flag.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.set_disabled(disabled);
        self
    }

    /// Method to set the disabled flag.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Builder style method to set the pressed flag.
    pub fn pressed(mut self, pressed: bool) -> Self {
        self.set_pressed(pressed);
        self
    }

    /// Method to set the pressed flag.
    pub fn set_pressed(&mut self, pressed: bool) {
        self.pressed = pressed;
    }

    /// Create a new icon button (without text).
    pub fn new_icon(icon_class: impl Into<Classes>) -> Self {
        yew::props!(Self {}).icon_class(icon_class)
    }

    /// Create a Refresh/Reload button.
    pub fn refresh(loading: bool) -> Self {
        let icon_class = if loading {
            "fa fa-fw fa-spinner fa-pulse"
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
            let style = format!(
                "--pwt-ripple-x: {x}px; --pwt-ripple-y: {y}px; --pwt-ripple-radius: {radius}px;"
            );
            Container::new()
                .class("pwt-button-ripple")
                .class(self.ripple_pos.is_some().then(|| "animate"))
                .attribute("style", style)
                .onanimationend(ctx.link().callback(|_| Msg::AnimationEnd))
                // Chromium fires onclick from nested elements, so we need to suppress that manually here
                .onclick(suppress_onclick)
                .into()
        });

        if props.show_arrow {
            children.push(html! {<i role="none" class="fa fa-caret-down"/>});
        }

        Container::form_widget_props(props.std_props.clone(), Some(props.listeners.clone()))
            .children(children)
            .tag("button")
            .class("pwt-button")
            .class(props.pressed.then(|| "pressed"))
            .attribute("aria-disabled", props.disabled.then(|| "true"))
            .attribute("autofocus", props.autofocus.then(|| ""))
            .attribute("aria-label", props.aria_label.clone())
            .attribute("tabindex", props.tabindex.map(|i| i.to_string()))
            .onpointerdown(ctx.link().callback(Msg::ShowRippleAnimation))
            .into()
    }
}
