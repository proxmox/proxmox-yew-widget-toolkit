use std::borrow::Cow;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::{ApplyAttributeAs, Listeners, VList, VTag};

use crate::props::{EventSubscriber, WidgetBuilder};
use crate::widget::dom::IntoHtmlElement;
use crate::widget::Container;

use pwt_macros::widget;

/// Button.
///
/// Buttons can be text only, icons with text, or icons only.
#[widget(pwt=crate, comp=crate::widget::PwtButton, @element)]
#[derive(Properties, PartialEq, Clone)]
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
    ShowRippleAnimation(i32, i32, i32),
    AnimationEnd,
}

#[doc(hidden)]
pub struct PwtButton {
    ripple_pos: Option<(i32, i32, i32)>,
    onpointerdown: Rc<yew::html::onpointerdown::Wrapper>,
}

impl Component for PwtButton {
    type Message = Msg;
    type Properties = Button;

    fn create(ctx: &Context<Self>) -> Self {
        let onpointerdown = Callback::from({
            let link = ctx.link().clone();
            let node_ref = ctx.props().std_props.node_ref.clone();
            move |event: PointerEvent| {
                if let Some(element) = node_ref.clone().into_html_element() {
                    let client = element.get_bounding_client_rect();
                    let x = event.client_x() as f64 - client.x();
                    let y = event.client_y() as f64 - client.y();
                    let width = client.width();
                    let height = client.height();
                    let radius = width.max(height);
                    link.send_message(Msg::ShowRippleAnimation(x as i32, y as i32, radius as i32));
                }
            }
        });

        Self {
            ripple_pos: None,
            onpointerdown: Rc::new(yew::html::onpointerdown::Wrapper::new(onpointerdown)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ShowRippleAnimation(x, y, radius) => {
                self.ripple_pos = Some((x, y, radius));
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

        let mut attributes = props.std_props.cumulate_attributes(Some(classes!(
            "pwt-button",
            if props.pressed { "pressed" } else { "" }
        )));
        let attr_map = attributes.get_mut_index_map();

        if props.disabled {
            attr_map.insert(
                AttrValue::Static("disabled"),
                (AttrValue::Static(""), ApplyAttributeAs::Attribute),
            );
        }
        if props.autofocus {
            attr_map.insert(
                AttrValue::Static("autofocus"),
                (AttrValue::Static(""), ApplyAttributeAs::Attribute),
            );
        }
        if let Some(ref aria_label) = props.aria_label {
            attr_map.insert(
                AttrValue::Static("aria-label"),
                (aria_label.clone(), ApplyAttributeAs::Attribute),
            );
        }
        if let Some(ref tabindex) = props.tabindex {
            attr_map.insert(
                AttrValue::Static("tabindex"),
                (tabindex.to_string().into(), ApplyAttributeAs::Attribute),
            );
        }

        let mut children = Vec::new();

        if let Some(icon_class) = &props.icon_class {
            if !icon_class.is_empty() {
                // Chromium fires onclick from nested elements, so we need to suppress that manually here
                let onclick: Option<Callback<MouseEvent>> = match props.disabled {
                    true => Some(Callback::from(|event: MouseEvent| {
                        event.prevent_default();
                        event.stop_propagation();
                    })),
                    false => None,
                };
                children.push(html!{
                    <span class="pwt-font-label-large"><i {onclick} role="none" aria-hidden="true" class={icon_class.clone()}></i></span>
                });
            }
        }

        if let Some(text) = &props.text {
            children.push((&*text).into());
        }

        if let Some((x, y, radius)) = self.ripple_pos {
            children.push({
                let style = format!(
                    "--pwt-ripple-x: {x}px; --pwt-ripple-y: {y}px; --pwt-ripple-radius: {radius}px;"
                );
                Container::new()
                    .class("pwt-button-ripple")
                    .attribute("style", style)
                    .onanimationend(ctx.link().callback(|_| Msg::AnimationEnd))
                    .into()
            });
        }

        let mut listeners = props.listeners.listeners.clone();
        listeners.push(Some(self.onpointerdown.clone()));

        let listeners = Listeners::Pending(listeners.into_boxed_slice());

        VTag::__new_other(
            Cow::Borrowed("button"),
            props.std_props.node_ref.clone(),
            props.std_props.key.clone(),
            attributes,
            listeners,
            VList::with_children(children, None),
        )
        .into()
    }
}
