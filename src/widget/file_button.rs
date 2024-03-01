
use web_sys::{HtmlElement, HtmlInputElement};

use yew::html::{IntoEventCallback, IntoPropValue};

use crate::dom::IntoHtmlElement;
use crate::prelude::*;
use crate::props::{EventSubscriber, WidgetBuilder};
use crate::widget::Container;

use pwt_macros::{builder, widget};

/// File Selection Button.
///
/// FileButtons can be text only, icons with text, or icons only.
#[widget(pwt=crate, comp=crate::widget::PwtFileButton, @element)]
#[derive(Properties, PartialEq, Clone)]
#[builder]
pub struct FileButton {
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

    /// Defines the file types the file input should accept.
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub accept: Option<AttrValue>,

    /// Allow to select more than one file.
    #[prop_or_default]
    #[builder]
    pub multiple: bool,

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

    #[prop_or_default]
    #[builder_cb(IntoEventCallback, into_event_callback, Option<web_sys::FileList>)]
    pub on_change: Option<Callback<Option<web_sys::FileList>>>,
}

impl FileButton {
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
pub struct PwtFileButton {
    ripple_pos: Option<(i32, i32, i32)>,
    input_ref: NodeRef,
}

impl Component for PwtFileButton {
    type Message = Msg;
    type Properties = FileButton;

    fn create(_ctx: &Context<Self>) -> Self {
        Self { ripple_pos: None, input_ref: NodeRef::default() }
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

        children.push(
            Container::new()
                .tag("input")
                .node_ref(self.input_ref.clone())
                .attribute("type", "file")
                .attribute("accept", props.accept.clone())
                .attribute("multiple", props.multiple.then(|| ""))
                .class("pwt-d-none")
                .onchange({
                    let on_change = props.on_change.clone();
                    let input_ref = self.input_ref.clone();
                    move |_| {
                        if let Some(on_change) = &on_change  {
                            if let Some(el) = input_ref.cast::<HtmlInputElement>() {
                                on_change.emit(el.files());
                            }
                        }
                    }
                })
                .into()
        );

        let listeners = (!props.disabled).then_some(props.listeners.clone());

        Container::form_widget_props(props.std_props.clone(), listeners)
            .children(children)
            .tag("label")
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
